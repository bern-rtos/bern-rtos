#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use st_nucleo_f446::StNucleoF446;
use rtos_trace::RtosTrace;
use systemview_target::{SystemView};

use st_nucleo_f446::hal::prelude::*;

use log::{info, warn, error, LevelFilter};

static SYSTEMVIEW: SystemView = SystemView::new();

#[entry]
fn main() -> ! {
    let board = StNucleoF446::new();
    let mut delay = board.delay;

    SYSTEMVIEW.init();
    log::set_logger(&SYSTEMVIEW).ok();
    log::set_max_level(LevelFilter::Info);

    SystemView::task_new(0);
    SystemView::task_exec_begin(0);

    let mut i = 0;
    loop {
        SystemView::task_exec_begin(0);
        info!("hello world {}", i);
        warn!("hello world");
        error!("hello world");
        SystemView::task_exec_end();

        i += 1;
        delay.delay_ms(100_u16);
    }
}


rtos_trace::global_application_callbacks!{Application}
struct Application;

impl rtos_trace::RtosTraceApplicationCallbacks for Application {
    fn system_description() {
        systemview_target::send_system_desc_app_name!("SystemView no-os example");
    }

    fn sysclock() -> u32 {
        72_000_000
    }
}