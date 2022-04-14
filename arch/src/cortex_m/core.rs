//! ARM Cortex-M implementation of [`ICore`].

use crate::core::{ExecMode, ICore};
use cortex_m::{Peripherals, asm};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::scb;

pub struct ArchCore {
    peripherals: Peripherals,
}

impl ICore for ArchCore {
    fn new() -> Self {
        // NOTE(unsafe): we must be able to take the peripherals or else the
        // system is doomed
        let mut peripherals = unsafe { Peripherals::steal() };
        peripherals.SYST.set_clock_source(SystClkSource::Core);

        ArchCore {
            peripherals
        }
    }

    fn set_systick_div(&mut self, divisor: u32) {
        self.peripherals.SYST.set_reload(divisor - 1);
        self.peripherals.SYST.clear_current();
    }


    fn start(&mut self) {
        self.peripherals.SYST.enable_counter();
        self.peripherals.SYST.enable_interrupt();

        // enable PendSV interrupt on lowest priority
        unsafe {
            self.peripherals.SCB.set_priority(scb::SystemHandler::PendSV, 0xFF);
        }
        // todo: move
        self.peripherals.SCB.enable(scb::Exception::MemoryManagement);
    }

    fn bkpt() {
        asm::bkpt();
    }

    fn execution_mode() -> ExecMode {
        if cortex_m::register::control::read().spsel().is_msp() {
            ExecMode::Kernel
        } else {
            ExecMode::Thread
        }
    }
}