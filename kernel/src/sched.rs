//! Scheduler.

/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.

pub(crate) mod event;

use core::sync::atomic::{self, Ordering};
use core::mem::MaybeUninit;
use core::ptr::NonNull;

use event::Event;
use crate::task::{self, Task, Transition};
use crate::syscall;
use crate::time;
use crate::sync::critical_section;
use crate::mem::{
    linked_list::*,
    boxed::Box,
    array_pool::ArrayPool,
    pool_allocator,
};

use bern_arch::{ICore, IScheduler, IStartup, IMemoryProtection};
use bern_arch::arch::{ArchCore, Arch};
use bern_arch::memory_protection::{Config, Type, Access, Permission};
use bern_conf::CONF;

// These statics are MaybeUninit because, there currently no solution to
// initialize an array dependent on a `const` size and a non-copy type.
// These attempts didn't work:
// - const fn with internal MaybeUninit: not stable yet
// - proc_macro: cannot evaluate const
type TaskPool = ArrayPool<Node<Task>, { CONF.task.pool_size }>;
type EventPool = ArrayPool<Node<Event>, { CONF.event.pool_size }>;
static mut TASK_POOL: MaybeUninit<TaskPool> = MaybeUninit::uninit();
static mut EVENT_POOL: MaybeUninit<EventPool> = MaybeUninit::uninit();

static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();

// todo: split scheduler into kernel and scheduler

struct Scheduler {
    core: ArchCore,
    task_running: Option<Box<Node<Task>>>,
    tasks_ready: [LinkedList<Task, TaskPool>; CONF.task.pool_size],
    tasks_sleeping: LinkedList<Task, TaskPool>,
    tasks_terminated: LinkedList<Task, TaskPool>,
    events: LinkedList<Event, EventPool>,
    event_counter: usize,
}

/// Initialize scheduler.
///
/// **Note:** Must be called before any other non-const kernel functions.
pub fn init() {
    Arch::init_static_memory();

    // allow flash read/exec
    Arch::enable_memory_region(
        0,
        Config {
            addr: CONF.memory.flash.start_address as *const _,
            memory: Type::Flash,
            size: CONF.memory.flash.size,
            access: Access { user: Permission::ReadOnly, system: Permission::ReadOnly },
            executable: true
        });


    // allow peripheral RW
    Arch::enable_memory_region(
        1,
        Config {
            addr: CONF.memory.peripheral.start_address as *const _,
            memory: Type::Peripheral,
            size: CONF.memory.peripheral.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    //allow .shared section RW access
    let shared = Arch::region();
    Arch::enable_memory_region(
        2,
        Config {
            addr: shared.start,
            memory: Type::SramInternal,
            size: CONF.memory.shared.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    Arch::disable_memory_region(3);
    Arch::disable_memory_region(4);

    let core = ArchCore::new();

    // Init static pools, this is unsafe but stable for now. Temporary solution
    // until const fn works with MaybeUninit.
    unsafe {
        let mut task_pool: [Option<Node<Task>>; CONF.task.pool_size] =
            MaybeUninit::uninit().assume_init();
        for element in task_pool.iter_mut() {
            *element = None;
        }
        TASK_POOL = MaybeUninit::new(ArrayPool::new(task_pool));

        let mut event_pool: [Option<Node<Event>>; CONF.event.pool_size] =
            MaybeUninit::uninit().assume_init();
        for element in event_pool.iter_mut() {
            *element = None;
        }
        EVENT_POOL = MaybeUninit::new(ArrayPool::new(event_pool));

        let mut tasks_ready: [LinkedList<Task, TaskPool>; CONF.task.priorities as usize] =
            MaybeUninit::uninit().assume_init();
        for element in tasks_ready.iter_mut() {
            *element = LinkedList::new(&*TASK_POOL.as_mut_ptr());
        }


        SCHEDULER = MaybeUninit::new(Scheduler {
            core,
            task_running: None,
            tasks_ready: tasks_ready,
            tasks_sleeping: LinkedList::new(&*TASK_POOL.as_mut_ptr()),
            tasks_terminated: LinkedList::new(&*TASK_POOL.as_mut_ptr()),
            events: LinkedList::new(&*EVENT_POOL.as_mut_ptr()),
            event_counter: 0,
        });
    }
}

/// Set the kernel tick frequency.
///
/// **Note:** Must at least be called once before you start the scheduler.
pub fn set_tick_frequency(tick_frequency: u32, clock_frequency: u32) {
    // NOTE(unsafe): scheduler must be initialized first
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    let divisor = clock_frequency / tick_frequency;
    sched.core.set_systick_div(divisor);
}

/// Start the scheduler.
///
/// Will never return.
pub fn start() -> ! {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    // ensure an idle task is present
    if sched.tasks_ready[(CONF.task.priorities as usize) -1].len() == 0 {
        Task::new()
            .idle_task()
            .static_stack(crate::alloc_static_stack!(128))
            .spawn(move || default_idle());
    }

    let mut task = None;
    for list in sched.tasks_ready.iter_mut() {
        if list.len() > 0 {
            task = list.pop_front();
            break;
        }
    }

    Arch::apply_regions((*task.as_ref().unwrap()).memory_regions());
    Arch::enable_memory_protection();
    sched.task_running = task;
    sched.core.start();

    let stack_ptr = (*sched.task_running.as_ref().unwrap()).stack_ptr();
    Arch::start_first_task(stack_ptr);
}

/// Add a task to the scheduler.
pub(crate) fn add(mut task: Task) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable

    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        unsafe {
            let stack_ptr = Arch::init_task_stack(
                task.stack_ptr(),
                task::entry as *const usize,
                task.runnable_ptr(),
                syscall::task_exit as *const usize
            );
            task.set_stack_ptr(stack_ptr);
        }

        let prio: usize = task.priority().into();
        sched.tasks_ready[prio].emplace_back(task).ok();
    });
}

/// Put the running task to sleep.
pub(crate) fn sleep(ms: u32) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let task = &mut *sched.task_running.as_mut().unwrap();
        task.sleep(ms);
        task.set_transition(Transition::Sleeping);
    });
    Arch::trigger_context_switch();
}

/// Yield the CPU.
pub(crate) fn yield_now() {
    Arch::trigger_context_switch();
}

/// Exit the running task.
pub(crate) fn task_terminate() {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let task = &mut *sched.task_running.as_mut().unwrap();
        task.set_transition(Transition::Terminating);
    });
    Arch::trigger_context_switch();
}

/// Tick occurred, update sleeping list
pub(crate) fn tick_update() {
    let now = time::tick();

    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut trigger_switch = false;
    critical_section::exec(|| {
        // update pending -> ready list
        let preempt_prio = match sched.task_running.as_ref() {
            Some(task) => (*task).priority().into(),
            None => usize::MAX,
        };

        let mut cursor = sched.tasks_sleeping.cursor_front_mut();
        while let Some(task) = cursor.inner() {
            if task.next_wut() <= now as u64 {
                // todo: this is inefficient, we know that node exists
                if let Some(node) = cursor.take() {
                    let prio: usize = (*node).priority().into();
                    sched.tasks_ready[prio].push_back(node);
                    if prio < preempt_prio {
                        trigger_switch = true;
                    }
                }
            } else {
                break; // the list is sorted by wake-up time, we can abort early
            }
            cursor.move_next();
        }

        #[cfg(feature = "time-slicing")]
        if sched.tasks_ready[preempt_prio].len() > 0 {
            trigger_switch = true;
        }
    });

    if trigger_switch {
        Arch::trigger_context_switch();
    }
}

fn default_idle() {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub(crate) fn event_register() -> Result<usize, pool_allocator::Error> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let id = sched.event_counter + 1;
        sched.event_counter = id;
        let result = sched.events.emplace_back(Event::new(id));
        result.map(|_| id)
    })
}

pub(crate) fn event_await(id: usize, _timeout: usize) -> Result<(), event::Error> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let event = match sched.events.iter_mut().find(|e|
            e.id() == id
        ) {
            Some(e) => unsafe { NonNull::new_unchecked(e) },
            None => {
                return Err(event::Error::InvalidId);
            }
        };

        let task = sched.task_running.as_mut().unwrap();
        (*task).set_blocking_event(event);
        (*task).set_transition(Transition::Blocked);
        return Ok(());
    }).map(|_| Arch::trigger_context_switch())
    // todo: returning ok will not work, because the result will be returned to the wrong task
}

pub(crate) fn event_fire(id: usize) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut switch = false;

    critical_section::exec(|| {
        if let Some(e) = sched.events.iter_mut().find(|e| e.id() == id) {
            if let Some(t) = e.pending.pop_front() {
                let prio: usize = (*t).priority().into();
                sched.tasks_ready[prio].push_back(t);
            }
            switch = true;
        }
    });

    if switch {
        Arch::trigger_context_switch();
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Switch context from current to next task.
///
/// The function takes the current stack pointer of the running task and will
/// return the stack pointer of the next task.
///
/// **Note:** This function must be called from the architecture specific task
/// switch implementation.
#[no_mangle]
fn switch_context(stack_ptr: u32) -> u32 {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    Arch::disable_memory_protection();
    let new_stack_ptr = critical_section::exec(|| {
        (*sched.task_running.as_mut().unwrap()).set_stack_ptr(stack_ptr as *mut usize);

        // Put the running task into its next state
        let mut pausing = sched.task_running.take().unwrap();
        let prio: usize = (*pausing).priority().into();
        match (*pausing).transition() {
            Transition::None => sched.tasks_ready[prio].push_back(pausing),
            Transition::Sleeping => {
                (*pausing).set_transition(Transition::None);
                sched.tasks_sleeping.insert_when(
                    pausing,
                    |pausing, task| {
                        pausing.next_wut() < task.next_wut()
                    });
            },
            Transition::Blocked => {
                let event = (*pausing).blocking_event().unwrap(); // cannot be none
                (*pausing).set_transition(Transition::None);
                unsafe { &mut *event.as_ptr() }.pending.insert_when(
                    pausing,
                    |pausing, task| {
                        pausing.priority().0 < task.priority().0
                    });
            }
            Transition::Terminating => {
                (*pausing).set_transition(Transition::None);
                sched.tasks_terminated.push_back(pausing);
            },
            _ => (),
        }

        // Load the next task
        let mut task = None;
        for list in sched.tasks_ready.iter_mut() {
            if list.len() > 0 {
                task = list.pop_front();
                break;
            }
        }
        if task.is_none() {
            panic!("Idle task must not be suspended");
        }

        Arch::apply_regions((*task.as_ref().unwrap()).memory_regions());
        sched.task_running = task;
        let stack_ptr = (*sched.task_running.as_ref().unwrap()).stack_ptr();
        stack_ptr as u32
    });

    Arch::enable_memory_protection();
    new_stack_ptr
}

#[repr(usize)]
enum StackSpace {
    Sufficient = 1,
    Insufficient = 0,
}
/// Check if the given stack pointer is within the stack range of the running
/// task.
#[no_mangle]
fn check_stack(stack_ptr: usize) -> StackSpace {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let stack = (*sched.task_running.as_ref().unwrap()).stack();
    if stack_ptr > (stack.bottom_ptr() as usize) {
        StackSpace::Sufficient
    } else {
        StackSpace::Insufficient
    }
}

/// Exception if a memory protection rule was violated.
#[no_mangle]
fn memory_protection_exception() {
    task_terminate();
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn init() {
        Arch::disable_interrupts_context().expect().return_once(|priority| {
            assert_eq!(priority, usize::MAX);
        });
        Arch::enable_interrupts_context().expect().returning(|| {});

        let core_ctx = ArchCore::new_context();
        core_ctx.expect()
            .returning(|| {
                ArchCore::default()
            });

        super::init();

        let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

        critical_section::exec(|| {
            assert_eq!(sched.task_running.is_none(), true);
            assert_eq!(sched.tasks_terminated.len(), 0);
        });
    }
}