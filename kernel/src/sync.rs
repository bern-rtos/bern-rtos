//! Synchronization primitives.

#[allow(dead_code)]
pub(crate) mod critical_mutex;
pub(crate) mod critical_section;
pub mod mutex;
pub mod semaphore;

pub mod channel;
pub mod mpsc;
pub mod spsc;

pub mod ipc;

pub use mutex::{Mutex, MutexGuard};
pub use semaphore::{Semaphore, SemaphorePermit};

/// Common error type for all sync primitives
#[derive(Debug)]
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
