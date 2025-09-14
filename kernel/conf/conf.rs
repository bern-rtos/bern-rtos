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

use bern_conf_type::*;
use bern_units::memory_size::Byte;

pub const CONF: Conf<0> = Conf {
    kernel: Kernel {
        priorities: 8,
        memory_size: Byte::from_kB(2),
    },

    shared: Shared {
        size: Byte::from_kB(4),
    },

    memory_map: MemoryMap {
        flash: Memory {
            link_name: "FLASH",
            start_address: 0x0800_0000,
            size: Byte::from_kB(512),
        },
        sram: Memory {
            link_name: "RAM",
            start_address: 0x2000_0000,
            size: Byte::from_kB(128),
        },
        peripheral: Memory {
            link_name: "",
            start_address: 0x4000_0000,
            size: Byte::from_MB(512),
        },
        additional: [],
    },

    data_placement: DataPlacement {
        kernel: "RAM",
        processes: "RAM",
        shared: "RAM",
    },
};
