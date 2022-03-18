//! System time.

use core::sync::atomic::{AtomicU32, Ordering};
use bern_units::frequency::Hertz;
use crate::sched;

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
#[inline(always)]
fn system_tick_update() {
    if TICK_LOW.load(Ordering::Relaxed) == u32::MAX {
        TICK_HIGH.fetch_add(1, Ordering::Relaxed);
    }
    TICK_LOW.fetch_add(1, Ordering::Relaxed);

    sched::tick_update();
}

/// Get the current system time in ticks.
pub fn tick() -> u64 {
    let tick;
    loop {
        let high = TICK_HIGH.load(Ordering::Relaxed);
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
    where Hertz: From<T> + From<S>
{
    let divisor = Hertz::from(sysclock).0 / Hertz::from(tick_frequency).0;
    TICK_PER_MS.store(divisor, Ordering::Relaxed);
    sched::update_tick_frequency(divisor);
}

#[cfg(feature = "log-defmt")]
defmt::timestamp!("{=u64}", tick());