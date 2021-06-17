//! System time.

use core::sync::atomic::{AtomicU32, Ordering};
use crate::sched;

// todo: finish
pub(crate) struct Duration {
    ticks: u64
}

#[allow(dead_code)]
impl Duration {
    pub const fn from_millis(millis: u64) -> Self {
        Duration {
            ticks: millis, // todo: scale
        }
    }
    pub const fn infinite() -> Self {
        Duration {
            ticks: u64::MAX,
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }
}

/// Current system tick count.
static COUNT: AtomicU32 = AtomicU32::new(0); // todo: replace with u64

/// Update system tick count by adding 1.
///
/// **Note:** This function must be called from the architecture implementation.
#[no_mangle]
#[inline(always)]
fn system_tick_update() {
    COUNT.fetch_add(1, Ordering::Relaxed);
    sched::tick_update();
}

/// Get the current system time in ticks.
pub fn tick() -> u64 {
    COUNT.load(Ordering::Relaxed) as u64
}