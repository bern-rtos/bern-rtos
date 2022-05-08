macro_rules! _stub_callbacks_os {
    () => {
        #[no_mangle]
        pub unsafe extern "C" fn _rtos_trace_task_list() {
        }
        #[no_mangle]
        pub unsafe extern "C" fn _rtos_trace_time() -> u64 {
            0
        }
    }
}

macro_rules! _stub_callbacks_app {
    () => {
        #[no_mangle]
        pub unsafe extern "C" fn _rtos_trace_system_description() {
        }
        #[no_mangle]
        pub unsafe extern "C" fn _rtos_trace_sysclock() -> u64 {
            0
        }
    }
}


#[cfg(not(feature = "callbacks-os"))]
_stub_callbacks_os!{}

#[cfg(not(feature = "callbacks-app"))]
_stub_callbacks_app!{}