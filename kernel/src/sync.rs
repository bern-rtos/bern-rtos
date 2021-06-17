//! Synchronization primitives.

#[allow(dead_code)]

pub(crate) mod critical_mutex;
pub(crate) mod critical_section;
mod mutex;
mod semaphore;

pub use mutex::{Mutex, MutexGuard};
pub use semaphore::{Semaphore, SemaphorePermit};

/// Common error type for all sync primitives
pub enum Error {
    /// Would block task
    WouldBlock,
    /// Request timed out
    TimeOut,
    /// Task holding the sync primitive panicked
    Poisoned,
    /// Cannot allocate primitive
    OutOfMemory,
}