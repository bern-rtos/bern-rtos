use syn::parse::{Parse, ParseStream};
use syn::{Result, Type};
use syn::LitInt;
use proc_macro2::{TokenStream, Span};
use proc_macro2::Ident;
use syn::Token;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;

pub struct EnumMap{
    ident: Ident,
    repr: Type,
    entries: Vec<Entry>
}

pub struct Entry {
    pub ident: Ident,
    pub bits: LitInt,
    pub value: LitInt,
}

impl Parse for Entry {
    /// Parse one enum entry: `ident = bits, value`, e.g. `S128 = 4, 128`
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let bits: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let value: LitInt = input.parse()?;

        Ok(Entry {
            ident: ident,
            bits: bits,
            value: value
        })
    }
}

impl Parse for EnumMap {
    /// Parse multiple lines to map to an enum:
    /// ```txt
    /// enum_ident;
    /// ident = bits, value;
    /// /*...*/
    /// ```
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![;]>()?;

        let entries = Punctuated::<Entry, Token![;]>::parse_terminated(input)?;
        let enum_map = entries.into_iter().collect::<Vec<_>>();
        Ok(EnumMap {
            ident: enum_ident,
            repr: ty,
            entries: enum_map
        })
    }
}

impl ToTokens for EnumMap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let enum_ident = self.ident.clone();
        let repr = self.repr.clone();

        let idents = self.entries
            .iter()
            .map(|e| e.ident.clone())
            .collect::<Vec<_>>();
        let bits = self.entries
            .iter()
            .map(|e| e.bits.clone())
            .collect::<Vec<_>>();
        let values = self.entries
            .iter()
            .map(|e| e.value.clone())
            .collect::<Vec<_>>();

        /* format enum */
        let formatted = TokenStream::from(quote!{
            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
            #[repr(#repr)]
            pub enum #enum_ident {
                #(#idents = #bits,)*
            }
            impl Size {
                pub const fn bits(self) -> #repr {
                    self as #repr
                }
            }
        });
        tokens.extend(formatted);

        /* format macro to create enum from value */
        let mut macro_ident = self.ident.to_string().to_lowercase();
        macro_ident.push_str("_from");
        let values_str = values.iter().map(|v| v.to_string());
        let macro_ident = Ident::new(macro_ident.as_str(), Span::call_site());
        let formatted = TokenStream::from(quote!{
            #[macro_export]
            macro_rules! #macro_ident {
                #( (#values) => { #enum_ident::#idents }; )*
                ($x:expr) => {
                    compile_error!(concat!("Invalid parameter - possible values are:", #("\n         ", #values_str,)*));
                }
            }
        });
        tokens.extend(formatted);
    }
}