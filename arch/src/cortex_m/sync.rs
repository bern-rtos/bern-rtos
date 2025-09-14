//! ARM Cortex-M implementation of [`ISync`].

use crate::cortex_m::Arch;
use crate::sync::ISync;

impl ISync for Arch {
    #[allow(unused_variables)]
    fn disable_interrupts(priority: usize) {
        // todo: only mask interrupts up to a certain priority
        cortex_m::interrupt::disable();
    }

    fn enable_interrupts() {
        unsafe {
            cortex_m::interrupt::enable();
        }
    }
}
