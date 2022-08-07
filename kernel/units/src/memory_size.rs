//! Units representing memory sizes.

use derive_more::{Add, Sub, Mul, Div};

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct Byte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct KiloByte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct MegaByte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct GigaByte(pub u32);

/// Byte is implemented specifically because const trait implementations are not
/// stable yet.
#[allow(non_snake_case)]
impl Byte {
    pub const fn from_kB(kb: u32) -> Self {
        Byte(kb * 1_024)
    }

    pub const fn from_MB(mb: u32) -> Self {
        Byte(mb * 1_024 * 1_024)
    }

    pub const fn from_GB(gb: u32) -> Self {
        Byte(gb * 1_024 * 1_024 * 1_024)
    }
}

/// Extension trait that adds convenience methods to the `u32` type
#[allow(non_snake_case)]
pub trait ExtByte {
    /// Wrap in `Byte`
    fn B(self) -> Byte;

    /// Wrap in `KiloByte`
    fn kB(self) -> KiloByte;

    /// Wrap in `MegaByte`
    fn MB(self) -> MegaByte;

    /// Wrap in `GigaByte`
    fn GB(self) -> GigaByte;
}

#[allow(non_snake_case)]
impl ExtByte for u32 {
    fn B(self) -> Byte {
        Byte(self)
    }

    fn kB(self) -> KiloByte {
        KiloByte(self)
    }

    fn MB(self) -> MegaByte {
        MegaByte(self)
    }

    fn GB(self) -> GigaByte {
        GigaByte(self)
    }
}

impl Into<u32> for Byte {
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<usize> for Byte {
    fn into(self) -> usize {
        self.0 as usize
    }
}


impl From<u32> for Byte {
    fn from(b: u32) -> Self {
        b.B()
    }
}


impl From<KiloByte> for Byte {
    fn from(kb: KiloByte) -> Self {
        Self(kb.0 * 1_024)
    }
}


impl From<MegaByte> for Byte {
    fn from(mb: MegaByte) -> Self {
        Self(mb.0 * 1_024 * 1_024)
    }
}

impl From<MegaByte> for KiloByte {
    fn from(mb: MegaByte) -> Self {
        Self(mb.0 * 1_024)
    }
}


impl From<GigaByte> for Byte {
    fn from(gb: GigaByte) -> Self {
        Self(gb.0 * 1_024 * 1_024 * 1_024)
    }
}

impl From<GigaByte> for KiloByte {
    fn from(gb: GigaByte) -> Self {
        Self(gb.0 * 1_024 * 1_024)
    }
}

impl From<GigaByte> for MegaByte {
    fn from(gb: GigaByte) -> Self {
        Self(gb.0 * 1_024)
    }
}
