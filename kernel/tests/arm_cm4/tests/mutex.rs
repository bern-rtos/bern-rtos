#![no_main]
#![no_std]

mod common;
use common::main as _;
use bern_kernel::sync::Mutex;

struct MyStruct {
    a: u32,
}

#[link_section = ".shared"]
static MUTEX: Mutex<MyStruct> = Mutex::new(MyStruct{ a: 42 });

#[bern_test::tests]
mod tests {
    use super::*;
    use crate::common::*;
    use bern_kernel as kernel;
    use kernel::sched;
    use kernel::task::{Task, Priority};

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        /* idle task */
        Task::new()
            .idle_task()
            .static_stack(kernel::alloc_static_stack!(128))
            .spawn(move || {
                loop {
                    cortex_m::asm::nop();
                }
            });

        /* watchdog */
        Task::new()
            .priority(Priority(0))
            .static_stack(kernel::alloc_static_stack!(512))
            .spawn(move || {
                kernel::sleep(100); // todo: replace with event

                /* if the test does not fail within x time it succeeded */
                bern_test::test_succeeded();
                __test_tear_down();
            });
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
    fn not_registered() {
        Task::new()
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match MUTEX.lock(1000) {
                    Ok(_) => panic!("Mutex wasn't registered, should have failed"),
                    Err(_) => (),
                }
            });

        sched::start();
    }

    #[test]
    fn wait_for_lock(_board: &mut Board) {
        MUTEX.register().ok();
        /* Taking a permit and blocking it */
        Task::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match MUTEX.try_lock() {
                    Ok(mut value) => {
                        value.a = 54;
                        kernel::sleep(10);
                    },
                    Err(_) => panic!("Could not acquire mutex"),
                }
            });
        /* Wait for permit */
        Task::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match MUTEX.lock(1000) {
                    Ok(value) => {
                        assert_eq!(value.a, 54);
                    },
                    Err(_) => panic!("Did not wait for mutex"),
                }
            });

        sched::start();
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
        MUTEX.register().ok();
        /* T1: low priority task */
        Task::new()
            .priority(Priority(3))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match MUTEX.try_lock() {
                    Ok(_) => {
                        kernel::sleep(20);
                    },
                    Err(_) => panic!("Could not acquire mutex"),
                }
            });
        /* T2: high priority task */
        Task::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                kernel::sleep(5); // let T1 start
                match MUTEX.lock(1000) {
                    Ok(_) => kernel::sleep(100), // test finished successfully
                    Err(_) => panic!("Did not wait for mutex"),
                }
            });

        /* T3: interfering medium priority task */
        Task::new()
            .priority(Priority(2))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                kernel::sleep(10); // let T1 start
                // preempt T1
                panic!("T3 could preempt T1 within time frame");
            });

        sched::start();
    }
}