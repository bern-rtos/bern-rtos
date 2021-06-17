//! Synchronization.

/// Synchronization.
pub trait ISync {
    /// Disable any interrupt below priority.
    fn disable_interrupts(priority: usize);
    /// Enable all interrupts.
    fn enable_interrupts();
}