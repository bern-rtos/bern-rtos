use core::alloc::Layout;
use core::ptr::{NonNull, slice_from_raw_parts_mut};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use bern_units::memory_size::{Byte, ExtByte};
use crate::alloc::allocator::{Allocator, AllocError};
use crate::log;

/// The strict memory allocator can allocate memory but never release.
pub struct Bump {
    /// Start of memory block.
    start: NonNull<u8>,
    /// End of memory block.
    end: NonNull<u8>,
    /// Current allocation pointer.
    current: AtomicPtr<u8>,
    /// Memory wasted due to padding.
    wastage: AtomicUsize,
}

impl Bump {
    ///
    /// # Safety
    /// `start` must be a valid address and the memory block must not exceed its
    /// intended range.
    pub const unsafe fn new(start: NonNull<u8>, end: NonNull<u8>) -> Self {
        Bump {
            start,
            end,
            current: AtomicPtr::new(start.as_ptr()),
            wastage: AtomicUsize::new(0),
        }
    }
}

impl Allocator for Bump {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        loop { // CAS loop
            let old = self.current.load(Ordering::Acquire);
            let padding = old.align_offset(layout.align());
            log::trace!(
                "Try to allocate {}B at 0x{:x}",
                layout.size(),
                old as usize + padding
            );

            if self.capacity() < ((layout.size() + padding) as u32).B() {
                return Err(AllocError::OutOfMemory);
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
        log::warn!(
            "BumpAllocator cannot deallocate memory (0x{:x}, {}B). Ignoring call from .",
            ptr.as_ptr(),
            layout.size()
        );
    }

    fn capacity(&self) -> Byte {
        Byte((self.end.as_ptr() as usize - self.start.as_ptr() as usize) as u32)
    }

    fn usage(&self) -> Byte {
        Byte((self.end.as_ptr() as usize -
            self.current.load(Ordering::Relaxed) as usize) as u32)
    }
}

// Note(unsafe): We use atomic pointers.
unsafe impl Sync for Bump { }