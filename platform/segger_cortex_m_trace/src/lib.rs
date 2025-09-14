#![no_std]

use hal::pac::Peripherals;
use hal::prelude::*;
use hal::time::Hertz;
use stm32f4xx_hal as hal;

pub struct SeggerCortexMTrace {}

impl SeggerCortexMTrace {
    pub fn new() -> Self {
        let stm32_peripherals = Peripherals::take().expect("cannot take stm32 peripherals");

        /* system clock */
        let rcc = stm32_peripherals.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(Hertz::MHz(48)).freeze();

        /* gpio's */
        let _gpioa = stm32_peripherals.GPIOA.split();
        let _gpiob = stm32_peripherals.GPIOB.split();
        let _gpioc = stm32_peripherals.GPIOC.split();

        /* assemble... */
        SeggerCortexMTrace {}
    }
}
