//! System calls.
//!
//! # Example
//!
//! Launch a task that wait for 100 iterations and the terminates.
//! ```no_run
//! fn main() -> ! {
//!     /*...*/
//!     sched::init();
//!
//!     Task::new()
//!         .static_stack(kernel::alloc_static_stack!(512))
//!         .spawn(move || {
//!             for a in 0..100 {
//!                 bern_kernel::sleep(100);
//!             }
//!             bern_kernel::task_exit();
//!         });
//! }
//! ```

use core::mem;

use crate::sched;
use crate::sched::event;
use crate::task::{RunnableResult, TaskBuilder};

use bern_arch::ISyscall;
use bern_arch::arch::Arch;


// todo: create with proc macro

#[repr(u8)]
enum Service {
    MoveClosureToStack,
    TaskSpawn,
    TaskSleep,
    TaskYield,
    TaskExit,
    EventRegister,
    EventAwait,
    EventFire,
}

impl Service {
    /// Get syscall service id
    pub const fn service_id(self) -> u8 {
        self as u8
    }
}

/// Move the closure for the task entry point to the task stack.
///
/// This will copy the `closure` to stack point store in the `builder`c
pub(crate) fn move_closure_to_stack<F>(closure: F, builder: &mut TaskBuilder)
    where F: 'static + FnMut() -> RunnableResult
{
    Arch::syscall(
        Service::MoveClosureToStack.service_id(),
        &closure as *const _ as usize,
        mem::size_of::<F>() as usize,
        builder as *mut _ as usize
    );
}

/// Add a task to the scheduler based on its `TaskBuilder` and entry point.
pub(crate) fn task_spawn(builder: &mut TaskBuilder, runnable: &&mut (dyn FnMut() -> RunnableResult)) {
    Arch::syscall(
        Service::TaskSpawn.service_id(),
        builder as *mut _ as usize,
        runnable as *const _ as usize,
        0
    );
}


/// Put the current task to sleep for `ms` milliseconds.
pub fn sleep(ms: u32) {
    Arch::syscall(
        Service::TaskSleep.service_id(),
        ms as usize,
        0,
        0
    );
}

/// Yield the CPU.
///
/// **Note:** If the calling task is the only ready task of its priority it will
/// be put to running state again.
pub fn yield_now() {
    Arch::syscall(
        Service::TaskYield.service_id(),
        0,
        0,
        0
    );
}

/// Terminate the current task voluntarily.
pub fn task_exit() {
    Arch::syscall(
        Service::TaskExit.service_id(),
        0,
        0,
        0
    );
}

/// Allocate and request the ID a new event.
///
/// The ID is later used to await and fire events.
pub(crate) fn event_register() -> usize {
    Arch::syscall(
        Service::EventRegister.service_id(),
        0,
        0,
        0
    )
}

/// Wait until an event or a timeout occurs.
pub(crate) fn event_await(id: usize, timeout: u32) -> Result<(), event::Error> {
    let ret_code = Arch::syscall(
        Service::EventAwait.service_id(),
        id,
        timeout as usize,
        0
    ) as u8;
    unsafe { mem::transmute(ret_code) }
}

/// Trigger an event given its ID.
pub(crate) fn event_fire(id: usize) {
    Arch::syscall(
        Service::EventFire.service_id(),
        id,
        0,
        0
    );
}


// userland barrier ////////////////////////////////////////////////////////////

/// System Call handler.
///
/// **Note:** The syscall above will trigger hardware specific system call
/// handler which **must** call this function.
// todo: return result
#[allow(unused_variables)]
#[no_mangle]
fn syscall_handler(service: Service, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match service {
        Service::MoveClosureToStack => {
            let builder: &mut TaskBuilder = unsafe { mem::transmute(arg2 as *mut TaskBuilder) };
            TaskBuilder::move_closure_to_stack(
                builder,
                arg0 as *const u8,
                arg1
            );
            0
        },
        Service::TaskSpawn => {
            let builder: &mut TaskBuilder = unsafe { mem::transmute(arg0 as *mut TaskBuilder) };
            let runnable: &&mut (dyn FnMut() -> RunnableResult) = unsafe {
                mem::transmute(arg1 as *mut &mut (dyn FnMut() -> RunnableResult))
            };
            TaskBuilder::build(
                builder,
                runnable,
            );
            0
        },
        Service::TaskSleep => {
            let ms: u32 = arg0 as u32;
            sched::sleep(ms);
            0
        },
        Service::TaskYield => {
            sched::yield_now();
            0
        },
        Service::TaskExit => {
            sched::task_terminate();
            0
        },

        Service::EventRegister => {
            match sched::event_register() {
                Ok(id) => id,
                Err(_) => 0,
            }
        },
        Service::EventAwait => {
            let id = arg0;
            let timeout = arg1;
            let result = sched::event_await(id, timeout);
            let result: Result<(), event::Error> = Ok(());
            let ret_code: u8 = unsafe { mem::transmute(result) };
            ret_code as usize
        },
        Service::EventFire => {
            let id = arg0;
            sched::event_fire(id);
            0
        },
    }
}