use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    GenericParam, Generics, Ident, Token,
};

pub(crate) struct PodDecl {
    _type_token: Token![type],
    pub ident: Ident,
    pub generics: Generics,
    _semi_token: Token![;],
}

impl Parse for PodDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(PodDecl {
            _type_token: input.parse()?,
            ident: input.parse()?,
            generics: input.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

/// Generate the TypeName impl (shared by export_pod, export_name, and export_class).
pub(crate) fn export_pod_inner(input: TS2, in_hicc: bool) -> Result<TokenStream, syn::Error> {
    let pod: PodDecl = syn::parse2(input.clone()).map_err(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            &format!(
                "export_pod expected `type Name;` or `type Name<'a, T: Bound>;`: {}",
                e
            ),
        )
    })?;

    let ident = &pod.ident;
    let generics = &pod.generics;
    let gp: Vec<TS2> = generics.params.iter().map(|p| quote! { #p }).collect();
    let has_params = !gp.is_empty();
    let self_type: TS2 = if has_params {
        let args: Vec<TS2> = generics
            .params
            .iter()
            .map(|p| match p {
                GenericParam::Lifetime(_) => quote! { '_ },
                GenericParam::Type(tp) => {
                    let i = &tp.ident;
                    quote! { #i }
                }
                GenericParam::Const(cp) => {
                    let i = &cp.ident;
                    quote! { #i }
                }
            })
            .collect();
        quote! { #ident < #(#args),* > }
    } else {
        quote! { #ident }
    };

    let value_type = if gp.is_empty() {
        quote! {
            default impl ::hicc_rs::ValueType for #ident {
                const N: usize = 0;
                type Result = Self;
                type Depth = ::hicc_rs::Depth0;
            }
        }
    } else {
        quote! {
            default impl<#(#gp),*> ::hicc_rs::ValueType for #self_type {
                const N: usize = 0;
                type Result = Self;
                type Depth = ::hicc_rs::Depth0;
            }
        }
    };

    let out_str = value_type.to_string();
    if in_hicc {
        let re = regex_lite::Regex::new(r"::\s*hicc_rs\s*::").unwrap();
        let out = re.replace_all(&out_str, "crate::");
        out.parse::<TS2>().map(|t| t.into()).map_err(|e| {
            syn::Error::new(proc_macro2::Span::call_site(), &format!("reparse: {}", e))
        })
    } else {
        out_str.parse::<TS2>().map(|t| t.into()).map_err(|e| {
            syn::Error::new(proc_macro2::Span::call_site(), &format!("reparse: {}", e))
        })
    }
}
