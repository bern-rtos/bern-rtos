#![no_main]
#![no_std]

mod common;
use common::main as _;

fn recursion(i: u32) {
    recursion(i+1);
}

#[bern_test::tests]
mod tests {
    use crate::common::*;

    #[test_tear_down]
    fn reset() {
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[tear_down]
    fn stop() {
        cortex_m::asm::bkpt();
    }

    #[test]
    fn should_fail() {
        assert_eq!(1, 0);
    }

    #[test]
    fn with_board(board: &mut Board) {
        let mut led = board.led.take().unwrap();
        led.set_high().ok();
        board.shield.led_0.set_high().ok();
        assert_eq!(led.is_high().unwrap(), true);
    }

    #[test]
    #[ignore]
    fn stack_overflow() {
        super::recursion(0);
    }

    #[test]
    #[should_panic]
    fn should_panic() {
        assert_eq!(1, 0);
    }

    #[test]
    #[should_panic]
    fn should_panic_but_does_not() {
        assert_eq!(1, 1);
    }

    #[test]
    #[ignore]
    fn a_third_test() {
        assert_eq!(1, 1);
    }
}