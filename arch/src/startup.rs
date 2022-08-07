//! Startup.

/// Memory region for startup code.
#[derive(Copy, Clone)]
pub struct Region {
    /// Region base address.
    pub start: *const usize,
    /// Region end address.
    pub end: *const usize,
    /// Address from which data will be loaded.
    pub data: Option<*const usize>,
}

/// Startup.
pub trait IStartup {
    /// Init static region.
    fn init_static_region(region: Region);
    /// Kernel data region.
    fn kernel_data() -> Region;
    /// Kernel heap region.
    fn kernel_heap() -> Region;
}