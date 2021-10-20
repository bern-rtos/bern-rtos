use core::alloc::Layout;
use core::ptr::NonNull;

pub struct AllocError;

pub trait Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, AllocError>;
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);
    fn capacity(&self) -> usize;
}