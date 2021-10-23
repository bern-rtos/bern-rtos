//! Data structures.

pub mod boxed;
pub mod linked_list;
pub mod allocator;
pub mod strict_allocator;

pub use bern_arch::arch::memory_protection::Size;
pub use bern_arch::{size_from_raw, alignment_from_size};