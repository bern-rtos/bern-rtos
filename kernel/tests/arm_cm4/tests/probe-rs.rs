#![no_main]
#![no_std]

//use cortex_m::iprintln;
use core::panic::PanicInfo;
use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, stm32};

// todo: abstract println backends
use rtt_target::{rprintln, rtt_init, set_print_channel};

#[cortex_m_rt::entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 512
                mode: BlockIfFull
                name: "Terminal"
            }
            1: {
                size: 512
                mode: BlockIfFull
                name: "Control Up"
            }
        }
        down: {
            0: {
                size: 512
                mode: BlockIfFull
                name: "Terminal"
            }
            1: {
                size: 512
                mode: BlockIfFull
                name: "Control Down"
            }
        }
    };
    set_print_channel(channels.up.0);

    let mut output = channels.up.1;
    let mut input = channels.down.1;
    let mut buf = [0u8; 512];
    let mut count: u8 = 0;
    loop {
        let bytes = input.read(&mut buf[..]);
        if bytes > 0 {
            for c in buf.iter_mut() {
                c.make_ascii_uppercase();
            }

            let mut p = 0;
            while p < bytes {
                p += output.write(&buf[p..bytes]);
            }
        }

        rprintln!("Messsge no. {}/{}", count, bytes);

        count += 1;

        for _ in 0..1_000_000 {
            cortex_m::asm::nop();
        }
    }

    rprintln!("Running test...");

    tests::example();

    loop {
        cortex_m::asm::bkpt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    cortex_m::asm::bkpt();
    loop {}
}

mod tests {
    pub fn example() {
        assert!(1 == 0, "wrong");
    }
}