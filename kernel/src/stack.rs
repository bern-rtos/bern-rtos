//! Task stack management.
//!
//! # Example
//! You typically want to use a macro allocate a static stack ([`alloc_static_stack`]) and create the
//! management object accordingly:
//! ```no_run
//! let stack: bern_kernel::stack::Stack = alloc_static_stack!(512);
//! ```

use core::ops::{DerefMut, Deref};
use crate::bern_arch::arch::memory_protection::Size;

/// Stack management structure
#[repr(C)]
pub struct Stack {
    /// Pointer to the first element of the stack
    bottom: *mut u8,
    /// Stack size
    size: Size,
    /// Current stack pointer
    pub ptr: *mut usize,
}

impl Stack {
    /// Create a new stack object from an existing byte array with a fixed size
    pub fn new(stack: &mut [u8], size: Size) -> Self {
        Stack {
            bottom: stack.as_mut_ptr(),
            ptr: unsafe { stack.as_mut_ptr().offset(stack.len() as isize) } as *mut usize,
            size
        }
    }

    /// Pointer to first element of the stack
    pub fn bottom_ptr(&self) -> *mut u8 {
        self.bottom
    }

    /// Stack size in bytes
    pub fn size(&self) -> Size {
        self.size
    }
}

/// A newtype with alignment of at least `A` bytes
///
/// Copied from <https://docs.rs/aligned/0.3.4/aligned/>
///
/// **Note:** The alignment structs are dependent on the memory protection
/// hardware and must thus be implemented in the architecture specific code.
/// e.g.:
/// ```rust,no_run
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

/// Allocate a static stack with given size in bytes.
///
/// The macro checks whether the size meets the size and alignment requirements
/// of the memory protection hardware in use. e.g.
/// ```no_run
/// let stack: bern_kernel::stack::Stack = alloc_static_stack!(512);
/// // ok
///
/// let stack: bern_kernel::stack::Stack = alloc_static_stack!(431);
/// // compiler error on cortex-m as size is not feasible for memory protection
/// ```
///
/// The arrays for static stacks are put in the `.task_stack` linker section.
///
/// # Safety
/// This macro must no be called multiple times as new stack objects to the same
/// static stack will be returned.
#[macro_export]
macro_rules! alloc_static_stack {
    ($size:tt) => {
        {
            #[link_section = ".task_stack"]
            static mut STACK: $crate::stack::Aligned<$crate::bern_arch::alignment_from_size!($size), [u8; $size]> =
                $crate::stack::Aligned([0; $size]);

            // this is unsound, because the same stack can 'allocated' multiple times
            unsafe { $crate::stack::Stack::new(&mut *STACK, $crate::bern_arch::size_from_raw!($size)) }
        }
    };
}