#![no_main]
#![no_std]

use bern_kernel as kernel;
use kernel::{
    sched,
    sync::mutex::Mutex,
};

use panic_halt as _;

use cortex_m;
use cortex_m_rt::entry;
use segger_cortex_m_trace::SeggerCortexMTrace;
use stm32f4xx_hal::prelude::*;
use bern_kernel::exec::thread::{Priority, Runnable};
use bern_kernel::sync::semaphore::Semaphore;

#[link_section = ".shared"]
static MUTEX: Mutex<u32> = Mutex::new(42);
#[link_section = ".shared"]
static SEMAPHORE: Semaphore = Semaphore::new(4);

#[entry]
fn main() -> ! {
    let board = SeggerCortexMTrace::new();

    sched::init();
    MUTEX.register().ok();
    SEMAPHORE.register().ok();

    /* idle task */
    Runnable::new()
        .idle_task()
        .static_stack(kernel::alloc_static_stack!(128))
        .spawn(move || {
            loop {
                cortex_m::asm::nop();
            }
        });

    /* task 1 */
    Runnable::new()
        .priority(Priority(1))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                {
                    match MUTEX.lock(1000) {
                        Ok(mut value) => *value = 54,
                        Err(_) => (),
                    }
                }
                kernel::sleep(100);
            }
        });

    /* task 2 */
    Runnable::new()
        .priority(Priority(3))
        .static_stack(kernel::alloc_static_stack!(1024))
        .spawn(move || {
            /* spawn a new task while the system is running */
            Runnable::new()
                .static_stack(kernel::alloc_static_stack!(512))
                .spawn(move || {
                    loop {
                        kernel::sleep(800);
                    }
                });

            loop {
                match MUTEX.try_lock() {
                    Ok(_) => kernel::sleep(500),
                    Err(_) => (),
                }
                kernel::sleep(1000);
            }
        });


    let mut a = 10;
    Runnable::new()
        .priority(Priority(3))
        .static_stack(kernel::alloc_static_stack!(512))
        .spawn(move || {
            loop {
                a += 1;
                kernel::sleep(50);
                kernel::sleep(150);

                if a >= 60 {
                    let perm0 = SEMAPHORE.acquire(100);
                    let perm1 = SEMAPHORE.acquire(100);
                    let perm2 = SEMAPHORE.acquire(100);
                    let perm3 = SEMAPHORE.acquire(100);
                    let perm4 = SEMAPHORE.acquire(100);
                    core::mem::drop(perm0.ok().unwrap());
                    core::mem::drop(perm1.ok().unwrap());
                    core::mem::drop(perm2.ok().unwrap());
                    core::mem::drop(perm3.ok().unwrap());
                    core::mem::drop(perm4.ok().unwrap());
                    //kernel::task_exit();
                }
            }
        });

    /* blocking task */
    Runnable::new()
        .priority(Priority(4))
        .static_stack(kernel::alloc_static_stack!(128))
        .spawn(move || {
            loop {
                cortex_m::asm::nop();
            }
        });

    sched::start();
}
