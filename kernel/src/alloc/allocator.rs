use bern_units::memory_size::Byte;
use core::alloc::Layout;
use core::ptr::NonNull;

#[allow(unused)]
#[derive(Debug, Eq, PartialEq)]
pub enum AllocError {
    OutOfMemory,
    WrongAlignment,
    Other,
}

pub trait Allocator {
    /// Try to allocate a layout.
    fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;
    /// Deallocate memory at a location with a laoyut.
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);
    /// Total capacity of allocator.
    fn capacity(&self) -> Byte;
    /// Memory used by the allocator,
    fn usage(&self) -> Byte;
}
