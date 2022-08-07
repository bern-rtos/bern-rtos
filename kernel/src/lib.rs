//! Bern RTOS preemptive real-time kernel for microcontrollers written in Rust.
//!
//! # Documentation
//!
//! - [Bern RTOS Kernel Book](https://kernel.bern-rtos.org/)
//! - [API Documentation](https://docs.rs/bern-kernel/)
//!
//! **The API Documentation is not up to date, please prefer the
//! [Bern RTOS Kernel Book](https://kernel.bern-rtos.org/) for now.**
//!
//! # Semantic Versioning
//!
//! This project follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).
//!
//! Currently the version is below 1.0.0 meaning that everything is very much unstable
//! and the API could change completely.
//!
//! # Cargo Features
//!
//! - `time-slicing` (default): A task runs at max for one system tick period if there are
//!   other tasks of equal priority
//! - `log-defmt`: Activate system log messages using [`defmt`](https://docs.rs/defmt/latest/defmt/).
//!   The user must select a log transport in the application, e.g. `defmt-rtt`.
//! - `log-rtt`: Activate system log messages with `core` formatting and RTT transport.
//! - `log-global`: Activate system log messages using the [`log`](https://docs.rs/log/) facade.
//!   The use must provide a global logger.
//! - `trace`: Activate system tracing. The user must provide a global tracer,
//!   e.g. [`systemview-target`](https://docs.rs/systemview-target/).
//!
//! # License
//! - [MIT License](https://gitlab.com/bern-rtos/bern-rtos/-/blob/main/kernel/LICENSE.md)
//!
//! # Supported Architectures
//!
//! | Core Name | Architecture | Rust Target |
//! |-----------|--------------|-------------|
//! | ARM Cortex-M3 w/MPU | Armv7-M | `thumbv7m-none-eabi` |
//! | ARM Cortex-M4 w/MPU | Armv7E-M | `thumbv7em-none-eabi` |
//! | ARM Cortex-M7 w/MPU | Armv7E-M | `thumbv7em-none-eabi` |
//!
//! # Quickstart
//!
//! ```sh,no_run
//! cargo generate --git https://gitlab.com/bern-rtos/templates/cortex-m.git
//! ```

#![cfg_attr(target_os = "none", no_std)]

mod sched;
pub mod syscall;
pub mod time;
pub mod stack;
pub mod sync;
pub mod mem;
pub mod kernel;
pub mod alloc;
pub mod exec;
pub mod log;

pub use crate::syscall::*;
pub use bern_kernel_macros::*;
pub use bern_units as units;

pub use embedded_time;

#[allow(unused_imports)]
use bern_arch::arch as _;
pub use bern_arch;
pub use kernel::*;

