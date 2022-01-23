//! Scheduler.

/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.

pub(crate) mod event;

use core::sync::atomic::{self, Ordering};
use core::mem::MaybeUninit;
use core::ptr::NonNull;

use event::Event;
use crate::exec::task::{self, Task, Transition};
use crate::syscall;
use crate::time;
use crate::sync::critical_section;
use crate::mem::{boxed::Box, linked_list::*, Size};
use crate::alloc::allocator::AllocError;
use crate::alloc::bump::Bump;
use crate::kernel::static_memory;

use bern_arch::{ICore, IMemoryProtection, IScheduler, IStartup};
use bern_arch::arch::{Arch, ArchCore};
use bern_arch::memory_protection::{Access, Config, Permission, Type};
use bern_conf::CONF;
use crate::exec::interrupt::Interrupt;


// These statics are MaybeUninit because, there currently no solution to
// initialize an array dependent on a `const` size and a non-copy type.
// These attempts didn't work:
// - const fn with internal MaybeUninit: not stable yet
// - proc_macro: cannot evaluate const

#[link_section = ".kernel"]
static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();
#[link_section = ".kernel"]
static mut KERNEL_ALLOCATOR: MaybeUninit<Bump> = MaybeUninit::uninit();

// todo: split scheduler into kernel and scheduler
struct Scheduler {
    core: ArchCore,
    task_running: Option<Box<Node<Task>>>,
    tasks_ready: [LinkedList<Task>; CONF.task.priorities as usize],
    tasks_sleeping: LinkedList<Task>,
    tasks_terminated: LinkedList<Task>,
    interrupt_handlers: LinkedList<Interrupt>,
    events: LinkedList<Event>,
    event_counter: usize,
}

/// Initialize scheduler.
///
/// **Note:** Must be called before any other non-const kernel functions.
pub fn init() {
    Arch::init_static_region(static_memory::kernel_data());

    // Memory regions 0..2 are reserved for tasks
    Arch::disable_memory_region(0);
    Arch::disable_memory_region(1);
    Arch::disable_memory_region(2);

    // Allow flash read/exec
    Arch::enable_memory_region(
        3,
        Config {
            addr: CONF.memory.flash.start_address as *const _,
            memory: Type::Flash,
            size: CONF.memory.flash.size,
            access: Access { user: Permission::ReadOnly, system: Permission::ReadOnly },
            executable: true
        });

    // Allow peripheral RW
    Arch::enable_memory_region(
        4,
        Config {
            addr: CONF.memory.peripheral.start_address as *const _,
            memory: Type::Peripheral,
            size: CONF.memory.peripheral.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    // Allow .data & .bss read/write
    Arch::enable_memory_region(
        5,
        Config {
            addr: CONF.memory.sram.start_address as *const _,
            memory: Type::SramInternal,
            size: Size::S4K, // todo: read from linker symbol or config
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    Arch::disable_memory_region(6);
    Arch::disable_memory_region(7);

    let core = ArchCore::new();

    unsafe {
        KERNEL_ALLOCATOR = MaybeUninit::new(
            Bump::new(
                NonNull::new_unchecked(static_memory::kernel_heap().start as *mut u8),
                NonNull::new_unchecked(static_memory::kernel_heap().end as *mut u8)));
    }

    // Init static pools, this is unsafe but stable for now. Temporary solution
    // until const fn works with MaybeUninit.
    unsafe {
        let mut tasks_ready: [LinkedList<Task>; CONF.task.priorities as usize] =
            MaybeUninit::uninit().assume_init();
        for element in tasks_ready.iter_mut() {
            *element = LinkedList::new();
        }


        SCHEDULER = MaybeUninit::new(Scheduler {
            core,
            task_running: None,
            tasks_ready,
            tasks_sleeping: LinkedList::new(),
            tasks_terminated: LinkedList::new(),
            interrupt_handlers: LinkedList::new(),
            events: LinkedList::new(),
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
pub(crate) fn start() -> ! {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    // ensure an idle task is present
    /*if sched.tasks_ready[(CONF.task.priorities as usize) -1].len() == 0 {
        IDLE_PROC.create_thread()
            .idle_task()
            .stack(256)
            .spawn(default_idle);
    }*/

    let mut task = None;
    for list in sched.tasks_ready.iter_mut() {
        if list.len() > 0 {
            task = list.pop_front();
            break;
        }
    }

    Arch::apply_regions((**task.as_ref().unwrap()).memory_regions());
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
        sched.tasks_ready[prio].emplace_back(
            task,
            unsafe { &*KERNEL_ALLOCATOR.as_ptr() }
        ).ok();
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
                    let prio: usize = (**node).priority().into();
                    sched.tasks_ready[prio].push_back(node);
                    if prio < preempt_prio {
                        trigger_switch = true;
                    }
                }
            } else {
                break; // the list is sorted by wake-up time, we can abort early
            }
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

#[allow(unused)]
fn default_idle() {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

pub(crate) fn interrupt_handler_add(interrupt: Interrupt) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        sched.interrupt_handlers.emplace_back(
            interrupt,
            unsafe { &*KERNEL_ALLOCATOR.as_ptr() }
        ).ok();
    });
}

pub(crate) fn event_register() -> Result<usize, AllocError> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let id = sched.event_counter + 1;
        sched.event_counter = id;
        let result = sched.events.emplace_back(
            Event::new(id),
            unsafe { &*KERNEL_ALLOCATOR.as_ptr() }
        );
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

#[no_mangle]
fn kernel_interrupt_handler(irqn: u16) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    defmt::trace!("IRQ {} called.", irqn);
    for handler in sched.interrupt_handlers.iter_mut() {
        if handler.contains_interrupt(irqn) {
            handler.call(irqn);
        }
    }

}

pub(crate) fn with_callee<F, R>(f: F) -> R
    where F: FnOnce(&Task) -> R
{
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let task = &***sched.task_running.as_ref().unwrap();
    f(task)
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
        // Did pausing task break hardware specific boundaries?
        if stack_ptr == 0 {
            pausing.set_transition(Transition::Terminating);
        }
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
    defmt::warn!("Memory exception, terminating thread.");
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