use core::cell::UnsafeCell;
use core::ptr::NonNull;

use crate::mem::pool_allocator::{PoolAllocator, Error};
use crate::mem::boxed::Box;

/// Fixed size data pool based on arrays
///
/// # Example
/// ```no_run
/// static POOL: ArrayPool<Node<Task>, 10> = ArrayPool::new([None; 10]);
/// let mut list = LinkedList::new(&POOL);
/// ```
#[derive(Debug)]
pub struct ArrayPool<T, const N: usize> {
    pool: UnsafeCell<[Option<T>; N]>,
}

impl<T, const N: usize> ArrayPool<T, {N}>
{
    /// Create a new data pool from an existing array
    ///
    /// **Note:** The array must consist of `Option` type to evaluate whether
    /// as cell is empty or not.
    pub const fn new(array: [Option<T>; N]) -> Self {
        ArrayPool {
            pool: UnsafeCell::new(array),
        }
    }
}

// todo: make sync safe!
unsafe impl<T, const N: usize> Sync for ArrayPool<T, {N}> {}


impl<T, const N: usize> PoolAllocator<T> for ArrayPool<T, {N}> {
    fn insert(&self, element: T) -> Result<Box<T>, Error> {
        // NOTE(unsafe): must be called from critical section
        for item in unsafe { &mut *self.pool.get() }.iter_mut() {
            if item.is_none() {
                *item = Some(element);
                match item {
                    Some(i) => unsafe {
                        return Ok(Box::from_raw(NonNull::new_unchecked(i as *mut _)))
                    },
                    _ => return Err(Error::Unknown),
                }
            }
        }
        return Err(Error::OutOfMemory);
    }
}