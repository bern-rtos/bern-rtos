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
use crate::mem::allocator::{Allocator, AllocError};

// Box structure stored on a heap.
pub struct BoxData<T> {
    value: T,
    /// Allocator used for this box.
    ///
    /// **Note:** Allocators are stored as trait object to allow boxes with
    /// different types to be placed in one common container (e.g. a list).
    ///
    /// **Note:** A static reference is used because an allocator is not allowed
    /// to drop at runtime.
    alloc: &'static dyn Allocator,
}

impl<T> Deref for BoxData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for BoxData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// A pointer type for dynamically allocated memory.
///
/// For more details see [module-level documentation](../index.html)
pub struct Box<T> {
    boxed: NonNull<BoxData<T>>,
}

impl<T> Box<T> {
    /// Try to move a value to an allocated memory space.
    pub fn try_new_in(value: T, alloc: &'static dyn Allocator) -> Result<Self, AllocError> {
        let internal = BoxData {
            value,
            alloc,
        };

        // Note(unsafe): Alignment is power of two.
        let layout = unsafe {
            Layout::from_size_align_unchecked(mem::size_of_val(&internal), 4)
        };
        let memory = match alloc.allocate(layout) {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        let memory = memory.cast::<BoxData<T>>();
        // Note(unsafe): Memory size checked by the allocator
        unsafe {
            memory.as_ptr().write(internal);
        }

        Ok(Box {
            boxed: memory,
        })
    }


    /// Create a box from a `NonNull` pointer
    pub unsafe fn from_raw(pointer: NonNull<BoxData<T>>) -> Self {
        Box {
            boxed: pointer,
        }
    }

    /// Returns the pointer to the box structure on memory.
    pub fn leak(b: Self) -> NonNull<BoxData<T>> {
        let boxed = b.boxed;
        mem::forget(b);
        boxed
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.boxed.as_ref()).value }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.boxed.as_mut()).value }
    }
}

impl<'a,T> Drop for Box<T> {
    fn drop(&mut self) {
        panic!("Box drop not implemented yet!");
    }
}