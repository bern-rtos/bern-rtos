//! Mock all architecture interfaces.

#![allow(unreachable_code)]

use mockall::*;
use mockall::predicate::*;

use crate::scheduler::IScheduler;
use crate::syscall::ISyscall;
use crate::core::ICore;
use crate::sync::ISync;
use crate::startup::IStartup;

// re-exports
pub use crate::mock::MockArch as Arch;
pub use crate::mock::MockArchCore as ArchCore;

mock!{
    pub Arch {}

    impl IScheduler for Arch {
        unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize;
        fn start_first_task(stack_ptr: *const usize) -> !;
        fn trigger_context_switch();
    }

    impl ISyscall for Arch {
        fn syscall(service: u8, arg0: usize, arg1: usize, arg2: usize) -> usize;
    }

    impl ISync for Arch {
        fn disable_interrupts(priority: usize);
        fn enable_interrupts();
    }

    impl IStartup for Arch {
        fn init_static_memory();
    }
}

mock!{
    pub ArchCore {}

    impl ICore for ArchCore {
        fn new() -> Self;
        fn start(&mut self);
        fn bkpt();
    }
}