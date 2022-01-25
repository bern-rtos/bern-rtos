use core::ptr::NonNull;
use bern_arch::arch::Arch;
use bern_arch::arch::memory_protection::{MemoryRegion, Size};
use bern_arch::IMemoryProtection;
use bern_arch::memory_protection::{Access, Config, Permission, Type};
use crate::exec::process::Process;
use crate::mem::boxed::Box;
use crate::sched::event::Event;
use crate::stack::Stack;
use crate::time;


pub trait RunnableTrait: 'static + FnMut() -> RunnableResult {}
pub type RunnableResult = (); // todo: replace with '!' when possible


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


/// Task priority.
///
/// 0 is the highest priority.
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Priority(pub u8);
// todo: check priority range at compile time

impl Priority {
    pub fn is_interrupt_handler(self) -> bool {
        self.0 == 0
    }
}

impl Into<usize> for Priority {
    fn into(self) -> usize {
        self.0 as usize
    }
}


// todo: manage lifetime of stack & runnable
/// Task control block
pub struct Runnable {
    process: &'static Process,
    transition: Transition,
    runnable_ptr: *mut usize,
    next_wut: u64,
    stack: Stack,
    priority: Priority,
    blocking_event: Option<NonNull<Event>>,
    memory_regions: [MemoryRegion; 3],
}

impl Runnable {
    pub(crate) fn new(process: &'static Process, runnable_ptr: *mut usize, stack: Stack, priority: Priority) -> Self {
        // prepare memory region configs
        let memory_regions = [
            Arch::prepare_memory_region(
                0,
                Config {
                    addr: process.start_addr() as *const _,
                    memory: Type::SramInternal,
                    size: process.size(),
                    access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
                    executable: false
                }),
            Arch::prepare_memory_region(
                1,
                Config {
                    addr: stack.bottom_ptr() as *const _,
                    memory: Type::SramInternal,
                    size: Size::S32,
                    access: Access { user: Permission::NoAccess, system: Permission::NoAccess },
                    executable: false
                }),
            Arch::prepare_unused_region(2)
        ];

        Runnable {
            process,
            transition: Transition::None,
            runnable_ptr,
            next_wut: 0,
            stack,
            priority,
            blocking_event: None,
            memory_regions,
        }
    }

    pub(crate) fn runnable_ptr(&self) -> *const usize {
        self.runnable_ptr
    }

    pub(crate) fn stack(&self) -> &Stack {
        &self.stack
    }
    pub(crate) fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
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

    pub(crate) fn process(&self) -> &Process {
        self.process
    }
}

/// Static and non-generic entry point of the task.
///
/// This function simply starts the closure stored on the task stack. It will
/// only be called when the task runs for the first time.
///
/// **Note:** Don't be fooled by the `&mut &mut` the first one is a reference
/// and second one is part of the trait object type
pub(crate) fn entry(entry_fn: &mut &mut (dyn FnMut() -> RunnableResult)) {
    (entry_fn)();
}
