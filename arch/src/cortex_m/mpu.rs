//! ARM Cortex-M Memory Protection Unit.
//!
//! Based on <https://github.com/helium/cortex-mpu>.

use cortex_m::peripheral::{self, mpu, MPU};
use cortex_m::asm;

/// Valid sizes for the MPU.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Size {
    S32 = 4,
    S64 = 5,
    S128 = 6,
    S256 = 7,
    S512 = 8,
    S1K = 9,
    S2K = 10,
    S4K = 11,
    S8K = 12,
    S16K = 13,
    S32K = 14,
    S64K = 15,
    S128K = 16,
    S256K = 17,
    S512K = 18,
    S1M = 19,
    S2M = 20,
    S4M = 21,
    S8M = 22,
    S16M = 23,
    S32M = 24,
    S64M = 25,
    S128M = 26,
    S256M = 27,
    S512M = 28,
    S1G = 29,
    S2G = 30,
    S4G = 31,
}
impl Size {
    /// Return register value from `Size`.
    pub const fn bits(self) -> u32 {
        self as u32
    }

    /// Return `Size` in bytes
    pub const fn size_bytes(self) -> u32 {
        2u32.pow((self as u32) + 1)
    }
}

/* Control Register */
pub const MPU_ENABLE: u32 = 1;
pub const MPU_HARD_FAULT_ENABLED: u32 = 1 << 1;
pub const MPU_PRIVILEGED_DEFAULT_ENABLE: u32 = 1 << 2;

/* Region Number Register */
pub enum RegionNumber {
    Ignore,
    Use(u8),
}

/* Region Base Address Register */
pub const MPU_REGION_VALID: u32 = 1 << 4;

/* Region Attribute and Status Register */
pub const MPU_REGION_ENABLE: u32 = 1;

pub enum Permission {
    NoAccess,
    ReadOnly,
    ReadWrite,
}

impl From<crate::memory_protection::Permission> for Permission {
    /// Match permission from interface to local permission enum
    fn from(permission: crate::memory_protection::Permission) -> Self {
        match permission {
            crate::memory_protection::Permission::NoAccess => Permission::NoAccess,
            crate::memory_protection::Permission::ReadOnly => Permission::ReadOnly,
            crate::memory_protection::Permission::ReadWrite => Permission::ReadWrite,
        }
    }
}

/// Memory attributes on the hardware.
pub enum Attributes {
    /// No caching or buffering allowed
    StronglyOrdered,
    /// Memory mapped peripheral.
    Device {
        /// Can be shared between multiple cores.
        shareable: bool,
    },
    /// Normal memory, e.g. Flash or SRAM.
    Normal {
        /// Can be shared between multiple cores.
        shareable: bool,
        /// (inner, outer)
        cache_policy: (CachePolicy, CachePolicy),
    },
}

/// Requested cache policy if implemented on MCU.
pub enum CachePolicy {
    /// No caching allowed.
    NoCache,
    /// Write without caching, allow read caching.
    WriteThrough,
    /// Allow read and write caching. Cache will write back to main memory on
    /// its own.
    WriteBack {
        /// Write allocate: fetch data on a write miss to the cache.
        wa: bool,
    },
}

/// MPU subregions.
///
/// A memory region is divided into 8 subregion of equal size. The memory region
/// rule can be disabled for any of the 8 subregions.
/// A bit corresponds to one subregion: LSB - lowest address, MSB - highest
/// address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Subregions(u8);
impl Subregions {
    /// Memory rule applies to all subregions.
    pub const ALL: Subregions = Subregions(0xFF);
    /// All subregions disabled.
    ///
    /// **Note:** Just disable the memory region altogether.
    pub const NONE: Subregions = Subregions(0);

    pub const fn bits(self) -> u32 {
        !self.0 as u32
    }
}

impl Default for Subregions {
    fn default() -> Self {
        Subregions::ALL
    }
}

/// Raw register values, for fast MPU updates.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemoryRegion {
    pub region_base_address_reg: u32,
    pub region_attribute_size_reg: u32,
}

pub struct Mpu<'a>(&'a mut mpu::RegisterBlock);

impl Mpu<'_> {
    /// Get the memory protection peripheral.
    #[inline]
    pub unsafe fn take() -> Self {
        Self(&mut *(peripheral::MPU::PTR as *mut _))
    }

    /// Global MPU enable.
    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            self.0.ctrl.write(MPU_ENABLE | MPU_PRIVILEGED_DEFAULT_ENABLE);
        }
        asm::dsb();
        asm::isb();
    }

    /// Global MPU disable.
    #[inline]
    pub fn disable(&mut self) {
        asm::dmb();
        unsafe {
            self.0.ctrl.write(0);
        }
    }

    /// Compile RBAR register values from configuration.
    pub const fn prepare_region_base_address(addr: u32, region: RegionNumber) -> u32 {
        let base_addr = addr & !0x1F;
        let (valid, region) = match region {
            RegionNumber::Ignore => (0, 0),
            RegionNumber::Use(region) => (MPU_REGION_VALID, region),
        };

        base_addr | valid | region as u32
    }

    /// Apply memory base address and region.
    #[inline]
    pub fn set_region_base_address(&mut self, addr: u32, region: RegionNumber) {
        let register = Self::prepare_region_base_address(
            addr,
            region
        );

        unsafe {
            self.0.rbar.write(register);
        }
    }

    /// Compile RASR register values from configuration.
    pub const fn prepare_region_attributes(executable: bool,
                                     access: (Permission, Permission),
                                     attributes: Attributes,
                                     subregions: Subregions,
                                     region_size: Size) -> u32 {

        // (privileged, unprivileged)
        let ap = match access {
            (Permission::NoAccess, Permission::NoAccess) => 0b000,
            (Permission::ReadWrite, Permission::NoAccess) => 0b001,
            (Permission::ReadWrite, Permission::ReadOnly) => 0b010,
            (Permission::ReadWrite, Permission::ReadWrite) => 0b011,
            (Permission::ReadOnly, Permission::NoAccess) => 0b101,
            (Permission::ReadOnly, Permission::ReadOnly) => 0b111,
            (_, _) => 0b000, // no access
        };

        let (tex, c, b, s) = match attributes {
            Attributes::StronglyOrdered => (0b000, 0, 0, 0),
            Attributes::Device { shareable: true } => (0b000, 0, 1, 0),
            Attributes::Device { shareable: false } => (0b010, 0, 0, 0),
            Attributes::Normal { shareable, cache_policy } => match cache_policy {
                (CachePolicy::WriteThrough, CachePolicy::WriteThrough) => (0b000, 1, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteBack { wa: false }) => (0b000, 1, 1, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::NoCache) => (0b001, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: true }) => (0b001, 1, 1, shareable as u32),

                (CachePolicy::NoCache, CachePolicy::WriteBack { wa: true }) => (0b100, 0, 1, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::WriteThrough) => (0b100, 1, 0, shareable as u32),
                (CachePolicy::NoCache, CachePolicy::WriteBack { wa: false }) => (0b100, 1, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::NoCache) => (0b101, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteThrough) => (0b101, 1, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: true }, CachePolicy::WriteBack { wa: false }) => (0b101, 1, 1, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::NoCache) => (0b110, 0, 0, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::WriteBack { wa: true}) => (0b110, 0, 1, shareable as u32),
                (CachePolicy::WriteThrough, CachePolicy::WriteBack { wa: false}) => (0b110, 1, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::NoCache) => (0b111, 0, 0, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteBack { wa: true}) => (0b111, 0, 1, shareable as u32),
                (CachePolicy::WriteBack { wa: false }, CachePolicy::WriteThrough) => (0b111, 1, 0, shareable as u32),
            },
        };

        let register = (!executable as u32) << 28 |
            ap << 24 |
            tex << 19 | s << 18 | c << 17 | b << 16 |
            subregions.bits() << 8 |
            region_size.bits() << 1 |
            MPU_REGION_ENABLE;

        register
    }

    /// Apply memory region attributes and size.
    #[inline]
    pub fn set_region_attributes(&mut self,
                                 executable: bool,
                                 access: (Permission, Permission),
                                 attributes: Attributes,
                                 subregions: Subregions,
                                 region_size: Size) {

        let register = Self::prepare_region_attributes(
            executable,
            access,
            attributes,
            subregions,
            region_size
        );

        unsafe {
            self.0.rasr.write(register);
        }
    }

    /// Apply one precompiled region.
    pub fn set_region(&mut self, memory_region: &MemoryRegion) {
        unsafe {
            self.0.rbar.write(memory_region.region_base_address_reg);
            self.0.rasr.write(memory_region.region_attribute_size_reg);
        }
    }

    /// Apply 3 precompiled regions.
    pub fn set_regions(&mut self, memory_region: &[MemoryRegion; 3]) {
        unsafe {
            self.0.rbar_a1.write(memory_region[0].region_base_address_reg);
            self.0.rasr_a1.write(memory_region[0].region_attribute_size_reg);
            self.0.rbar_a2.write(memory_region[1].region_base_address_reg);
            self.0.rasr_a2.write(memory_region[1].region_attribute_size_reg);
            self.0.rbar_a3.write(memory_region[2].region_base_address_reg);
            self.0.rasr_a3.write(memory_region[2].region_attribute_size_reg);
        }
    }

    /// Disable one memory region.
    #[inline]
    pub fn disable_region(&mut self, region: u8) {
        unsafe {
            self.0.rnr.write(region as u32);
            self.0.rasr.write(0);
        }
    }
}