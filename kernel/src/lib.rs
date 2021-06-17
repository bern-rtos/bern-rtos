//! Bern RTOS kernel for microcontroller.
//!
//! # Documentation
//! Refer to the kernel book [kernel.bern-rtos.org](https://kerneel.bern-rtos.org).
//!
//! # Semantic Versioning
//! This project follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).
//!
//! Currently the version is below 1.0.0 meaning that everything is very much unstable
//! and the API could change completely.
//!
//! # Cargo features
//! - `time-slicing`: A task runs at max for one system tick period if there are
//!   other tasks of equal priority

#![cfg_attr(target_os = "none", no_std)]
#![feature(unsize)]
#![feature(asm)]
#![feature(naked_functions)]

pub mod task;
pub mod sched;
pub mod syscall;
pub mod time;
pub mod stack;
pub mod sync;
pub mod mem;

pub use crate::syscall::*;
pub use bern_kernel_macros::*;

#[allow(unused_imports)]
use bern_arch::arch as _;
pub use bern_arch;
