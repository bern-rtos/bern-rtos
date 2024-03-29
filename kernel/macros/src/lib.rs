//! Procedural macros for the `bern_kernel`.
//!
//! This crate provides macros that:
//! - simplify the kernel usage
//! - make kernel development less tedious (used internally)
mod enum_map;
mod process;

use proc_macro::TokenStream;
use quote::ToTokens;

/// Generates an enum with values and a map to match the enum to another type.
///
/// e.g.
/// ```ignore
/// enum_map!{
///     Size, u8;
///     S128 = 5, 128;
///     S256 = 6, 256;
/// }
/// ```
/// expands to
/// ```ignore
/// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// #[repr(u8)]
/// pub enum Size {
///     S128 = 5,
///     S256 = 6,
/// }
/// impl Size {
///     pub const fn bits(self) -> u8 {
///         self as u8
///     }
/// }
///
/// #[macro_export]
/// macro_rules! size_from {
///     (128) => { Size::S128 };
///     (256) => { Size::S256 };
///     ($x:expr) => {
///         compile_error!("Invalid parameter - possible values are: 128, 256");
///     }
/// }
/// ```
/// ```
#[doc(hidden)]
#[proc_macro]
pub fn enum_map(input: TokenStream) -> TokenStream {
    let map = syn::parse_macro_input!(input as enum_map::EnumMap);
    let mut output = proc_macro2::TokenStream::new();
    map.to_tokens(&mut output);
    TokenStream::from(output)
}

/// Creates a new process and the required linker sections.
///
/// # Example
/// ```ignore
/// // Create a process named `my_proc` with 32kB memory.
/// static MY_PROC: &Process = bern_kernel::new_process!(my_proc, 32768);
///
/// // Place static variable in process memory.
/// #[link_section = ".process.my_proc"]
/// static DATA: u32 = 0xDEADBEEF;
///
/// #[entry]
/// fn main() -> ! {
///     let board = Board::new();
///     bern_kernel::init();
///     /*..*/
///     MY_PROC.init(move |c| {
///         // Spawn threads.
///     }).unwrap();
///     /*..*/
///     bern_kernel::start();
/// }
/// ```
#[proc_macro]
pub fn new_process(input: TokenStream) -> TokenStream {
    let map = syn::parse_macro_input!(input as process::ProcessInfo);
    let mut output = proc_macro2::TokenStream::new();
    map.to_tokens(&mut output);
    TokenStream::from(output)
}