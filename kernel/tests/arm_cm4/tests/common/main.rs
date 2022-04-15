use core::sync::atomic::{self, Ordering};

use st_nucleo_f446::StNucleoF446 as Board;
use stm32f4xx_hal::prelude::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut board = Board::new();

    super::super::tests::runner(&mut board);

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}