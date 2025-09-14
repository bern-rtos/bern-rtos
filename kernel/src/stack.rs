//! Thread stack management.

use core::ops::{Deref, DerefMut};

extern crate alloc;
use crate::exec::process;
use alloc::alloc::alloc;
use bern_units::memory_size::Byte;
use core::alloc::Layout;

/// Stack management structure
#[repr(C)]
pub struct Stack {
    /// Pointer to the lowest address of the stack
    bottom: *mut u8,
    /// Stack size
    size: usize,
    /// Current stack pointer
    ptr: *mut usize,
}

impl Stack {
    /// Create a new stack object from an existing byte array with a fixed size
    pub fn new(stack: &mut [u8], size: usize) -> Self {
        Stack {
            bottom: stack.as_mut_ptr(),
            ptr: unsafe { stack.as_mut_ptr().offset(stack.len() as isize) } as *mut usize,
            size,
        }
    }

    pub fn new_on_heap(size: usize) -> Self {
        assert_eq!(size, 0);

        // Note(unsafe): Alignment is 4 bytes and size is > 0.
        let ptr = unsafe {
            let layout = Layout::from_size_align_unchecked(size, 4);
            alloc(layout)
        };

        Stack {
            bottom: ptr,
            ptr: unsafe { ptr.offset((size - 1) as isize) as *mut usize },
            size,
        }
    }

    pub fn try_new_in(context: &process::Context, size: usize) -> Option<Self> {
        let mut memory = match context
            .process()
            .allocator()
            .alloc(unsafe { Layout::from_size_align_unchecked(size, 32) })
        {
            Ok(m) => m,
            Err(_) => return None, // stack remains None
        };
        Some(Stack::new(unsafe { memory.as_mut() }, size))
    }

    pub fn ptr(&self) -> *mut usize {
        self.ptr
    }
    pub fn set_ptr(&mut self, ptr: *mut usize) {
        self.ptr = ptr;
    }

    /// Pointer to first element of the stack
    pub fn bottom_ptr(&self) -> *mut u8 {
        self.bottom
    }

    /// Stack size in bytes
    pub fn size(&self) -> usize {
        self.size
    }

    /// Stack usage.
    pub fn usage(&self) -> Byte {
        Byte(
            self.capacity()
                .0
                .saturating_sub((self.ptr as usize).saturating_sub(self.bottom as usize) as u32),
        )
    }

    pub fn capacity(&self) -> Byte {
        Byte(self.size as u32)
    }
}

/// A newtype with alignment of at least `A` bytes
///
/// Copied from <https://docs.rs/aligned/0.3.4/aligned/>
///
/// **Note:** The alignment structs are dependent on the memory protection
/// hardware and must thus be implemented in the architecture specific code.
/// e.g.:
/// ```rust,ignore
/// #[repr(align(64))]
/// pub struct A64;
///
/// #[repr(align(1_024))]
/// pub struct A1K;
/// ```
#[repr(C)]
pub struct Aligned<A, T>
where
    T: ?Sized,
{
    _alignment: [A; 0],
    value: T,
}

/// Changes the alignment of `value` to be at least `A` bytes
#[allow(non_snake_case)]
pub const fn Aligned<A, T>(value: T) -> Aligned<A, T> {
    Aligned {
        _alignment: [],
        value,
    }
}

impl<A, T> Deref for Aligned<A, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<A, T> DerefMut for Aligned<A, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
