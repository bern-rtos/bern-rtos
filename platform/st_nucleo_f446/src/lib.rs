#![no_std]

pub use stm32f4xx_hal as hal;
use hal::prelude::*;
use hal::pac::{
    Peripherals,
    USART2,
};
use hal::gpio::*;
use hal::serial::{
    Serial,
    Tx,
    Rx,
};

pub struct Vcp {
    pub tx: Tx<USART2>,
    pub rx: Rx<USART2>,
}

pub struct ShieldBfh {
    //pub led_0: EPin<Output<PushPull>>,
    pub led_1: EPin<Output<PushPull>>,
    pub led_2: EPin<Output<PushPull>>,
    pub led_3: EPin<Output<PushPull>>,
    //pub led_4: EPin<Output<PushPull>>, // conflict with USART2
    //pub led_5: EPin<Output<PushPull>>, // conflict with USART2
    pub led_6: EPin<Output<PushPull>>,
    pub led_7: EPin<Output<PushPull>>,

    pub button_0: EPin<Input<PullUp>>, // PB6
    pub button_1: EPin<Input<PullUp>>, // PB0
    pub button_2: EPin<Input<PullUp>>, // PB2
    pub button_3: EPin<Input<PullUp>>, // PB3
    pub button_4: EPin<Input<PullUp>>, // PB4
    pub button_5: EPin<Input<PullUp>>, // PB5
    pub button_6: EPin<Input<PullUp>>, // PB1
    pub button_7: EPin<Input<PullUp>>, // PB7
}

pub struct StNucleoF446 {
    pub led: Option<EPin<Output<PushPull>>>,
    pub button: EPin<Input<Floating>>,
    pub vcp: Option<Vcp>, // allow taking vcp and passing the board on, not optimal
    pub shield: ShieldBfh,
}

impl StNucleoF446 {
    pub fn new() -> Self {
        let mut stm32_peripherals = Peripherals::take()
            .expect("cannot take stm32 peripherals");

        /* Enable SYSCFGEN for interrutps to work */
        stm32_peripherals.RCC.apb2enr.write(|w| w.syscfgen().enabled());

        /* system clock */
        let rcc = stm32_peripherals.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(72.mhz()).freeze();

        /* gpio's */
        let gpioa = stm32_peripherals.GPIOA.split();
        let gpiob = stm32_peripherals.GPIOB.split();
        let gpioc = stm32_peripherals.GPIOC.split();

        /* Virtual Com Port (VCP) over debug adapter */
        let txd = gpioa.pa2.into_alternate();
        let rxd = gpioa.pa3.into_alternate();
        let vcp = Serial::new(
            stm32_peripherals.USART2,
            (txd, rxd),
            hal::serial::config::Config::default().baudrate(115_200.bps()),
            &clocks
        ).unwrap();
        let (vcp_tx, vcp_rx) = vcp.split();

        /* board IOs */
        let led = gpioa.pa5.into_push_pull_output().erase();
        let mut button = gpioc.pc13.into_floating_input().erase();

        /* BFH BTE5056 shield */
        //let shield_led_0 = gpiob.pb11.into_push_pull_output().downgrade();
        let shield_led_1 = gpiob.pb12.into_push_pull_output().erase();
        let shield_led_2 = gpioc.pc2.into_push_pull_output().erase();
        let shield_led_3 = gpioc.pc3.into_push_pull_output().erase();
        //let shield_led_4 = gpioa.pa2.into_push_pull_output().erase();
        //let shield_led_5 = gpioa.pa3.into_push_pull_output().erase();
        let shield_led_6 = gpioc.pc6.into_push_pull_output().erase();
        let shield_led_7 = gpioc.pc7.into_push_pull_output().erase();

        let mut shield_button_0 = gpiob.pb6.into_pull_up_input().erase();
        let mut shield_button_1 = gpiob.pb0.into_pull_up_input().erase();
        let shield_button_2 = gpiob.pb2.into_pull_up_input().erase();
        let shield_button_3 = gpiob.pb3.into_pull_up_input().erase();
        let shield_button_4 = gpiob.pb4.into_pull_up_input().erase();
        let shield_button_5 = gpiob.pb5.into_pull_up_input().erase();
        let shield_button_6 = gpiob.pb1.into_pull_up_input().erase();
        let mut shield_button_7 = gpiob.pb7.into_pull_up_input().erase();

        /* enable button interrupts */
        let mut syscfg = stm32_peripherals.SYSCFG.constrain();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut stm32_peripherals.EXTI);
        button.trigger_on_edge(&mut stm32_peripherals.EXTI, Edge::Falling);

        shield_button_0.make_interrupt_source(&mut syscfg);
        shield_button_0.enable_interrupt(&mut stm32_peripherals.EXTI);
        shield_button_0.trigger_on_edge(&mut stm32_peripherals.EXTI, Edge::Falling);

        shield_button_1.make_interrupt_source(&mut syscfg);
        shield_button_1.enable_interrupt(&mut stm32_peripherals.EXTI);
        shield_button_1.trigger_on_edge(&mut stm32_peripherals.EXTI, Edge::Falling);

        shield_button_7.make_interrupt_source(&mut syscfg);
        shield_button_7.enable_interrupt(&mut stm32_peripherals.EXTI);
        shield_button_7.trigger_on_edge(&mut stm32_peripherals.EXTI, Edge::Falling);


        unsafe {
            hal::pac::NVIC::unmask(hal::pac::interrupt::EXTI0);
            hal::pac::NVIC::unmask(hal::pac::interrupt::EXTI9_5);
            hal::pac::NVIC::unmask(hal::pac::interrupt::EXTI15_10);
        }

        /* assemble... */
        StNucleoF446 {
            led: Some(led),
            button,
            vcp: Some(Vcp {
                tx: vcp_tx,
                rx: vcp_rx,
            }),
            shield: ShieldBfh {
                //led_0: shield_led_0,
                led_1: shield_led_1,
                led_2: shield_led_2,
                led_3: shield_led_3,
                //led_4: shield_led_4,
                //led_5: shield_led_5,
                led_6: shield_led_6,
                led_7: shield_led_7,

                button_0: shield_button_0,
                button_1: shield_button_1,
                button_2: shield_button_2,
                button_3: shield_button_3,
                button_4: shield_button_4,
                button_5: shield_button_5,
                button_6: shield_button_6,
                button_7: shield_button_7,
            },
        }
    }
}
