extern "C" {
    fn system_tick_update();
}

#[no_mangle]
pub extern "C" fn SysTick() {
    unsafe { system_tick_update() };
}
