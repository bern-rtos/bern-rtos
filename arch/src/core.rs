//! CPU core peripherals.

pub enum ExecMode {
    Kernel,
    Thread,
}

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

    fn execution_mode() -> ExecMode;

    fn is_in_interrupt() -> bool;

    fn debug_time() -> u32;
}