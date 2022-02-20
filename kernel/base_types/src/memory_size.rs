#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Byte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct KiloByte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MegaByte(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct GigaByte(pub u32);

/// Byte is implemented specifically because const trait implementations are not
/// stable yet.
impl Byte {
    pub const fn from_kb(kb: u32) -> Self {
        Byte(kb * 1_024)
    }

    pub const fn from_mb(mb: u32) -> Self {
        Byte(mb * 1_024 * 1_024)
    }

    pub const fn from_gb(gb: u32) -> Self {
        Byte(gb * 1_024 * 1_024 * 1_024)
    }
}

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn b(self) -> Byte;

    /// Wrap in `Hertz`
    fn kb(self) -> KiloByte;

    /// Wrap in `KiloHertz`
    fn mb(self) -> MegaByte;

    /// Wrap in `MegaHertz`
    fn gb(self) -> GigaByte;
}

impl U32Ext for u32 {
    fn b(self) -> Byte {
        Byte (self)
    }

    fn kb(self) -> KiloByte {
        KiloByte(self)
    }

    fn mb(self) -> MegaByte {
        MegaByte(self)
    }

    fn gb(self) -> GigaByte {
        GigaByte(self)
    }
}

impl From<u32> for Byte {
    fn from(b: u32) -> Self {
        b.b()
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
