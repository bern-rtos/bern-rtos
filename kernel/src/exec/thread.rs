//! Task creation and control.
//!
//! # Example
//! Create a task using the builder pattern:
//! ```no_run
//! let mut heartbeat = board.shield.led_1;
//! Task::new()
//!     .priority(Priority(0))
//!     .static_stack(kernel::alloc_static_stack!(512))
//!     .spawn(move || {
//!         loop {
//!             kernel::sleep(200);
//!             heartbeat.toggle().ok();
//!         }
//!     });
//! ```
//! The task builder ([`TaskBuilder`]) is used to configure a task. On
//! [`TaskBuilder::spawn()`] the information is passed to the scheduler and the
//! task is put into ready state.
//!
//! Tasks can be spawened from `main()` or within other tasks.

#![allow(unused)]

use core::alloc::Layout;
use core::mem;
use core::ptr;
use core::ops::Deref;

use crate::sched;
use crate::syscall;
use crate::time;
use crate::stack::Stack;
use crate::sched::event::Event;
use core::ptr::NonNull;
use bern_arch::arch::memory_protection::{MemoryRegion, Size};
use bern_arch::arch::Arch;
use bern_arch::memory_protection::{Config, Type, Access, Permission};
use bern_arch::IMemoryProtection;
use bern_conf::CONF;
use crate::alloc::allocator::AllocError;
use crate::exec::process;
use crate::exec::process::Process;
use crate::exec::runnable::{Priority, RunnableResult, Runnable, Transition};
use crate::mem::boxed::Box;

pub struct Thread {}

impl Thread {
    /// Create a new task using the [`TaskBuilder`]
    pub fn new(context: &process::Context) -> ThreadBuilder {
        ThreadBuilder {
            process: context.process(),
            stack: None,
            // set default to lowest priority above idle
            priority: Default::default(),
        }
    }
}

/// Builder to create a new task
pub struct ThreadBuilder {
    /// Parent process
    process: &'static Process,
    /// Task stack
    stack: Option<Stack>,
    /// Task priority
    priority: Priority,
}

impl ThreadBuilder {
    /// Set stack size.
    pub fn stack(&mut self, stack: Stack) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    /// Set task priority.
    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    /// This task will replace the default idle task.
    pub fn idle_task(&mut self) -> &mut Self {
        self.priority = Priority::idle();
        self
    }

    // todo: return result
    /// Spawns the task and takes the entry point as closure.
    ///
    /// **Note:** A task cannot access another tasks stack, thus all stack
    /// initialization must be handled via syscalls.
    pub fn spawn<F>(&mut self, entry: F)
        where F: 'static + FnMut() -> RunnableResult
    {
        let mut boxed_entry = match Box::try_new_in(entry, self.process.allocator()) {
            Ok(b) => b,
            Err(_) => { panic!("todo: allocate stack"); }
        };

        let entry_ptr = unsafe { &mut *Box::leak(boxed_entry).as_ptr() };
        syscall::thread_spawn(
            self,
            &(entry_ptr as &mut dyn FnMut() -> RunnableResult)
        );
    }

    // userland barrier ////////////////////////////////////////////////////////
    pub(crate) fn build(&mut self, entry: &&mut (dyn FnMut() -> RunnableResult)) {
        let mut stack = match self.stack.take() {
            Some(stack) => stack,
            None => panic!("todo: return error"),
        };
        let mut ptr = stack.ptr() as *mut u8;

        // copy runnable trait object to stack
        let entry_len = mem::size_of_val(entry);
        unsafe {
            ptr = Self::align_ptr(ptr, 8);
            ptr = ptr.offset(-(entry_len as isize));
            ptr::write(ptr as *mut _, entry.deref());
        }
        let runnable_ptr = ptr as *mut usize;

        // align top of stack
        unsafe { ptr = Self::align_ptr(ptr, 8); }
        stack.set_ptr(ptr as *mut usize);

        let mut thread = Runnable::new(
            self.process,
            runnable_ptr,
            stack,
            self.priority,
        );
        sched::add_task(thread)
    }

    unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
        let offset = ptr as usize % align;
        ptr.offset(-(offset as isize))
    }
}




#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use bern_arch::arch::Arch;

}