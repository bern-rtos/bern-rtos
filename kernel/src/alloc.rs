pub mod allocator;
pub mod bump;
pub mod heap;
pub mod pool;
pub mod wrapper;
pub mod const_pool;

use crate::alloc::wrapper::Wrapper;

#[global_allocator]
static ALLOCATOR: Wrapper = Wrapper::new();
