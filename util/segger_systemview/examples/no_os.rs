#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use core::fmt::Write;

use st_nucleo_f446::StNucleoF446;
use segger_systemview::{SystemView, info, warn, error};

use st_nucleo_f446::hal::prelude::*;

#[entry]
fn main() -> ! {
    let board = StNucleoF446::new();
    let mut delay = board.delay;

    let _systemview = SystemView::new();

    loop {
        info!("hello world");
        warn!("hello world");
        error!("hello world");
        delay.delay_ms(10_u16);
    }
}


#[allow(non_snake_case)]
#[no_mangle]
pub fn SystemCoreClock() -> cty::c_uint {
    72_000_000
}