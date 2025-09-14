//! Bern RTOS kernel architecture support.
//!
//! This crates was developed for the `bern_kernel` but can be used as basis for
//! any real-time operating system.
//!
//! # Documentation
//!
//! - [Bern RTOS Kernel Book](https://kernel.bern-rtos.org/)
//! - [API Documentation](https://docs.rs/bern-arch/)
//!
//! # License
//! - [MIT License](https://gitlab.com/bern-rtos/bern-rtos/-/blob/main/arch/LICENSE.md)

#![cfg_attr(target_os = "none", no_std)]

#![allow(unused)]

pub mod syscall;
pub mod core;
pub mod scheduler;
pub mod sync;
pub mod startup;
pub mod memory_protection;

// re-exports
pub use crate::scheduler::IScheduler;
pub use crate::syscall::ISyscall;
pub use crate::core::ICore;
pub use crate::sync::ISync;
pub use crate::startup::IStartup;
pub use crate::memory_protection::IMemoryProtection;

// select architecture support
#[cfg(not(target_os = "none"))]
pub mod mock;
#[cfg(not(target_os = "none"))]
pub use crate::mock as arch;

#[cfg(all(arm_cortex_m, target_os = "none"))]
pub mod cortex_m;
#[cfg(all(arm_cortex_m, target_os = "none"))]
pub use crate::cortex_m as arch;