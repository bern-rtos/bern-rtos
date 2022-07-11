use crate::StNucleoF446;

pub trait TracePin {
    fn set_trace_out_high();
    fn set_trace_out_low();
    fn trace_input() -> bool;
}

impl TracePin for StNucleoF446 {
    #[inline]
    fn set_trace_out_high() {
        unsafe {
            (*stm32f4xx_hal::pac::GPIOC::ptr()).odr.modify(|_, w|  w.odr7().set_bit());
        }
    }

    #[inline]
    fn set_trace_out_low() {
        unsafe {
            (*stm32f4xx_hal::pac::GPIOC::ptr()).odr.modify(|_, w|  w.odr7().clear_bit());
        }
    }

    fn trace_input() -> bool {
        todo!()
    }
}