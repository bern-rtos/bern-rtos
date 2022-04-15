//#![deny(unsafe_code)]
#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

use defmt_rtt as _;

#[allow(unused_imports)]
use bern_kernel::log::{debug, error, info, trace, warn};

use cortex_m;
use bern_kernel::bern_arch::cortex_m::cortex_m_rt::entry;
use st_nucleo_f446::StNucleoF446;

use bern_kernel::sync::{Mutex, Semaphore};

use core::sync::atomic;
use core::sync::atomic::Ordering;

extern crate alloc;
use alloc::sync::Arc;
use alloc::boxed::Box;
use stm32f4xx_hal::gpio::ExtiPin;
use bern_kernel::alloc::const_pool::ConstPool;
use bern_kernel::exec::interrupt::{InterruptHandler, InterruptStack};
use bern_kernel::exec::process::Process;
use bern_kernel::exec::runnable::Priority;
use bern_kernel::exec::thread::Thread;
use bern_kernel::exec::worker::{WorkItem, Workqueue};
use bern_kernel::stack::Stack;
use bern_kernel::units::frequency::ExtMilliHertz;

#[link_section=".process.my_process"]
static mut SOME_ARR: [u8; 8] = [1,2,3,4,5,6,7,8];

static PROC: &Process = bern_kernel::new_process!(my_process, 8192);

#[entry]
fn main() -> ! {
    let mut board = StNucleoF446::new();

    bern_kernel::init();
    bern_kernel::time::set_tick_frequency(
        1.kHz(),
        48.MHz()
    );

    info!("Bern RTOS example application");

    let mut heartbeat = board.led.take().unwrap();
    let mut button_0 = board.shield.button_0;
    let mut button_1 = board.shield.button_1;
    let mut button_7 = board.shield.button_7;
    let mut led_1 = board.shield.led_1;
    let mut led_7 = board.shield.led_7;

    PROC.init(move |c| {
        let asdf = Box::new(42);
        info!("{:x}", &asdf as *const _);

        let button_event = Arc::new(Semaphore::new(0));

        let worker = Workqueue::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .build();

        Thread::new(c)
            .priority(Priority::new(1))
            .stack(Stack::try_new_in(c, 1024).unwrap())
            .spawn(move || {
                let bla = Arc::new(Mutex::new(96));

                unsafe { SOME_ARR[0] += 1; }

                loop {
                    heartbeat.set_high();
                    bern_kernel::sleep(100);
                    heartbeat.set_low();
                    bern_kernel::sleep(900);

                    if let Ok(mut b) = bla.try_lock() {
                        *b += 1;
                        //info!("b = {}", *b);
                    };

                    bern_kernel::syscall::print_kernel_stats();
                    //recursion(1);
                }
            });

        let button_event_consumer = Arc::clone(&button_event);
        Thread::new(c)
            .priority(Priority::new(2))
            .stack(Stack::try_new_in(c, 1024).unwrap())
            .spawn(move || {
                loop {
                    if let Ok(p) = button_event_consumer.acquire(1000) {
                        info!("Processing button event from thread.");
                        led_7.toggle();
                        p.forget();
                    };
                }
            });

        Thread::new(c)
            .priority(Priority::new(2))
            .stack(Stack::try_new_in(c, 500).unwrap())
            .spawn(move || {
                recursion(0);
                loop {
                    //bern_kernel::sleep(1000);
                }
            });

        InterruptHandler::new(c)
            .stack(InterruptStack::Kernel)
            .connect_interrupt(stm32f4xx_hal::interrupt::EXTI9_5 as u16)
            .connect_interrupt(stm32f4xx_hal::interrupt::EXTI0 as u16)
            .handler(move |_c| {
                if button_0.is_low() {
                    led_1.toggle();
                    info!("Button 0 pressed.");
                }
                button_0.clear_interrupt_pending_bit();

                if button_1.is_low() {
                    led_1.set_high();
                    info!("Button 1 pressed.");
                }
                button_1.clear_interrupt_pending_bit();
            });

        #[link_section=".process.my_process"]
        static WORK_ITEMS: ConstPool<WorkItem<u32>, 10> = ConstPool::new();
        let button_event_producer = Arc::clone(&button_event);
        InterruptHandler::new(c)
            .stack(InterruptStack::Kernel)
            .connect_interrupt(stm32f4xx_hal::interrupt::EXTI9_5 as u16)
            .handler(move |_c| {
                if button_7.is_low() {
                    button_event_producer.add_permits(1);
                    info!("Button 7 pressed.");
                    //worker.submit(move || {
                    //    info!("Processing interrupt in worker queue.");
                    //}).ok();
                    WORK_ITEMS.try_acquire()
                        .map(|mut i| {
                            **i = 7;
                            worker.submit(
                                i,
                                move |d| {
                                    info!("Processing button {} from Workqueue.", d);
                                }
                            ).ok();
                        });
                }
                button_7.clear_interrupt_pending_bit();
            });
    }).ok();

    bern_kernel::start();
}

fn recursion(a: u32) {
    recursion(a + 1);
}

#[panic_handler] // built-in ("core") attribute
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    error!("Application panicked!");
    error!("{}", defmt::Display2Format(info));

    cortex_m::asm::bkpt();

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

/*#[allow(non_snake_case)]
#[interrupt]
fn EXTI15_10() {
    unsafe {
        if let Some(button) = &mut BUTTON {
            if button.is_low().unwrap() {
                info!("Interrupted!!!");
            }
            button.clear_interrupt_pending_bit();
        }
    }

}*/
