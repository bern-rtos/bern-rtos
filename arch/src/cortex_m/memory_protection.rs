//! ARM Cortex-M implementation of [`IMemoryProtection`].

use crate::memory_protection::{IMemoryProtection, Config, Type};
use crate::arch::Arch;
use crate::arch::mpu::{self, Mpu, RegionNumber, Permission, Subregions, Attributes, CachePolicy};
pub use crate::arch::mpu::{Size, MemoryRegion};

use cortex_m::asm;
use cortex_m_rt::exception;

impl IMemoryProtection for Arch {
    type Size = Size;
    type MemoryRegion = MemoryRegion;

    fn enable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.enable();
    }

    fn disable_memory_protection() {
        let mut mpu =  unsafe{ Mpu::take() };
        mpu.disable();
    }

    fn enable_memory_region(region: u8, config: Config<Size>) {
        let mut mpu =  unsafe{ Mpu::take() };

        let memory_region = Self::prepare_memory_region(region, config);
        mpu.set_region(&memory_region);
    }

    fn disable_memory_region(region: u8) {
        let mut mpu = unsafe { Mpu::take() };
        mpu.disable_region(region);
    }

    fn prepare_memory_region(region: u8, config: Config<Self::Size>) -> Self::MemoryRegion {
        let region_base_address = Mpu::prepare_region_base_address(
            config.addr as u32,
            RegionNumber::Use(region)
        );

        let attributes = match config.memory {
            Type::SramInternal => Attributes::Normal {
                shareable: true,
                cache_policy: (CachePolicy::WriteThrough, CachePolicy::WriteThrough),
            },
            Type::SramExternal => Attributes::Normal {
                shareable: true,
                cache_policy: (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: true }),
            },
            Type::Flash => Attributes::Normal {
                shareable: false,
                cache_policy: (CachePolicy::WriteThrough, CachePolicy::WriteThrough),
            },
            Type::Peripheral => Attributes::Device {
                shareable: true
            }
        };

        let region_attributes = Mpu::prepare_region_attributes(
            config.executable,
            (Permission::from(config.access.system), Permission::from(config.access.user)),
            attributes,
            Subregions::ALL,
            config.size,
        );

        MemoryRegion {
            region_base_address_reg: region_base_address,
            region_attribute_size_reg: region_attributes,
        }
    }

    fn prepare_unused_region(region: u8) -> Self::MemoryRegion {
        MemoryRegion {
            region_base_address_reg: mpu::MPU_REGION_VALID | region as u32,
            region_attribute_size_reg: 0, // disable region
        }
    }

    fn apply_regions(memory_regions: &[MemoryRegion; 3]) {
        let mut mpu = unsafe { Mpu::take() };

        mpu.set_regions(memory_regions);
    }
}


extern "Rust" {
    /// Exception called on a memory protection violation.
    ///
    /// **Note:** Must be implemented in the kernel.
    pub fn memory_protection_exception();
}

#[allow(non_snake_case)]
#[exception]
fn MemoryManagement() -> () {
    unsafe {
        memory_protection_exception();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[repr(align(32))]
pub struct A32;

#[repr(align(64))]
pub struct A64;

#[repr(align(128))]
pub struct A128;

#[repr(align(256))]
pub struct A256;

#[repr(align(512))]
pub struct A512;

#[repr(align(1_024))]
pub struct A1K;

#[repr(align(2_048))]
pub struct A2K;

#[repr(align(4_096))]
pub struct A4K;

#[repr(align(8_192))]
pub struct A8K;

#[repr(align(16_384))]
pub struct A16K;

#[repr(align(32_768))]
pub struct A32K;

#[repr(align(65_536))]
pub struct A64K;

#[repr(align(131_072))]
pub struct A128K;

#[repr(align(262_144))]
pub struct A256K;

#[repr(align(524_288))]
pub struct A512K;

#[repr(align(1_048_576))]
pub struct A1M;

#[repr(align(2_097_152))]
pub struct A2M;

#[repr(align(4_194_304))]
pub struct A4M;

#[repr(align(8_388_608))]
pub struct A8M;

#[repr(align(16_777_216))]
pub struct A16M;

#[repr(align(33_554_432))]
pub struct A32M;

#[repr(align(67_108_864))]
pub struct A64M;

#[repr(align(134_217_728))]
pub struct A128M;

#[repr(align(268_435_456))]
pub struct A256M;

#[repr(align(536_870_912))]
pub struct A512M;

/// Return a valid memory protection alignment object from size in bytes.
#[macro_export]
macro_rules! alignment_from_size {
    (32) => { $crate::arch::memory_protection::A32 };
    (64) => { $crate::arch::memory_protection::A64 };
    (128) => { $crate::arch::memory_protection::A128 };
    (256) => { $crate::arch::memory_protection::A256 };
    (512) => { $crate::arch::memory_protection::A512 };
    (1_024) => { $crate::arch::memory_protection::A1K };
    (1024) => { $crate::arch::memory_protection::A1K };
    (2_048) => { $crate::arch::memory_protection::A2K };
    (2048) => { $crate::arch::memory_protection::A2K };
    (4_096) => { $crate::arch::memory_protection::A4K };
    (4096) => { $crate::arch::memory_protection::A4K };
    (8_192) => { $crate::arch::memory_protection::A8K };
    (8192) => { $crate::arch::memory_protection::A8K };
    (16_384) => { $crate::arch::memory_protection::A16K };
    (16384) => { $crate::arch::memory_protection::A16K };
    (32_768) => { $crate::arch::memory_protection::A32K };
    (32768) => { $crate::arch::memory_protection::A32K };
    (65_536) => { $crate::arch::memory_protection::A64K };
    (65536) => { $crate::arch::memory_protection::A64K };
    (131_072) => { $crate::arch::memory_protection::A128K };
    (131072) => { $crate::arch::memory_protection::A128K };
    (262_144) => { $crate::arch::memory_protection::A256K };
    (262144) => { $crate::arch::memory_protection::A256K };
    (524_288) => { $crate::arch::memory_protection::A512K };
    (524288) => { $crate::arch::memory_protection::A512K };
    (1_048_576) => { $crate::arch::memory_protection::A1M };
    (1048576) => { $crate::arch::memory_protection::A1M };
    ($x:expr) => {
        compile_error!("Size does not meet alignment requirements from the MPU. \
        Compatible sizes are: 32B, 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB, 16KB, \
        32KB, 64KB, 128KB, 256KB, 512KB, 1M");
    };
}

/// Return a valid memory protection size object from size in bytes.
#[macro_export]
macro_rules! size_from_raw {
    (32) => { $crate::arch::memory_protection::Size::S32 };
    (64) => { $crate::arch::memory_protection::Size::S64 };
    (128) => { $crate::arch::memory_protection::Size::S128 };
    (256) => { $crate::arch::memory_protection::Size::S256 };
    (512) => { $crate::arch::memory_protection::Size::S512 };
    (1_024) => { $crate::arch::memory_protection::Size::S1K };
    (1024) => { $crate::arch::memory_protection::Size::S1K };
    (2_048) => { $crate::arch::memory_protection::Size::S2K };
    (2048) => { $crate::arch::memory_protection::Size::S2K };
    (4_096) => { $crate::arch::memory_protection::Size::S4K };
    (4096) => { $crate::arch::memory_protection::Size::S4K };
    (8_192) => { $crate::arch::memory_protection::Size::S8K };
    (8192) => { $crate::arch::memory_protection::Size::S8K };
    (16_384) => { $crate::arch::memory_protection::Size::S16K };
    (16384) => { $crate::arch::memory_protection::Size::S16K };
    (32_768) => { $crate::arch::memory_protection::Size::S32K };
    (32768) => { $crate::arch::memory_protection::Size::S32K };
    (65_536) => { $crate::arch::memory_protection::Size::S64K };
    (65536) => { $crate::arch::memory_protection::Size::S64K };
    (131_072) => { $crate::arch::memory_protection::Size::S128K };
    (131072) => { $crate::arch::memory_protection::Size::S128K };
    (262_144) => { $crate::arch::memory_protection::Size::S256K };
    (262144) => { $crate::arch::memory_protection::Size::S256K };
    (524_288) => { $crate::arch::memory_protection::Size::S512K };
    (524288) => { $crate::arch::memory_protection::Size::S512K };
    (1_048_576) => { $crate::arch::memory_protection::Size::S1M };
    (1048576) => { $crate::arch::memory_protection::Size::S1M };
    ($x:expr) => {
        compile_error!("Size cannot be protected by MPU. \
        Compatible sizes are: 32B, 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB, 16KB, \
        32KB, 64KB, 128KB, 256KB, 512KB, 1M");
    };
}