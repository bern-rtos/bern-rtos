//! ARM Cortex-M hardware support.

pub mod syscall;
pub mod core;
pub mod sync;
mod scheduler;
mod tick;
mod register;
pub mod startup;
pub mod memory_protection;
mod mpu;
mod interrupt;

pub struct Arch;

// re-exports
pub use crate::cortex_m::core::ArchCore;
pub use cortex_m_rt;