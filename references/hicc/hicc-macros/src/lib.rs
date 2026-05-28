use proc_macro::TokenStream;
use quote::quote;

use hicc_autogen::{Cpp, ImportClass, ImportLib};

macro_rules! error_tokens {
    ($e: expr) => {
        match $e {
            Ok(val) => val,
            Err(e) => return e.to_compile_error().into(),
        }
    };
}

#[proc_macro]
pub fn import_lib(input: TokenStream) -> TokenStream {
    let lib = syn::parse_macro_input!(input as ImportLib);
    let items = error_tokens!(lib.generate());
    quote!(#(#items)*).into()
}

#[proc_macro]
pub fn import_class(input: TokenStream) -> TokenStream {
    let class = syn::parse_macro_input!(input as ImportClass);
    let items = error_tokens!(class.generate());
    quote!(#(#items)*).into()
}

#[proc_macro]
pub fn cpp(input: TokenStream) -> TokenStream {
    let _ = syn::parse_macro_input!(input as Cpp);
    quote!().into()
}
