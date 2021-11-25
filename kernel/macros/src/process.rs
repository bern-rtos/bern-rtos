use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

use syn::parse::{Parse, ParseStream};
use syn::Result;
use syn::LitInt;
use proc_macro2::{Span, TokenStream};
use proc_macro2::Ident;
use syn::Token;
use quote::{ToTokens, quote};

pub struct ProcessInfo {
    ident: Ident,
    memory_size: LitInt
}


impl Parse for ProcessInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let memory_size: LitInt = input.parse()?;

        Ok(ProcessInfo {
            ident,
            memory_size,
        })
    }
}

impl ToTokens for ProcessInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let process_ident = self.ident.clone();
        let process_ident_upper = Ident::new(&process_ident.to_string().to_uppercase(), Span::call_site());
        let size = self.memory_size.clone();

        let smprocess = Ident::new(&format!("__smprocess_{}", process_ident), Span::call_site());
        let emprocess = Ident::new(&format!("__emprocess_{}", process_ident), Span::call_site());
        let siprocess = Ident::new(&format!("__siprocess_{}", process_ident), Span::call_site());
        let shprocess = Ident::new(&format!("__shprocess_{}", process_ident), Span::call_site());
        let ehprocess = Ident::new(&format!("__ehprocess_{}", process_ident), Span::call_site());

        // Create a static process structure.
        //
        // Process memory locations are filled in by the linker.
        // There must no be multiple linker sections with the same name. Instead
        // of checking the linker script within this macro, we declare the
        // static variable `no_mangle`, so that the compiler does the check for
        // us.
        let formatted = TokenStream::from(quote!{
            {
                use bern_kernel::process::Process;
                use bern_kernel::common::process::ProcessMemory;

                extern "C" {
                    static mut #smprocess: usize;
                    static mut #emprocess: usize;
                    static #siprocess: usize;

                    static mut #shprocess: usize;
                    static mut #ehprocess: usize;
                }

                #[no_mangle]
                #[link_section = ".kernel.process"]
                static #process_ident_upper: Process = Process::new(unsafe { ProcessMemory {
                    size: #size,

                    bss_start: (& #smprocess) as *const _ as *const u8,
                    bss_end: (& #emprocess) as *const _ as *const u8,
                    bss_load: (& #siprocess) as *const _ as *const u8,

                    heap_start: (& #shprocess) as *const _ as *const u8,
                    heap_end: (& #ehprocess) as *const _ as *const u8,
                }});

                & #process_ident_upper
            }
        });
        tokens.extend(formatted);

        // Append link section
        let process_name = self.ident.clone().to_string();
        let process_size = self.memory_size.clone().to_string();

        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(out.join("bern.x"))
            .unwrap();

        write!(file, r###"
SECTIONS {{
    .process_{0} : ALIGN(8)
    {{
        /* Process static memory */
        . = ALIGN(8);
        __smprocess_{0} = .;
        *(.process.{0});
        . = ALIGN(8);
        __emprocess_{0} = .;

        /* Process heap */
        . = ALIGN(8);
        __shprocess_{0} = .;
        . = __smprocess_{0} + {1};
        __ehprocess_{0} = .;

        ASSERT(__emprocess_{0} <= __ehprocess_{0}, "Error: No room left in process {0}.");
    }} > RAM
    __siprocess_{0} = LOADADDR(.process_{0});
}} INSERT AFTER .bss;
"###,
               process_name,
               process_size
        ).unwrap();
    }
}