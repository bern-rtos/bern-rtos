#![no_std]
#![no_main]

use cortex_m;
use cortex_m_rt::entry;
use panic_halt as _;

use st_nucleo_f446::StNucleoF446;
use segger_systemview::SystemView;

#[entry]
fn main() -> ! {
    let _board = StNucleoF446::new();

    let _systemview = SystemView::new();

    loop {


    }
}


#[allow(non_snake_case)]
#[no_mangle]
pub fn SystemCoreClock() -> cty::c_uint {
    72_000_000
}