//! Memory Protection.

use crate::arch::memory_protection::MemoryRegion;
/// Memory Protection.
///
/// # Implementation
/// In addition to the trait the following features have to be implemented:
///
/// ## Memory Protection Exception
/// A violation of a memory rule will trigger an exception. The exception must
/// call `memory_protection_exception()` to notify the kernel, i.e.:
/// ```no_run
/// extern "Rust" {
///     pub fn memory_protection_exception();
/// }
///
/// #[allow(non_snake_case)]
/// #[exception]
/// fn MemoryManagement() -> () {
///     unsafe {
///         memory_protection_exception();
///     }
/// }
/// ```
///
/// ## Size and Alignment
/// Different memory protection implementation have different requirements in
/// terms of sizes and alignment.
/// A hardware implementation must provide
/// - Alignment structs for all available alignments with naming convention
///   `A<size number><unit prefix (_,K,M,G)>`, i.e.
///   ```no_run
///   #[repr(align(4_096))]
///   pub struct A4K;
///   ```
/// - A macro `alignment_from_size!()` that returns valid alignment for given
///   memory size, i.e.
///   ```no_run
///   #[macro_export]
///   macro_rules! alignment_from_size {
///       (4_096) => { $crate::arch::memory_protection::A4K };
///       ($x:expr) => {
///           compile_error!("Size does not meet alignment requirements from the MPU. \
///           Compatible sizes are: 4KB");
///        };
///   }
///   ```
///
/// - A macro `size_from_raw!()` that returns a valid size type from raw number,
///   i.e.
///   ```no_run
///   #[macro_export]
///   macro_rules! size_from_raw {
///       (4_096) => { $crate::arch::memory_protection::Size::S4K };
///       ($x:expr) => {
///           compile_error!("Size cannot be protected by MPU. \
///           Compatible sizes are: 4KB");
///        };
///   }
///   ```
pub trait IMemoryProtection {
    /// Size of a memory region.
    ///
    /// Size must be an enum adhering to the naming convention
    /// `S<size number><unit prefix (_,K,M,G)>`, i.e.
    /// ```no_run
    /// #[repr(u8)]
    /// pub enum Size {
    ///     S64 = 5,
    ///     S4K = 11,
    ///     S128M = 26,
    /// }
    /// ```
    type Size;
    /// Precalculated memory region configuration.
    type MemoryRegion;

    /// Enable memory protection hardware.
    fn enable_memory_protection();
    /// Disable memory protection hardware.
    fn disable_memory_protection();
    /// Setup and enable one memory region.
    ///
    /// # Example
    /// Protect all flash memory from write access, instruction fetch allowed.
    /// ```no_run
    /// Arch::enable_memory_region(
    ///    0,
    ///    Config {
    ///         addr: 0x0800_0000 as *const _,
    ///         memory: Type::Flash,
    ///         size: Size::S512K,
    ///         access: Access { user: Permission::ReadOnly, system: Permission::ReadOnly },
    ///         executable: true
    /// });
    /// ```
    fn enable_memory_region(region: u8, config: Config<Self::Size>);
    /// Disable one memory region.
    fn disable_memory_region(region: u8);
    /// Compile register values from configuration and store in `MemoryRegion`.
    ///
    /// Same as [`Self::enable_memory_region()`] but return the register configuration
    /// instead of applying it to the actual registers.
    fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion;
    /// Compile register values for an unused memory region.
    fn prepare_unused_region(region: u8) -> Self::MemoryRegion;
    /// Apply 3 precompiled memory regions.
    fn apply_regions(memory_regions: &[MemoryRegion; 3]);
}

/// Access Permission
pub enum Permission {
    /// Access not permitted
    NoAccess,
    /// Can only be read
    ReadOnly,
    /// Full access, can be read and written
    ReadWrite,
}

/// Access configuration
pub struct Access {
    /// Permission in user mode (i.e. tasks)
    pub user: Permission,
    /// Permission in system mode (i.e. ISR, kernel)
    pub system: Permission,
}

/// Type of memory
pub enum Type {
    /// SRAM in the microcontroller
    SramInternal,
    /// SRAM attach to the microcontroller externally
    SramExternal,
    /// Internal flash memory
    Flash,
    /// Microcontroller peripherals
    Peripheral,
}

/// Memory region configuration
pub struct Config<S> {
    /// Region base address
    pub addr: *const usize,
    /// Memory type
    pub memory: Type,
    /// Size of region
    pub size: S,
    /// Permissions
    pub access: Access,
    /// Memory region can be used to fetch instructions
    pub executable: bool,
}