//! Startup.

/// Memory region for startup code.
#[derive(Copy, Clone)]
pub struct Region {
    /// Region base address
    pub start: *const usize,
    /// Region end address
    pub stop: *const usize,
}

/// Startup.
pub trait IStartup {
    /// Initialize static memory section, which are not already initialized.
    fn init_static_memory();
    /// Get specific memory region from linker script.
    fn region() -> Region;
}