//! Data structures.

pub mod boxed;
pub mod linked_list;
pub mod allocator;
pub mod bump_allocator;
pub mod wrapper;

pub use bern_arch::arch::memory_protection::Size;
pub use bern_arch::{size_from_raw, alignment_from_size};


use crate::mem::wrapper::Wrapper;

#[global_allocator]
static ALLOCATOR: Wrapper = Wrapper::new();