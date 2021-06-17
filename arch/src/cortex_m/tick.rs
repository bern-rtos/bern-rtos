extern {  fn system_tick_update(); }

#[no_mangle]
extern "C" fn SysTick() {
    unsafe { system_tick_update() };
}