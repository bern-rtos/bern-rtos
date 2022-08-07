//! Units representing time.

use derive_more::{Add, Sub, Mul, Div};

//#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
//pub struct MicroSecond(pub u64);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct MilliSecond(pub u64);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct Second(pub u64);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct Minute(pub u64);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct Hour(pub u64);

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Add, Sub, Mul, Div)]
pub struct Day(pub u64);

/// Second is implemented specifically because const trait implementations are not
/// stable yet.
impl MilliSecond {
    pub const fn from_s(s: u64) -> Self {
        MilliSecond(s * 1_000)
    }

    pub const fn from_min(min: u64) -> Self {
        MilliSecond(min * 60 * 1_000)
    }

    pub const fn from_h(h: u64) -> Self {
        MilliSecond(h * 60 * 60 * 1_000)
    }

    pub const fn from_d(d: u64) -> Self {
        MilliSecond(d * 24 * 60 * 60 * 1_000)
    }
}

/// Extension trait that adds convenience methods to the `u32` and `u64` type
#[allow(non_snake_case)]
pub trait ExtMilliSecond {
    // Wrap in `MilliSecond`
    fn ms(self) -> MilliSecond;

    /// Wrap in `Second`
    fn s(self) -> Second;

    /// Wrap in `Minute`
    fn min(self) -> Minute;

    /// Wrap in `Hour`
    fn h(self) -> Hour;

    /// Wrap in `Day`
    fn d(self) -> Day;
}

impl ExtMilliSecond for u32 {
    fn ms(self) -> MilliSecond {
        MilliSecond(self as u64)
    }

    fn s(self) -> Second {
        Second(self as u64)
    }

    fn min(self) -> Minute {
        Minute(self as u64)
    }

    fn h(self) -> Hour {
        Hour(self as u64)
    }

    fn d(self) -> Day {
        Day(self as u64)
    }
}

impl ExtMilliSecond for u64 {
    fn ms(self) -> MilliSecond {
        MilliSecond(self)
    }

    fn s(self) -> Second {
        Second(self)
    }

    fn min(self) -> Minute {
        Minute(self)
    }

    fn h(self) -> Hour {
        Hour(self)
    }

    fn d(self) -> Day {
        Day(self)
    }
}


impl From<u32> for MilliSecond {
    fn from(ms: u32) -> Self {
        (ms as u64).ms()
    }
}

impl From<u64> for MilliSecond {
    fn from(ms: u64) -> Self {
        ms.ms()
    }
}


impl From<Second> for MilliSecond {
    fn from(s: Second) -> Self {
        Self(s.0 * 1_000)
    }
}


impl From<Minute> for MilliSecond {
    fn from(min: Minute) -> Self {
        Self(min.0 * 60 * 1_000)
    }
}

impl From<Minute> for Second {
    fn from(min: Minute) -> Self {
        Self(min.0 * 60)
    }
}


impl From<Hour> for MilliSecond {
    fn from(h: Hour) -> Self {
        Self(h.0 * 60 * 60 * 1_000)
    }
}

impl From<Hour> for Second {
    fn from(h: Hour) -> Self {
        Self(h.0 * 60 * 60)
    }
}

impl From<Hour> for Minute {
    fn from(h: Hour) -> Self {
        Self(h.0 * 60)
    }
}


impl From<Day> for MilliSecond {
    fn from(d: Day) -> Self {
        Self(d.0 * 24 * 60 * 60 * 1_000)
    }
}

impl From<Day> for Second {
    fn from(d: Day) -> Self {
        Self(d.0 * 24 * 60 * 60)
    }
}

impl From<Day> for Minute {
    fn from(d: Day) -> Self {
        Self(d.0 * 24 * 60)
    }
}

impl From<Day> for Hour {
    fn from(d: Day) -> Self {
        Self(d.0 * 24)
    }
}
