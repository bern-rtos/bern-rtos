//! Data structures.

pub mod boxed;
pub mod linked_list;

pub use bern_arch::arch::memory_protection::Size;
pub use bern_arch::{alignment_from_size, size_from_raw};
