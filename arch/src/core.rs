//! CPU core peripherals.

/// Execution modes of the CPU.
pub enum ExecMode {
    /// Kernel is active, e.g. syscall or ISR.
    Kernel,
    /// CPU is in thread mode.
    Thread,
}

/// CPU core peripherals.
pub trait ICore {
    /// Setup core peripherals and return core object.
    fn new() -> Self;
    /// Set the system tick divisor.
    fn set_systick_div(&mut self, divisor: u32);
    /// Start peripherals used by kernel.
    fn start(&mut self);
    /// Break point instruction.
    fn bkpt();
    /// CPU execution mode.
    fn execution_mode() -> ExecMode;
    /// Returns true if the CPU is processing an interrupt.
    fn is_in_interrupt() -> bool;
    /// Cycles counted by CPU for debug purposes.
    fn debug_time() -> u32;
}