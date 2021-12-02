use core::alloc::Layout;
use core::ptr::{NonNull, slice_from_raw_parts_mut};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::mem::allocator::{Allocator, AllocError};

/// The strict memory allocator can allocate memory but never release.
pub struct BumpAllocator {
    /// End of memory block.
    end: NonNull<u8>,
    /// Current allocation pointer.
    current: AtomicPtr<u8>,
    /// Memory wasted due to padding.
    wastage: AtomicUsize,
}

impl BumpAllocator {
    ///
    /// # Safety
    /// `start` must be a valid address and the memory block must not exceed its
    /// intended range.
    pub const unsafe fn new(start: NonNull<u8>, end: NonNull<u8>) -> Self {
        BumpAllocator {
            end,
            current: AtomicPtr::new(start.as_ptr()),
            wastage: AtomicUsize::new(0),
        }
    }
}

impl Allocator for BumpAllocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        loop { // CAS loop
            let old = self.current.load(Ordering::Acquire);
            let padding = old.align_offset(layout.align());
            defmt::trace!(
                "Try allocating {}B at 0x{:x}",
                layout.size(),
                old as usize + padding
            );

            if self.capacity() < (layout.size() + padding) {
                return Err(AllocError);
            }

            // Note(unsafe): We checked the size requirements already
            unsafe {
                match self.current.compare_exchange(
                    old,
                    old.add(layout.size() + padding),
                    Ordering::SeqCst,
                    Ordering::Relaxed
                ) {
                    Ok(_) => {
                        let memory = slice_from_raw_parts_mut(old.add(padding), layout.size());
                        self.wastage.fetch_add(padding, Ordering::Relaxed);
                        return Ok(NonNull::new_unchecked(memory));
                    },
                    Err(_) => continue, // Allocation was interrupted, restart
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        defmt::warn!(
            "BumpAllocator cannot deallocate memory (0x{:x}, {}B). Ignoring call from .",
            ptr.as_ptr(),
            layout.size()
        );
    }

    fn capacity(&self) -> usize {
        self.end.as_ptr() as usize -
            self.current.load(Ordering::Relaxed) as usize
    }
}