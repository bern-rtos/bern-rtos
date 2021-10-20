use core::alloc::Layout;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};
use crate::mem::allocator::{Allocator, AllocError};

/// The strict memory allocator can allocate memory but never release.
pub struct StrictAllocator {
    start: NonNull<u8>,
    end: NonNull<u8>,
    current: AtomicPtr<u8>,
}

impl StrictAllocator {
    ///
    /// # Safety
    /// `start` must be a valid address and the memory block must not exceed its
    /// intended range.
    pub unsafe fn new(start: NonNull<u8>, size: usize) -> Self {
        StrictAllocator {
            start,
            end: NonNull::new_unchecked(start.as_ptr().add(size)),
            current: AtomicPtr::new(start.as_ptr())
        }
    }
}

impl Allocator for StrictAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let old = self.current.load(Ordering::Relaxed);
        let padding = old.align_offset(layout.align());

        if self.capacity() < (layout.size() + padding) {
            return Err(AllocError);
        }

        // Note(unsafe): we checked the size requirements already
        unsafe {
            match self.current.compare_exchange(
                old,
                old.add(layout.size() + padding),
                Ordering::SeqCst,
                Ordering::Release
            ) {
                Ok(_) => Ok(NonNull::new_unchecked(old)),
                Err(_) => Err(AllocError)
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unimplemented!();
    }

    fn capacity(&self) -> usize {
        self.end.as_ptr() as usize -
            self.current.load(Ordering::Relaxed) as usize
    }
}