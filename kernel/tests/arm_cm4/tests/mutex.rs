#![no_main]
#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;
use alloc::sync::Arc;

mod common;

use bern_kernel::exec::process::Process;
use bern_kernel::sync::Mutex;

struct MyStruct {
    a: u32,
}

static PROC: &Process = bern_kernel::new_process!(test, 8192);

#[bern_test::tests]
mod tests {
    use bern_kernel::exec::runnable::Priority;
    use bern_kernel::exec::thread::Thread;
    use super::*;
    use crate::common::*;
    use bern_kernel::{sleep, time};
    use bern_kernel::units::frequency::*;
    use bern_kernel::stack::Stack;
    use bern_kernel::*;

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
        cortex_m::asm::bkpt();
    }

    #[test]
    fn wait_for_lock(_board: &mut Board) {
        let mutex = Arc::new(Mutex::new(MyStruct{ a: 42 }));

        PROC.init(move |c| {
            let mutex_1 = mutex.clone();
            // Taking a permit and blocking it
            Thread::new(c)
                .priority(Priority::new(1))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    match mutex_1.try_lock() {
                        Ok(mut value) => {
                            value.a = 54;
                            sleep(10);
                        },
                        Err(_) => panic!("Could not acquire mutex"),
                    }
                });
            // Wait for permit
            Thread::new(c)
                .priority(Priority::new(1))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    match mutex.lock(1000) {
                        Ok(value) => {
                            assert_eq!(value.a, 54);
                        },
                        Err(_) => panic!("Did not wait for mutex"),
                    }
                });

            // watchdog
            Thread::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(1000);

                    /* if the test does not fail within x time it succeeded */
                    bern_test::test_succeeded();
                    __tear_down();
                });
        }).ok();

        start();
    }

    // Priority inversion: increase priority of blocking task if a higher
    // priority task is waiting for a resource.
    //                          unlock  lock  unlock
    //                            ___|_|_____|_
    //  1      lock   T2 ready > |_T1_|_T2___ |_________ lock
    //  2   ___|__________________            |_T3____ |_|______
    //  3  |_T1__________________|                     |_T1_____
    #[test]
    fn priority_inversion() {
        let mutex = Arc::new(Mutex::new(MyStruct{ a: 42 }));

        PROC.init(move |c| {
            let mutex_1 = mutex.clone();
            // T1: low priority task
            Thread::new(c)
                .priority(Priority::new(3))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    match mutex_1.try_lock() {
                        Ok(_) => {
                            sleep(20);
                        },
                        Err(_) => panic!("Could not acquire mutex"),
                    }
                });

            let mutex_2 = mutex.clone();
            // T2: high priority task
            Thread::new(c)
                .priority(Priority::new(1))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(5); // let T1 start
                    match mutex_2.lock(1000) {
                        Ok(_) => sleep(100), // test finished successfully
                        Err(_) => panic!("Did not wait for mutex"),
                    }
                });

            // T3: interfering medium priority task
            Thread::new(c)
                .priority(Priority::new(2))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    sleep(10); // let T1 start
                    // preempt T1
                    panic!("T3 could preempt T1 within time frame");
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