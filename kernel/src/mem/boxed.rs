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

use core::ptr::NonNull;
use core::ops::{Deref, DerefMut};

/// A pointer type for dynamically allocated memory.
///
/// For more details see [module-level documentation](../index.html)
#[derive(Debug)]
pub struct Box<T> {
    value: NonNull<T>,
}

impl<T> Box<T> {
    /// Create a box from a `NonNull` pointer
    // todo: add ref to allocator to create and drop memory
    pub fn from_raw(pointer: NonNull<T>) -> Self {
        Box {
            value: pointer,
        }
    }

    /// Consume the box and return the inner `NonNull` pointer
    pub fn into_nonnull(self) -> NonNull<T> {
        self.value
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}