//! Data structures.

pub(crate) mod pool_allocator;
pub(crate) mod array_pool;
pub mod boxed;
pub mod linked_list;
pub(crate) mod strict_allocator;
pub(crate) mod allocator;

pub use bern_arch::arch::memory_protection::Size;
pub use bern_arch::{size_from_raw, alignment_from_size};