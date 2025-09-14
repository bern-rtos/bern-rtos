//! Types and structures for the kernel config.
//!
//! Cargo features are somewhat limited and configuring the size of static
//! arrays is straight forward. The types for the config are put into separate
//! crate to ensure non-cyclic dependencies as the `conf` and `bern_kernel`
//! crate depend on it.
#![no_std]

use bern_units::memory_size::Byte;

/// Location of additional memory.
pub enum MemoryLocation {
    /// Located within the microcontroller.
    Internal,
    /// Memory is connected to the microcontroller via an external bus.
    External,
}

/// Type of Memory.
pub enum MemoryType {
    /// Read-only memory.
    Rom,
    /// EERPOM.
    ///
    /// Bytes can be set/cleared individually.
    Eeprom,
    /// Flash memory.
    ///
    /// Clearing memory requires clearing whole pages.
    Flash,
    /// RAM for non-volatile data storage.
    Ram,
    /// Memory mapped peripheral.
    Peripheral,
}

/// Kernel related config
pub struct Kernel {
    /// The number of different priorities.
    /// Keep the number low as it influences overhead when switching threads.
    pub priorities: u8,
    /// Size of kernel memory region.
    ///
    /// **Note:** Must be sized accoring to memory protection restrictions.
    pub memory_size: Byte,
}

pub struct Shared {
    /// Size of the shared memory region (bss + data).
    ///
    /// **Note:** Must be sized accoring to memory protection restrictions.
    pub size: Byte,
}

/// Definition of a memory section.
pub struct Memory {
    /// Name in the linker file.
    pub link_name: &'static str,
    /// Lowest address of the section.
    pub start_address: usize,
    /// Memory size.
    pub size: Byte,
}

/// Definition of optional memory sections that go beyond the default of
/// SRAM and flash.
pub struct OptionalMemory {
    /// Type of memory.
    pub memory_type: MemoryType,
    /// Location of the memory.
    pub location: MemoryLocation,
    /// Name in the linker file.
    pub link_name: &'static str,
    /// Lowest address of the section.
    pub start_address: usize,
    /// Memory size.
    pub size: Byte,
}

/// Memory map.
pub struct MemoryMap<const N: usize> {
    /// Default internal flash memory.
    pub flash: Memory,
    /// Default internal SRAM.
    pub sram: Memory,
    /// Memory mapped peripheral address range.
    pub peripheral: Memory,
    /// Additional memory components such as
    /// - Tightly coupled memory
    /// - External RAM
    /// - External flash
    pub additional: [OptionalMemory; N],
}

/// Placement of static data and allocators.
///
/// Provide the name of the memory section used in the linker script.
pub struct DataPlacement {
    /// Static kernel data and kernel allocator.
    ///
    /// Inaccessible to the user.
    pub kernel: &'static str,
    /// Static process data and allocator.
    pub processes: &'static str,
    /// Shared memory section.
    ///
    /// Typically placed in `RAM`.
    pub shared: &'static str,
}

/// Combined configuration.
pub struct Conf<const N: usize> {
    /// Kernel configuration.
    pub kernel: Kernel,
    /// Shared memory configuration.
    pub shared: Shared,
    /// Application memory map.
    pub memory_map: MemoryMap<N>,
    /// Placement of static data and allocators.
    pub data_placement: DataPlacement,
}
