#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{compiler_fence, Ordering};
use bern_kernel::exec::process::Context;
use bern_kernel::exec::runnable::Priority;
use bern_kernel::exec::thread::Thread;
use bern_kernel::sleep;
use bern_kernel::stack::Stack;
use bern_kernel::sync::Semaphore;
use crate::common_timing::*;

mod common_timing;

// Semaphore delay: A high priority thread is waiting for a semaphore. We
// measure the time it tasks for the context switch when the semaphore is given.
pub fn spawn_timing_thread(c: &Context, board: Board) {
    let input = board.shield.button_0;
    let sem = Arc::new(Semaphore::new(0));
    Board::set_trace_out_high();

    let sem_consumer = sem.clone();
    Thread::new(c)
        .priority(Priority::new(0))
        .stack(Stack::try_new_in(c, 1024).unwrap())
        .spawn(move || {
            loop {
                let guard = sem_consumer.acquire(0);
                Board::set_trace_out_high();
                guard
                    .unwrap()
                    .forget();
            }
        });

    Thread::new(c)
        .priority(Priority::new(1))
        .stack(Stack::try_new_in(c, 1024).unwrap())
        .spawn(move || {
            loop {
                if input.is_low() {
                    Board::set_trace_out_low();
                    sem.add_permits(1);
                    while input.is_low() {
                        compiler_fence(Ordering::SeqCst);
                    }
                }
            }
        });

}