//! Bern RTOS kernel configuration.
//!
//! This is the default kernel config. To apply your own config clone the this
//! crate into your project and apply a cargo path to override the default
//! config:
//! ```toml
//! # `Cargo.toml`
//! [patch.crates-io]
//! bern-conf = { path = "conf" }
//! ```
//!
//! [Example Configuration](../src/bern_conf/conf.rs.html#18-44)

#![no_std]

use bern_units::memory_size::Byte;
use bern_conf_type::*;


pub const CONF: Conf = Conf {
    task: Task {
        pool_size: 8,
        priorities: 8,
    },
    event: Event {
        pool_size: 32,
    },
    memory: Memory {
        flash: MemorySection {
            start_address: 0x0800_0000,
            size: Byte::from_kb(512),
        },
        sram: MemorySection {
            start_address: 0x2000_0000,
            size: Byte::from_kb(128),
        },
        peripheral: MemorySection {
            start_address: 0x4000_0000,
            size: Byte::from_kb(512),
        },
        shared: MemorySection {
            // will be ignored, start of shared section is read via linker
            // symbol
            start_address: 0x2001FC00,
            size: Byte::from_kb(1),
        }
    },
};