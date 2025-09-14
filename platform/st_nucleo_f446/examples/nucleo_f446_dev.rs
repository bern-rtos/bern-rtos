//#![deny(unsafe_code)]
#![no_main]
#![no_std]

//use defmt_rtt as _;

mod rtt_global;

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
use bern_kernel::mem::queue::spsc_const::ConstQueue;
use bern_kernel::stack::Stack;
use bern_kernel::units::frequency::ExtMilliHertz;

//use bern_kernel::log::rtt_target::{rtt_init_print};
use bern_kernel::{sleep, sync};

use log::LevelFilter;
use crate::rtt_global::RttLogger;

//use systemview_target::SystemView;
//rtos_trace::global_trace!{SystemView}
//static SYSTEMVIEW: SystemView = SystemView::new();
static LOGGER: RttLogger = RttLogger::new();

#[link_section=".process.my_process"]
static mut SOME_ARR: [u8; 8] = [1,2,3,4,5,6,7,8];

static PROC: &Process = bern_kernel::new_process!(my_process, 8192);
static PROC_RX: &Process = bern_kernel::new_process!(proc_rx, 8192);

#[entry]
fn main() -> ! {
    let mut board = StNucleoF446::new(72);

    //SYSTEMVIEW.init();
    //log::set_logger(&SYSTEMVIEW).ok();
    //log::set_max_level(LevelFilter::Info);
    //rtt_init_print!();
    LOGGER.init();
    log::set_logger(&LOGGER).ok();
    log::set_max_level(LevelFilter::Trace);

    bern_kernel::init();
    bern_kernel::time::set_tick_frequency(
        1.kHz(),
        72.MHz()
    );

    info!("Bern RTOS example application");

    let mut heartbeat = board.led.take().unwrap();
    let mut button = board.button;
    let mut button_0 = board.shield.button_0;
    let mut button_1 = board.shield.button_1;
    let mut button_7 = board.shield.button_7;
    let mut led_1 = board.shield.led_1;
    let mut led_7 = board.shield.led_7;

    // place a static receive queue in the receiving process
    #[link_section=".process.proc_rx"]
    static IPC_CHANNEL: ConstQueue<u32, 16> = ConstQueue::new();

    let channel = sync::ipc::spsc::channel();
    let (ipc_tx, ipc_rx) = channel.split(&IPC_CHANNEL).unwrap();

    PROC.init(move |c| {
        let asdf = Box::new(42);
        //info!("{:x}", &asdf as *const _ as usize);
        let _ = &asdf;

        let button_event = Arc::new(Semaphore::new(0));

        let worker = Workqueue::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .build();

        Thread::new(c)
            .priority(Priority::new(1))
            .stack(Stack::try_new_in(c, 2048).unwrap())
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
                        //systemview_target::info!("hello\0");
                        //info!("b = {}", *b);
                        if ipc_tx.send(*b).is_err() {
                            warn!("IPC queue full");
                        }
                    };

                    //bern_kernel::syscall::print_kernel_stats();
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


                    sleep(500);
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
            .connect_interrupt(stm32f4xx_hal::interrupt::EXTI15_10 as u16)
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

                if button.is_low() {
                    info!("Button pressed.");
                }
                button.clear_interrupt_pending_bit();
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
    }).unwrap();

    PROC_RX.init(|c| {
        Thread::new(c)
            .priority(Priority::new(2))
            .stack(Stack::try_new_in(c, 2048).unwrap())
            .spawn(move || {
                loop {
                    match ipc_rx.recv() {
                        Ok(v) => info!("IPC received {}", v),
                        Err(_) => warn!("IPC queue empty")
                    }
                    sleep(500);
                }
            });
    }).unwrap();

    bern_kernel::start();
}

#[allow(unconditional_recursion)]
fn recursion(a: u32) {
    recursion(a + 1);
}

#[panic_handler] // built-in ("core") attribute
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    error!("Application panicked!");
    //error!("{}", defmt::Display2Format(info));
    error!("{}", info);

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

// rtos_trace::global_application_callbacks!{Application}
// struct Application;
//
// impl rtos_trace::RtosTraceApplicationCallbacks for Application {
//     fn system_description() {
//         systemview_target::send_system_desc_app_name!("Development Application");
//         systemview_target::send_system_desc_device!("STM32F411RE");
//         systemview_target::send_system_desc_core!("Cortex-M4F");
//         systemview_target::send_system_desc_os!("Bern RTOS");
//         systemview_target::send_system_desc_interrupt!(15, "SysTick");
//         systemview_target::send_system_desc_interrupt!(11, "SysCall");
//     }
//
//     fn sysclock() -> u32 {
//         48_000_000
//     }
// }