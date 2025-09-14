//! System time.

use crate::sched;
use bern_units::frequency::Hertz;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};
use embedded_time::clock::Error;
use embedded_time::duration::Fraction;
use embedded_time::Instant;

#[link_section = ".kernel"]
static TICK_PER_MS: AtomicU32 = AtomicU32::new(0);

/// Upper 32 bit of system tick.
#[link_section = ".kernel"]
static TICK_HIGH: AtomicU32 = AtomicU32::new(0);
/// Lower 32 bit of system tick.
#[link_section = ".kernel"]
static TICK_LOW: AtomicU32 = AtomicU32::new(0);

/// Update system tick count by adding 1.
///
/// **Note:** This function must be called from the architecture implementation.
#[no_mangle]
fn system_tick_update() {
    if TICK_LOW.load(Ordering::Acquire) == u32::MAX {
        TICK_HIGH.fetch_add(1, Ordering::Relaxed);
    }
    TICK_LOW.fetch_add(1, Ordering::Relaxed);

    sched::tick_update();
}

/// Get the current system time in ticks.
pub(crate) fn tick_count() -> u64 {
    let tick;
    loop {
        let high = TICK_HIGH.load(Ordering::Acquire);
        let low = TICK_LOW.load(Ordering::Relaxed);

        // check if high count was updated inbetween
        if high == TICK_HIGH.load(Ordering::Relaxed) {
            tick = (high as u64) << 32 | (low as u64);
            break;
        }
    }

    tick
}

pub fn set_tick_frequency<T, S>(tick_frequency: T, sysclock: S)
where
    Hertz: From<T> + From<S>,
{
    let divisor = Hertz::from(sysclock).0 / Hertz::from(tick_frequency).0;
    TICK_PER_MS.store(divisor, Ordering::Relaxed);
    sched::update_tick_frequency(divisor);
}

#[derive(Copy, Clone)]
pub struct SysClock<T> {
    _marker: PhantomData<T>,
}

impl<T> SysClock<T> {
    pub fn new() -> SysClock<T> {
        SysClock {
            _marker: Default::default(),
        }
    }
}

impl embedded_time::Clock for SysClock<u64> {
    type T = u64;
    // todo: make systick frequency const
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);

    fn try_now(&self) -> Result<Instant<Self>, Error> {
        Ok(Instant::new(crate::syscall::tick_count()))
    }
}

impl embedded_time::Clock for SysClock<u32> {
    type T = u32;
    // todo: make systick frequency const
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1_000);

    fn try_now(&self) -> Result<Instant<Self>, Error> {
        Ok(Instant::new(crate::syscall::tick_count() as u32))
    }
}

#[cfg(feature = "log-defmt")]
defmt::timestamp!("{=u64}", crate::syscall::tick_count());
