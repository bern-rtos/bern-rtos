//! System call.

/// System call.
pub trait ISyscall {
    /// A system call with a service ID and 3 arguments.
    fn syscall(service: u8, arg0: usize, arg1: usize, arg2: usize) -> usize;
}