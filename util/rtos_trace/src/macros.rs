
#[macro_export]
macro_rules! global_trace {
    ($ident:ident) => {
        #[no_mangle]
        fn _rtos_trace_task_new(id: u32) {
            <$ident as $crate::RtosTrace>::task_new(id)
        }
        #[no_mangle]
        fn _rtos_trace_task_send_info(id: u32, info: $crate::TaskInfo) {
            <$ident as $crate::RtosTrace>::task_send_info(id: u32, info: $crate::TaskInfo)
        }
        #[no_mangle]
        fn _rtos_trace_task_terminate(id: u32) {
            <$ident as $crate::RtosTrace>::task_terminate(id)
        }
        #[no_mangle]
        fn _rtos_trace_task_exec_begin(id: u32) {
            <$ident as $crate::RtosTrace>::task_exec_begin(id)
        }
        #[no_mangle]
        fn _rtos_trace_task_exec_end() {
            <$ident as $crate::RtosTrace>::task_exec_end()
        }
        #[no_mangle]
        fn _rtos_trace_task_ready_begin(id: u32) {
            <$ident as $crate::RtosTrace>::task_ready_begin(id)
        }
        #[no_mangle]
        fn _rtos_trace_task_ready_end(id: u32) {
            <$ident as $crate::RtosTrace>::task_ready_end(id)
        }

        #[no_mangle]
        fn _rtos_trace_system_idle() {
            <$ident as $crate::RtosTrace>::system_idle()
        }

        #[no_mangle]
        fn _rtos_trace_isr_enter() {
            <$ident as $crate::RtosTrace>::isr_enter()
        }
        #[no_mangle]
        fn _rtos_trace_isr_exit() {
            <$ident as $crate::RtosTrace>::isr_exit()
        }
        #[no_mangle]
        fn _rtos_trace_isr_exit_to_scheduler() {
            <$ident as $crate::RtosTrace>::isr_exit_to_scheduler()
        }

        #[no_mangle]
        fn _rtos_trace_marker(id: u32) {
            <$ident as $crate::RtosTrace>::marker(id)
        }
        #[no_mangle]
        fn _rtos_trace_marker_begin(id: u32) {
            <$ident as $crate::RtosTrace>::marker_begin(id)
        }
        #[no_mangle]
        fn _rtos_trace_marker_end(id: u32) {
            <$ident as $crate::RtosTrace>::marker_end(id)
        }
    }
}


#[macro_export]
macro_rules! global_os_callbacks {
    ($ident:ident) => {
        #[no_mangle]
        fn _rtos_trace_task_list() {
            <$ident as $crate::RtosTraceOSCallbacks>::task_list()
        }
    }
}


#[macro_export]
macro_rules! global_application_callbacks {
    ($ident:ident) => {
        #[no_mangle]
        fn _rtos_trace_sysclock() -> u32 {
            <$ident as $crate::RtosTraceApplicationCallbacks>::sysclock()
        }
    }
}