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

/// Transition for next context switch
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Transition {
    /// No transition
    None,
    /// Task is going into sleep mode
    Sleeping,
    /// Task es beeing blocked and waiting for an event
    Blocked,
    /// Resume suspended task
    Resuming,
    /// Terminate task
    Terminating,
}


/// # Issue with closures and static tasks
///
/// Every closure has its own anonymous type. A closure can only be stored in a
/// generic struct. The task object stored in the task "list" (array) must all
/// have the same size -> not generic. Thus, the closure can only be referenced
/// as trait object. But need to force the closure to be static, so our
/// reference can be as well. A static closure is not possible, as every static
/// needs a specified type.
/// To overcome the issue of storing a closure into a static task we need to
/// **copy** it into a static stack. Access to the closure is provided via a
/// closure trait object, which now references a static object which cannot go
/// out of scope.

pub type RunnableResult = (); // todo: replace with '!' when possible

/// Task priority.
///
/// 0 is the highest priority.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Priority(pub u8);
// todo: check priority range at compile time

impl Into<usize> for Priority {
    fn into(self) -> usize {
        self.0 as usize
    }
}

/// Builder to create a new task
pub struct TaskBuilder {
    /// Task stack
    stack: Option<Stack>,
    /// Task priority
    priority: Priority,
}

impl TaskBuilder {
    /// Add a static stack to the task.
    pub fn static_stack(&mut self, stack: Stack) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    /// Set task priority.
    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        if priority.0 >= CONF.task.priorities as u8 {
            panic!("Priority out of range. todo: check at compile time");
        } else if priority.0 == CONF.task.priorities as u8 - 1  {
            panic!("Priority reserved for idle task. Use `is_idle_task()` instead. todo: check at compile time")
        }
        self.priority = priority;
        self
    }

    /// This task will replace the default idle task.
    pub fn idle_task(&mut self) -> &mut Self {
        self.priority = Priority(CONF.task.priorities as u8 - 1);
        self
    }

    // todo: return result
    /// Spawns the task and takes the entry point as closure.
    ///
    /// **Note:** A task cannot access another tasks stack, thus all stack
    /// initialization must be handled via syscalls.
    pub fn spawn<F>(&mut self, closure: F)
        where F: 'static + FnMut() -> RunnableResult
    {
        syscall::move_closure_to_stack(closure, self);
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: allocate stack"),
        };

        // create trait object pointing to closure on stack
        let mut runnable: &mut (dyn FnMut() -> RunnableResult);
        unsafe {
            runnable = &mut *(stack.ptr as *mut F);
        }

        syscall::task_spawn(self, &runnable);
    }

    // userland barrier ////////////////////////////////////////////////////////

    pub(crate) fn move_closure_to_stack(&mut self, closure: *const u8, size_bytes: usize) {
        // todo: check stack size otherwise this is pretty insecure
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: return error"),
        };

        unsafe {
            let mut ptr = stack.ptr as *mut u8;
            ptr = ptr.offset(-(size_bytes as isize));
            ptr::copy_nonoverlapping(closure, ptr, size_bytes);
            stack.ptr = ptr as *mut usize;
        }
    }

    pub(crate) fn build(&mut self, runnable: &&mut (dyn FnMut() -> RunnableResult)) {
        // todo: check stack size otherwise this is pretty insecure
        let mut stack = match self.stack.as_mut() {
            Some(stack) => stack,
            None => panic!("todo: return error"),
        };
        let mut ptr = stack.ptr as *mut u8;

        // copy runnable trait object to stack
        let runnable_len = mem::size_of_val(runnable);
        unsafe {
            ptr = Self::align_ptr(ptr, 8);
            ptr = ptr.offset(-(runnable_len as isize));
            ptr::write(ptr as *mut _, runnable.deref());
        }
        let runnable_ptr = ptr as *mut usize;

        // align top of stack
        unsafe { ptr = Self::align_ptr(ptr, 8); }
        stack.ptr = ptr as *mut usize;

        // prepare memory region configs
        let memory_regions = [Arch::prepare_memory_region(
            5,
            Config {
                addr: stack.bottom_ptr() as *const _,
                memory: Type::SramInternal,
                size: stack.size(),
                access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
                executable: false
            }),
            Arch::prepare_unused_region(6),
            Arch::prepare_unused_region(7)
        ];

        let mut task = Task {
            transition: Transition::None,
            runnable_ptr,
            next_wut: 0,
            stack: self.stack.take().unwrap(),
            priority: self.priority,
            blocking_event: None,
            memory_regions,
        };
        sched::add(task)
    }

    unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
        let offset = ptr as usize % align;
        ptr.offset(-(offset as isize))
    }
}


// todo: manage lifetime of stack & runnable
/// Task control block
pub struct Task {
    transition: Transition,
    runnable_ptr: *mut usize,
    next_wut: u64,
    stack: Stack,
    priority: Priority,
    blocking_event: Option<NonNull<Event>>,
    memory_regions: [MemoryRegion; 3],
}

impl Task {
    /// Create a new task using the [`TaskBuilder`]
    pub fn new() -> TaskBuilder {
        TaskBuilder {
            stack: None,
            // set default to lowest priority above idle
            priority: Priority(CONF.task.priorities - 2),
        }
    }

    // userland barrier ////////////////////////////////////////////////////////

    pub(crate) fn runnable_ptr(&self) -> *const usize {
        self.runnable_ptr
    }

    pub(crate) fn stack(&self) -> &Stack {
        &self.stack
    }
    pub(crate) fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
    pub(crate) fn stack_ptr(&self) -> *mut usize {
        self.stack.ptr
    }
    pub(crate) fn set_stack_ptr(&mut self, psp: *mut usize) {
        self.stack.ptr = psp;
    }
    pub(crate) fn stack_top(&self) -> *const usize {
        self.stack.bottom_ptr() as *const _
    }

    pub(crate) fn next_wut(&self) -> u64 {
        self.next_wut
    }
    pub(crate) fn sleep(&mut self, ms: u32) {
        self.next_wut = time::tick() + u64::from(ms);
    }

    pub(crate) fn transition(&self) -> &Transition {
        &self.transition
    }
    pub(crate) fn set_transition(&mut self, transition: Transition) {
        self.transition = transition;
    }

    pub(crate) fn priority(&self) -> Priority {
        self.priority
    }

    pub(crate) fn memory_regions(&self) -> &[MemoryRegion; 3] {
        &self.memory_regions
    }

    pub(crate) fn blocking_event(&self) -> Option<NonNull<Event>> {
        self.blocking_event
    }
    pub(crate) fn set_blocking_event(&mut self, event: NonNull<Event>) {
        self.blocking_event = Some(event);
    }
}

/// Static and non-generic entry point of the task.
///
/// This function simply starts the closure stored on the task stack. It will
/// only be called when the task runs for the first time.
///
/// **Note:** Don't be fooled by the `&mut &mut` the first one is a reference
/// and second one is part of the trait object type
pub(crate) fn entry(runnable: &mut &mut (dyn FnMut() -> RunnableResult)) {
    (runnable)();
}


#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use bern_arch::arch::Arch;

}