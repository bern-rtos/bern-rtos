use core::alloc::Layout;
use core::ptr::NonNull;

pub struct AllocError;

pub trait Allocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);
    fn capacity(&self) -> usize;
}