#![no_main]
#![no_std]
#![allow(unused_imports)]
#![feature(default_alloc_error_handler)]

mod common;
use common::main as _;
use bern_kernel::sync::Mutex;

extern crate alloc;
use alloc::sync::Arc;
use bern_kernel::exec::process::Process;

#[allow(unconditional_recursion)]
fn overflow_stack(a: u32) -> u32 {
    overflow_stack(a + 1)
}


static PROC: &Process = bern_kernel::new_process!(test, 4096);

#[bern_test::tests]
mod tests {
    use super::*;
    use crate::common::*;
    use bern_kernel::*;
    use bern_kernel::exec::runnable::Priority;
    use bern_kernel::exec::thread::Thread;
    use bern_kernel::stack::Stack;
    use bern_kernel::units::frequency::ExtMilliHertz;

    #[test_set_up]
    fn init_scheduler() {
        init();
        time::set_tick_frequency(
            1.kHz(),
            72.MHz()
        );
    }

    #[test_tear_down]
    fn reset() {
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[tear_down]
    fn stop() {
        //cortex_m::asm::bkpt();
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[test]
    fn normal_operation() {
        PROC.init(move |c| {
            Thread::new(c)
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(500);
                });

            // watchdog
            Thread::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(1000);

                    // if the test does not fail within x time it succeeded
                    bern_test::test_succeeded();
                    __tear_down();
                });
        }).unwrap();

        start();
    }

    #[test]
    fn gpio_access(board: &mut Board) {
        let mut led = board.led.take().unwrap();

        PROC.init(move |c| {
            Thread::new(c)
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    loop {
                        led.toggle();
                        sleep(10);
                    }
                });

            // watchdog
            Thread::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(1000);

                    // if the test does not fail within x time it succeeded
                    bern_test::test_succeeded();
                    __tear_down();
                });
        }).unwrap();

        start();
    }

    #[test]
    fn stack_overflow(_: &mut Board) {
        PROC.init(move |c| {
            Thread::new(c)
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    overflow_stack(0);
                    __tear_down();
                });

            // watchdog
            Thread::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(1000);

                    // if the test does not fail within x time it succeeded
                    bern_test::test_succeeded();
                    __tear_down();
                });
        }).unwrap();

        start();
    }


}