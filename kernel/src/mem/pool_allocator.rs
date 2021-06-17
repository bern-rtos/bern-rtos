use crate::mem::boxed::Box;

/// Memory allocation errors.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// No more memory available for allocator
    OutOfMemory,
    /// An unknown error occured
    Unknown,
}

/// Pool based allocators.
pub trait PoolAllocator<T> {
    /// Allocate and move an element to a pool and return box to value if
    /// operation succeeded.
    fn insert(&self, element: T) -> Result<Box<T>, Error>;
    // todo: drop
}