use crate::alloc::allocator::{AllocError, Allocator};
use crate::log;
use bern_units::memory_size::{Byte, ExtByte};
use core::alloc::Layout;
use core::ptr::{slice_from_raw_parts_mut, NonNull};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

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
        loop {
            // CAS loop
            let old = self.current.load(Ordering::Acquire);
            let padding = old.align_offset(layout.align());
            log::trace!(
                "Try to allocate {}B at 0x{:x}",
                layout.size(),
                old as usize + padding
            );

            if (self.capacity() - self.usage()) < ((layout.size() + padding) as u32).B() {
                return Err(AllocError::OutOfMemory);
            }

            // Note(unsafe): We checked the size requirements already
            unsafe {
                match self.current.compare_exchange(
                    old,
                    old.add(layout.size() + padding),
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        let memory = slice_from_raw_parts_mut(old.add(padding), layout.size());
                        self.wastage.fetch_add(padding, Ordering::Relaxed);
                        return Ok(NonNull::new_unchecked(memory));
                    }
                    Err(_) => continue, // Allocation was interrupted, restart
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        log::warn!(
            "BumpAllocator cannot deallocate memory (0x{:x}, {}B). Ignoring call from .",
            ptr.as_ptr() as usize,
            layout.size()
        );
        self.wastage.fetch_add(layout.size(), Ordering::Relaxed);
    }

    fn capacity(&self) -> Byte {
        Byte((self.end.as_ptr() as usize - self.start.as_ptr() as usize) as u32)
    }

    fn usage(&self) -> Byte {
        Byte((self.current.load(Ordering::Relaxed) as usize - self.start.as_ptr() as usize) as u32)
    }
}

// Note(unsafe): We use atomic pointers.
unsafe impl Sync for Bump {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    struct MyStruct {
        a: u32,
        b: u8,
    }

    #[test]
    fn alloc_and_dealloc() {
        // Manually align array to 32 bit by using `u32`.
        static mut BUFFER: [u32; 128 / 4] = [0; 128 / 4];

        let bump = unsafe {
            Bump::new(
                NonNull::new_unchecked(BUFFER.as_ptr() as *mut _),
                NonNull::new_unchecked(BUFFER.as_ptr().add(BUFFER.len()) as *mut _),
            )
        };
        assert_eq!(bump.capacity().0, 128);

        let layout =
            Layout::from_size_align(size_of::<MyStruct>(), align_of::<MyStruct>()).unwrap();
        let raw = bump.alloc(layout).unwrap();

        let memory = raw.cast::<MyStruct>();
        let s = unsafe { &mut *memory.as_ptr() };

        // Check that we can actually write to the variable.
        s.a = 42;
        s.b = 10;

        let usage = size_of::<MyStruct>() as u32;
        assert_eq!(
            bump.usage().0,
            (usage + bump.wastage.load(Ordering::Relaxed) as u32)
        );
        let wastage = bump.wastage.load(Ordering::Relaxed);

        unsafe {
            bump.dealloc(raw.cast::<u8>(), layout);
        }
        // Memory can not be reused only wasted with the bump allocator.
        assert_eq!(bump.usage().0, usage);
        assert_eq!(
            bump.wastage.load(Ordering::Relaxed),
            wastage + size_of::<MyStruct>()
        );
    }

    #[test]
    fn overflow() {
        static mut BUFFER: [u8; 128] = [0; 128];

        let bump = unsafe {
            Bump::new(
                NonNull::new_unchecked(BUFFER.as_ptr() as *mut _),
                NonNull::new_unchecked(BUFFER.as_ptr().add(BUFFER.len()) as *mut _),
            )
        };
        assert_eq!(bump.capacity().0, 128);

        let layout = Layout::from_size_align(16, 1).unwrap();
        for _ in 0..8 {
            let _dump = bump.alloc(layout).unwrap();
        }

        assert_eq!(bump.usage(), bump.capacity());

        let err = bump.alloc(layout).err().unwrap();
        assert_eq!(err, AllocError::OutOfMemory);
    }
}
