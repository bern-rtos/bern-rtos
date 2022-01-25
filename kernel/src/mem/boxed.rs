//! A pointer type for dynamically allocated memory.
//!
//! Similar to [`std::boxed::Box`](https://doc.rust-lang.org/std/boxed/index.html)
//! [`Box<T>`] is a pointer type that can deallocate memory when it goes out of
//! scope.
//!
//! This crate can also be used for pointer to statics because in contrast to
//! pointers [`Box<T>`] cannot be copied and an accidental drop can be caught.
//!
//! **Note:** in the current implementation the type is static, but de-/allocation
//! will be added with a dynamic memory allocation feature. The type should work
//! with pooled as well as heap allocators.
//!
//! # Examples
//!
//! Allocating a data structure in a static pool will return a boxed pointer to
//! the new value:
//! ```no_run
//! static POOL: ArrayPool<Node<MyStruct>, 10> = ArrayPool::new([None; 10]);
//! let boxed: Box<MyStruct> = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
//! ```

use core::alloc::Layout;
use core::mem;
use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};
use crate::alloc::allocator::{Allocator, AllocError};

/// A pointer type for dynamically allocated memory.
///
/// For more details see [module-level documentation](../index.html)
pub struct Box<T> {
    boxed: NonNull<T>,
}

impl<T> Box<T> {
    /// Try to move a value to an allocated memory space.
    pub fn try_new_in(value: T, alloc: &'static dyn Allocator) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let memory = match alloc.alloc(layout) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let memory = memory.cast::<T>();
        // Note(unsafe): Memory size checked by the allocator
        unsafe {
            memory.as_ptr().write(value);
        }

        Ok(Box {
            boxed: memory,
        })
    }


    /// Create a box from a `NonNull` pointer
    pub unsafe fn from_raw(pointer: NonNull<T>) -> Self {
        Box {
            boxed: pointer,
        }
    }

    /// Returns the pointer to the box structure on memory.
    pub fn leak(b: Self) -> NonNull<T> {
        let boxed = b.boxed;
        mem::forget(b);
        boxed
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.boxed.as_ref()) }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.boxed.as_mut()) }
    }
}

impl<'a,T> Drop for Box<T> {
    fn drop(&mut self) {
        panic!("Box drop not implemented yet!");
    }
}