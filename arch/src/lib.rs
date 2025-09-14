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

pub mod core;
pub mod memory_protection;
pub mod scheduler;
pub mod startup;
pub mod sync;
pub mod syscall;

// re-exports
pub use crate::core::ICore;
pub use crate::memory_protection::IMemoryProtection;
pub use crate::scheduler::IScheduler;
pub use crate::startup::IStartup;
pub use crate::sync::ISync;
pub use crate::syscall::ISyscall;

// select architecture support
#[cfg(not(target_os = "none"))]
pub mod mock;
#[cfg(not(target_os = "none"))]
pub use crate::mock as arch;

#[cfg(all(arm_cortex_m, target_os = "none"))]
pub mod cortex_m;
#[cfg(all(arm_cortex_m, target_os = "none"))]
pub use crate::cortex_m as arch;
