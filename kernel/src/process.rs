use core::alloc::Layout;
use core::ptr::NonNull;
use crate::mem::allocator::{Allocator, AllocError};
use crate::mem::bump_allocator::BumpAllocator;
use crate::mem::Size;
use crate::stack::Stack;
use crate::task;

pub struct Process {
    proc_memory: &'static mut [u8],
    proc_allocator: BumpAllocator,
    size: Size,
}

impl Process {
    pub fn new(proc_memory: &'static mut [u8], size: Size) -> Self {
        let proc_allocator = unsafe {
            BumpAllocator::new(
                NonNull::new_unchecked(proc_memory.as_mut_ptr()),
                proc_memory.len()
            )};

        Process {
            size,
            proc_memory,
            proc_allocator
        }
    }

    pub fn create_thread(&mut self) -> task::TaskBuilder {
        task::Task::new(self)
    }

    pub(crate) fn request_memory(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.proc_allocator.allocate(layout)
    }

    pub(crate) fn start_addr(&self) -> *const u8 {
        self.proc_memory.as_ptr()
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }
}


#[macro_export]
macro_rules! alloc_process_memory {
    ($name:ident, $size:tt) => {
        {
            #[link_section = ".proc.$name"]
            static mut MEMORY: $crate::stack::Aligned<$crate::bern_arch::alignment_from_size!($size), [u8; $size]> =
                $crate::stack::Aligned([0; $size]);

            // this is unsound, because the same stack can 'allocated' multiple times
            unsafe { &mut *MEMORY }
        }
    };
}