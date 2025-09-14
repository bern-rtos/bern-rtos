//! ARM Cortex-M hardware support.

pub mod core;
mod interrupt;
pub mod memory_protection;
mod mpu;
mod register;
mod scheduler;
pub mod startup;
pub mod sync;
pub mod syscall;
mod tick;

pub struct Arch;

// re-exports
pub use crate::cortex_m::core::ArchCore;
pub use cortex_m_rt;
