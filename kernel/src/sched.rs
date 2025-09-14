//! Scheduler.

/// # Basic Concept
/// Keep interrupt latency as short as possible, move work to PendSV.
pub(crate) mod event;
mod idle;

use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::alloc::allocator::AllocError;
use crate::exec::runnable::{self, Priority, Runnable, Transition};
use crate::mem::{boxed::Box, linked_list::*};
use crate::sync::critical_section;
use crate::time;
use crate::{log, syscall, KERNEL};
use event::Event;

use crate::exec::interrupt::InterruptHandler;
use bern_arch::arch::{Arch, ArchCore};
use bern_arch::{ICore, IMemoryProtection, IScheduler};
use bern_conf::CONF;
use rtos_trace::{trace, TaskInfo};

// These statics are MaybeUninit because, there currently no solution to
// initialize an array dependent on a `const` size and a non-copy type.
// These attempts didn't work:
// - const fn with internal MaybeUninit: not stable yet
// - proc_macro: cannot evaluate const

#[link_section = ".kernel"]
static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();

// todo: split scheduler into kernel and scheduler
struct Scheduler {
    core: ArchCore,
    task_running: Option<Box<Node<Runnable>>>,
    tasks_ready: [LinkedList<Runnable>; CONF.kernel.priorities as usize],
    tasks_sleeping: LinkedList<Runnable>,
    tasks_terminated: LinkedList<Runnable>,
    interrupt_handlers: LinkedList<InterruptHandler>,
    events: LinkedList<Event>,
    event_counter: usize,
    task_id_counter: u32,
}

/// Initialize scheduler.
///
/// **Note:** Must be called before any other non-const kernel functions.
pub(crate) fn init() {
    let core = ArchCore::new();

    // Init static pools, this is unsafe but stable for now. Temporary solution
    // until const fn works with MaybeUninit.
    unsafe {
        let mut tasks_ready: [LinkedList<Runnable>; CONF.kernel.priorities as usize] =
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
            // Start with ID 1 because 0 signals an initialized thread ID.
            task_id_counter: 1,
        });
    }

    idle::init();
}

/// Set the kernel tick frequency.
///
/// **Note:** Must at least be called once before you start the scheduler.
pub(crate) fn update_tick_frequency(ticks_per_ms: u32) {
    // NOTE(unsafe): scheduler must be initialized first
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    sched.core.set_systick_div(ticks_per_ms);
}

/// Start the scheduler.
///
/// Will never return.
pub(crate) fn start() -> ! {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

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

    let stack_ptr = (*sched.task_running.as_ref().unwrap()).stack().ptr();
    Arch::start_first_task(stack_ptr);

    // Note: We will never get here but the Arch crate cannot use the never type yet
    loop {}
}

/// Add a task to the scheduler.
pub(crate) fn add_task(mut task: Runnable) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        task.set_id(sched.task_id_counter);
        // Idle is handled separately in tracing mode
        if !task.priority().is_idle() {
            trace::task_new(task.id());
        }
        sched.task_id_counter += 1;

        unsafe {
            let stack_ptr = Arch::init_task_stack(
                task.stack().ptr(),
                runnable::entry as *const usize,
                task.runnable_ptr(),
                syscall::task_exit as *const usize,
            );
            task.stack_mut().set_ptr(stack_ptr);
        }

        let prio: usize = task.priority().into();
        trace_thread_info(&task);

        sched.tasks_ready[prio]
            .emplace_back(task, KERNEL.allocator())
            .ok();
    });
}

/// Put the running task to sleep.
pub(crate) fn sleep(ms: u32) {
    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let task = &mut *sched.task_running.as_mut().unwrap();
        trace::task_ready_end(task.id());
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
        trace::task_ready_end(task.id());
        task.set_transition(Transition::Terminating);
    });
    Arch::trigger_context_switch();
}

/// Tick occurred, update sleeping list
pub(crate) fn tick_update() {
    trace::isr_enter();
    let now = time::tick_count();

    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut trigger_switch = false;
    critical_section::exec(|| {
        // update pending -> ready list
        let preempt_prio = match sched.task_running.as_ref() {
            Some(task) => (*task).priority(),
            None => Priority::MAX,
        };

        let mut cursor = sched.tasks_sleeping.cursor_front_mut();
        while let Some(task) = cursor.inner() {
            if task.next_wut() <= now as u64 {
                trace::task_ready_begin(task.id());

                // todo: this is inefficient, we know that node exists
                if let Some(node) = cursor.take() {
                    let prio: usize = (**node).priority().into();
                    sched.tasks_ready[prio].push_back(node);

                    if prio < preempt_prio.into() {
                        trigger_switch = true;
                    }
                }
            } else {
                break; // the list is sorted by wake-up time, we can abort early
            }
        }

        #[cfg(feature = "time-slicing")]
        if sched.tasks_ready[usize::from(preempt_prio)].len() > 0
            && !preempt_prio.is_interrupt_handler()
        {
            trigger_switch = true;
        }
    });

    if trigger_switch {
        trace::isr_exit_to_scheduler();
        Arch::trigger_context_switch();
    } else {
        trace::isr_exit();
    }
}

pub(crate) fn interrupt_handler_add(interrupt: InterruptHandler) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        sched
            .interrupt_handlers
            .emplace_back(interrupt, KERNEL.allocator())
            .ok();
    });
}

pub(crate) fn event_register() -> Result<usize, AllocError> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let id = sched.event_counter + 1;
        sched.event_counter = id;
        sched
            .events
            .emplace_back(Event::new(id), KERNEL.allocator())
            .map(|_| id)
    })
}

pub(crate) fn event_await(id: usize, _timeout: usize) -> Result<(), event::Error> {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    critical_section::exec(|| {
        let event = match sched.events.iter_mut().find(|e| e.id() == id) {
            Some(e) => unsafe { NonNull::new_unchecked(e) },
            None => {
                return Err(event::Error::InvalidId);
            }
        };

        let task = sched.task_running.as_mut().unwrap();
        (*task).set_blocking_event(event);
        (*task).set_transition(Transition::Blocked);
        return Ok(());
    })
    .map(|_| Arch::trigger_context_switch())
    // todo: returning ok will not work, because the result will be returned to the wrong task
}

pub(crate) fn event_fire(id: usize) {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let mut switch = false;

    critical_section::exec(|| {
        if let Some(e) = sched.events.iter_mut().find(|e| e.id() == id) {
            if let Some(t) = e.pending.pop_front() {
                trace::task_ready_begin(t.id());

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

rtos_trace::global_os_callbacks! {Scheduler}

impl rtos_trace::RtosTraceOSCallbacks for Scheduler {
    fn task_list() {
        let sched = unsafe { SCHEDULER.assume_init_mut() };

        sched.task_running.as_ref().map(|thread| {
            if thread.priority() != Priority::idle() {
                trace_thread_info(thread)
            }
        });

        for (prio, threads) in sched.tasks_ready.iter().enumerate() {
            if prio == Priority::idle().into() {
                break;
            }

            for thread in threads.iter() {
                trace_thread_info(thread);
            }
        }

        for thread in sched.tasks_sleeping.iter() {
            trace_thread_info(thread);
        }

        for event in sched.events.iter() {
            for thread in event.pending.iter() {
                trace_thread_info(thread);
            }
        }

        for thread in sched.tasks_terminated.iter() {
            trace_thread_info(thread);
        }
    }

    fn time() -> u64 {
        time::tick_count() * 1000
    }
}

fn trace_thread_info(thread: &Runnable) {
    let info = TaskInfo {
        name: thread.name(),
        priority: thread.id().into(),
        stack_base: thread.stack().bottom_ptr() as usize,
        stack_size: thread.stack().size() as usize,
    };
    trace::task_send_info(thread.id(), info);
}

#[no_mangle]
fn kernel_interrupt_handler(irqn: u16) {
    trace::isr_enter();

    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    log::trace!("IRQ {} called.", irqn);
    for handler in sched.interrupt_handlers.iter_mut() {
        if handler.contains_interrupt(irqn) {
            handler.call(irqn);
        }
    }

    trace::isr_exit();
}

pub(crate) fn with_callee<F, R>(f: F) -> R
where
    F: FnOnce(&Runnable) -> R,
{
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    let task = &***sched.task_running.as_ref().unwrap();
    f(task)
}

pub(crate) fn print_thread_stats() {
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
    log::info!("Thread stats");
    log::info!("============");
    log::info!("Name    Priority    Stack usage            State");
    log::info!("----    --------    -----------            -----");

    if let Some(running) = sched.task_running.as_ref() {
        log::info!("{}    Running", ***running);
    }

    for ready in sched.tasks_ready.iter() {
        for thread in ready.iter() {
            log::info!("{}    Ready", thread);
        }
    }

    for sleeping in sched.tasks_sleeping.iter() {
        log::info!("{}    Sleeping (wut: {})", sleeping, sleeping.next_wut());
    }

    for event in sched.events.iter() {
        for blocked in event.pending.iter() {
            log::info!("{}    Blocked (id: {})", blocked, event.id());
        }
    }

    for terminated in sched.tasks_terminated.iter() {
        log::info!("{}    Terminated", terminated);
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
    trace::task_exec_end();

    // NOTE(unsafe): scheduler must be initialized first
    // todo: replace with `assume_init_mut()` as soon as stable
    let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };

    Arch::disable_memory_protection();
    let new_stack_ptr = critical_section::exec(|| {
        // Put the running task into its next state
        let mut pausing = sched.task_running.take().unwrap();
        let prio: usize = pausing.priority().into();
        // Did pausing task break hardware specific boundaries?
        if stack_ptr == 0 {
            if *pausing.transition() != Transition::Terminating {
                log::warn!("Stack overflow pervented, terminating thread.");
                pausing.set_transition(Transition::Terminating);
            }
        } else {
            pausing.stack_mut().set_ptr(stack_ptr as *mut usize);
        }

        match pausing.transition() {
            Transition::None => sched.tasks_ready[prio].push_back(pausing),
            Transition::Sleeping => {
                pausing.set_transition(Transition::None);
                sched.tasks_sleeping.insert_when(pausing, |pausing, task| {
                    pausing.next_wut() < task.next_wut()
                });
            }
            Transition::Blocked => {
                let event = pausing.blocking_event().unwrap(); // cannot be none
                pausing.set_transition(Transition::None);
                unsafe { &mut *event.as_ptr() }
                    .pending
                    .insert_when(pausing, |pausing, task| {
                        pausing.priority() < task.priority()
                    });
            }
            Transition::Terminating => {
                pausing.set_transition(Transition::None);
                sched.tasks_terminated.push_back(pausing);
            }
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

        let t = &(*task.as_ref().unwrap());
        Arch::apply_regions(t.memory_regions());

        if t.priority().is_idle() {
            trace::system_idle();
        } else {
            trace::task_exec_begin(t.id());
        }
        sched.task_running = task;
        let stack_ptr = (*sched.task_running.as_ref().unwrap()).stack().ptr();
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
    let stack = (*sched.task_running.as_mut().unwrap()).stack_mut();
    if stack_ptr > (stack.bottom_ptr() as usize) + Into::<usize>::into(Arch::min_region_size()) {
        StackSpace::Sufficient
    } else {
        // Mark stack as full for debug purposes
        stack.set_ptr(stack.bottom_ptr() as *mut _);
        StackSpace::Insufficient
    }
}

/// Exception if a memory protection rule was violated.
#[no_mangle]
fn memory_protection_exception() {
    trace::isr_enter();

    log::warn!("Memory exception, terminating thread.");
    task_terminate();

    trace::isr_exit_to_scheduler();
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    #[test]
    fn empty() {
        // Arch::disable_interrupts_context().expect().return_once(|priority| {
        //     assert_eq!(priority, usize::MAX);
        // });
        // Arch::enable_interrupts_context().expect().returning(|| {});
        //
        // let core_ctx = ArchCore::new_context();
        // core_ctx.expect()
        //     .returning(|| {
        //         ArchCore::default()
        //     });
        //
        // super::init();
        //
        // let sched = unsafe { &mut *SCHEDULER.as_mut_ptr() };
        //
        // critical_section::exec(|| {
        //     assert_eq!(sched.task_running.is_none(), true);
        //     assert_eq!(sched.tasks_terminated.len(), 0);
        // });
    }
}
