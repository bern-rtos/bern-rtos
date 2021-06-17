//! Types and structures for the kernel config.
//!
//! Cargo features are somewhat limited and configuring the size of static
//! arrays is straight forward. The types for the config are put into separate
//! crate to ensure non-cyclic dependencies as the `conf` and `bern_kernel`
//! crate depend on it.
#![no_std]

pub use bern_arch::arch::memory_protection::Size;

/// Task related config
pub struct Task {
    /// The number of tasks that can be allocated in the static pool
    pub pool_size: usize,
    /// The number of different priorities.
    /// Keep the number low as it influences overhead when switching tasks
    pub priorities: u8,
}

/// Event related config
pub struct Event {
    /// The number of events that can be allocated.
    /// An event is used for every mutex, semaphore, flag group and message
    /// queue.
    pub pool_size: usize,
}

/// Definition of a memory section or region
pub struct MemorySection {
    /// Lowest address of the section
    pub start_address: usize,
    /// Section size
    pub size: Size,
}

/// Memory sections
///
/// **Note:** This overlaps with the linker script and must be adjusted
/// manually.
pub struct Memory {
    /// Flash (non-volatile) memory section
    pub flash: MemorySection,
    /// SRAM (volatile) memory section
    pub sram: MemorySection,
    /// Memory mapped peripherals section
    pub peripheral: MemorySection,
    /// Shared memory section
    pub shared: MemorySection,
}

/// Combined config
pub struct Conf {
    pub task: Task,
    pub event: Event,
    pub memory: Memory,
}