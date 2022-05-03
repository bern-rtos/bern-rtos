#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use core::fmt::Write;

use st_nucleo_f446::StNucleoF446;
use rtos_trace::RtosTrace;
use systemview_target::{SystemView, info, warn, error};

use st_nucleo_f446::hal::prelude::*;


#[entry]
fn main() -> ! {
    let board = StNucleoF446::new();
    let mut delay = board.delay;

    SystemView::init();

    SystemView::task_new(0);
    SystemView::task_exec_begin(0);

    loop {
        SystemView::task_exec_begin(0);
        info!("hello world");
        warn!("hello world");
        error!("hello world");
        SystemView::task_exec_end();
        delay.delay_ms(100_u16);
    }
}


#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn SystemCoreClock() -> cty::c_uint {
    72_000_000
}