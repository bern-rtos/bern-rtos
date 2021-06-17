//! Bern RTOS kernel architecture support.
//!
//! This crates was developed for the `bern_kernel` but can be used as basis for
//! any real-time operating system.
//!
//! # Documentation
//! Refer to the kernel book [kernel.bern-rtos.org](https://kerneel.bern-rtos.org).
//!
//! # Supported Architectures
//! | Core Name | Architecture | Rust Target |
//! |-----------|--------------|-------------|
//! | ARM Cortex-M4 w/MPU | ARMv7E-M | `thumbv7em-none-eabi` |

#![cfg_attr(target_os = "none", no_std)]
#![feature(asm)]
#![feature(naked_functions)]

#![cfg_attr(not(target_os = "none"), never_type)]

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

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod cortex_m;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::cortex_m as arch;