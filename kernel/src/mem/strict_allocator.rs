use core::alloc::Layout;
use core::ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::mem::allocator::{Allocator, AllocError};

/// The strict memory allocator can allocate memory but never release.
pub struct StrictAllocator {
    /// Memory block from memory can be allocated.
    pool: *mut [u8],
    /// End of memory block.
    end: NonNull<u8>,
    /// Current allocation pointer.
    current: AtomicPtr<u8>,
    /// Memory wasted due to padding.
    wastage: AtomicUsize,
}

impl StrictAllocator {
    ///
    /// # Safety
    /// `start` must be a valid address and the memory block must not exceed its
    /// intended range.
    pub unsafe fn new(start: NonNull<u8>, size: usize) -> Self {
        StrictAllocator {
            pool: slice_from_raw_parts_mut(start.as_ptr(), size),
            end: NonNull::new_unchecked(start.as_ptr().add(size)),
            current: AtomicPtr::new(start.as_ptr()),
            wastage: AtomicUsize::new(0),
        }
    }
}

impl Allocator for StrictAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let old = self.current.load(Ordering::Relaxed);
        let padding = old.align_offset(layout.align());
        // Note(unsafe): No data is accessed before size check.
        let memory = unsafe {
            slice_from_raw_parts_mut(old.add(padding), layout.size())
        };

        if self.capacity() < (layout.size() + padding) {
            return Err(AllocError);
        }

        // Note(unsafe): we checked the size requirements already
        unsafe {
            match self.current.compare_exchange(
                old,
                old.add(layout.size() + padding),
                Ordering::SeqCst,
                Ordering::Relaxed
            ) {
                Ok(_) => {
                    self.wastage.fetch_add(padding, Ordering::Relaxed);
                    Ok(NonNull::new_unchecked(memory))
                },
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