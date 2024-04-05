use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Singleton)]
pub fn singleton(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let const_ident = format_ident!("{}", ident.to_string().to_uppercase());

    let output = quote! {
        const #const_ident: #ident = #ident;
        impl Singleton for #ident {
            fn instance() -> Self {
                #const_ident
            }
        }
    };

    output.into()
}
