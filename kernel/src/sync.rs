//! Synchronization primitives.

#[allow(dead_code)]

pub(crate) mod critical_mutex;
pub(crate) mod critical_section;
mod mutex;
mod semaphore;

pub mod channel;
pub mod spsc;
pub mod mpsc;

pub mod ipc_channel;
pub mod ipc_spsc;

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
