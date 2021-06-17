//! CPU core peripherals.

/// CPU core peripherals.
pub trait ICore {
    /// Setup core peripherals and return core object
    fn new() -> Self;
    /// Set the system tick divisor
    fn set_systick_div(&mut self, divisor: u32);
    /// Start peripherals used by kernel
    fn start(&mut self);
    /// Break point instruction
    fn bkpt();
}