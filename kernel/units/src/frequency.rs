#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MilliHertz(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Hertz(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct KiloHertz(pub u32);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MegaHertz(pub u32);


/// MilliHertz is implemented specifically because const trait implementations are not
/// stable yet.
#[allow(non_snake_case)]
impl MilliHertz {
    pub const fn from_Hz(hz: u32) -> Self {
        MilliHertz(hz * 1_000)
    }

    pub const fn from_kHz(khz: u32) -> Self {
        MilliHertz(khz * 1_000 * 1_000)
    }

    pub const fn from_MHz(mhz: u32) -> Self {
        MilliHertz(mhz * 1_000 * 1_000 * 1_000)
    }
}

/// Extension trait that adds convenience methods to the `u32` type
#[allow(non_snake_case)]
pub trait ExtMilliHertz {
    /// Wrap in `MilliHertz`
    fn mHz(self) -> MilliHertz;

    /// Wrap in `Hertz`
    fn Hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn kHz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn MHz(self) -> MegaHertz;
}

#[allow(non_snake_case)]
impl ExtMilliHertz for u32 {
    fn mHz(self) -> MilliHertz {
        MilliHertz(self)
    }

    fn Hz(self) -> Hertz {
        Hertz(self)
    }

    fn kHz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn MHz(self) -> MegaHertz {
        MegaHertz(self)
    }
}


impl From<u32> for MilliHertz {
    fn from(mhz: u32) -> Self {
        mhz.mHz()
    }
}


impl From<Hertz> for MilliHertz {
    fn from(hz: Hertz) -> Self {
        Self(hz.0 * 1_000)
    }
}


impl From<KiloHertz> for MilliHertz {
    fn from(khz: KiloHertz) -> Self {
        Self(khz.0 * 1_000 * 1_000)
    }
}

impl From<KiloHertz> for Hertz {
    fn from(khz: KiloHertz) -> Self {
        Self(khz.0 * 1_000)
    }
}


impl From<MegaHertz> for MilliHertz {
    fn from(mhz: MegaHertz) -> Self {
        Self(mhz.0 * 1_000 * 1_000 * 1_000)
    }
}

impl From<MegaHertz> for Hertz {
    fn from(mhz: MegaHertz) -> Self {
        Self(mhz.0 * 1_000 * 1_000)
    }
}

impl From<MegaHertz> for KiloHertz {
    fn from(mhz: MegaHertz) -> Self {
        Self(mhz.0 * 1_000)
    }
}
