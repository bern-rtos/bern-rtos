use core::alloc::Layout;
use core::ptr::NonNull;

#[allow(unused)]
#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    WrongAlignment,
    Other,
}

pub trait Allocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);
    fn capacity(&self) -> usize;
}