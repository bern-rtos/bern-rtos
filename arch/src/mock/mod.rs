//! Mock all architecture interfaces.

#![allow(unreachable_code)]

use mockall::*;
use mockall::predicate::*;

use crate::memory_protection::{IMemoryProtection, Access, Config, Type, Permission};
use crate::scheduler::IScheduler;
use crate::syscall::ISyscall;
use crate::core::ICore;
use crate::sync::ISync;
use crate::startup::{IStartup, Region};

// re-exports
pub use crate::mock::MockArch as Arch;
pub use crate::mock::MockArchCore as ArchCore;

mockall::mock!{
    pub Arch {}

    impl IMemoryProtection for Arch {
        type Size = u32;
        type MemoryRegion = u32;

        fn enable_memory_protection();
        fn disable_memory_protection();
        fn enable_memory_region(region: u8, config: Config<Self::Size>);
        fn disable_memory_region(region: u8);
        fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion;
        fn prepare_unused_region(region: u8) -> Self::MemoryRegion;
        fn apply_regions(memory_regions: &[Self::MemoryRegion; 3]);
    }

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
        fn init_static_region(region: Region);
        fn kernel_data() -> Region;
        fn kernel_heap() -> Region;
    }
}

mockall::mock!{
    pub ArchCore {}

    impl ICore for ArchCore {
        fn new() -> Self;
        fn set_systick_div(&mut self, divisor: u32);
        fn start(&mut self);
        fn bkpt();
    }
}