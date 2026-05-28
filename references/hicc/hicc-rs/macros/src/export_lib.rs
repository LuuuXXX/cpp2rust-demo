use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::{format_ident, quote};
use syn::{FnArg, ItemMod, ReturnType};

use crate::is_unit;

pub(crate) fn export_lib_inner(
    input: TS2,
    export_name_from_attr: &str,
    in_hicc: bool,
) -> Result<TokenStream, syn::Error> {
    let item_mod: ItemMod = syn::parse2(input)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), &e.to_string()))?;
    let export_name = export_name_from_attr.to_string();
    let content = item_mod.content.as_ref().ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "export_lib requires a module with body",
        )
    })?;

    let mod_ident = item_mod.ident;
    let mod_attrs: Vec<&syn::Attribute> = item_mod
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("export_lib"))
        .collect();
    let lib_struct = format_ident!("_Hicc_{}", export_name);
    let create_fn = format_ident!("{}", export_name);

    let mut lib_fns: Vec<&syn::ItemFn> = Vec::new();
    let mut pass_through: Vec<TS2> = Vec::new();

    for item in &content.1 {
        match item {
            syn::Item::Fn(f) => lib_fns.push(f),
            other => pass_through.push(quote! { #other }),
        }
    }

    if lib_fns.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "no functions found in export_lib block",
        ));
    }

    let mut struct_fields = Vec::new();
    let mut adapter_fns = Vec::new();
    let mut initializers = Vec::new();

    for f in &lib_fns {
        let fn_name = &f.sig.ident;
        let adapter_name = format_ident!("_hicc_{}_{}", mod_ident, fn_name);
        let has_body = !f.block.stmts.is_empty();
        let block = &f.block;
        let extra: Vec<_> = f
            .sig
            .inputs
            .iter()
            .filter_map(|i| match i {
                FnArg::Typed(pt) => Some((pt.pat.clone(), pt.ty.clone())),
                _ => None,
            })
            .collect();

        let param_types: Vec<_> = extra
            .iter()
            .map(|(_, t)| quote! { <#t as ::hicc_rs::AbiType>::InputType })
            .collect();
        let rt = match &f.sig.output {
            ReturnType::Type(_, ty) => {
                if is_unit(ty) {
                    quote! {}
                } else {
                    quote! { -> <#ty as ::hicc_rs::AbiType>::OutputType }
                }
            }
            _ => quote! {},
        };
        struct_fields.push(quote! {
            pub #fn_name: unsafe extern "C" fn(#(#param_types),*) #rt
        });

        let oty = match &f.sig.output {
            ReturnType::Type(_, ty) => {
                if is_unit(ty) {
                    None
                } else {
                    Some(ty.clone())
                }
            }
            _ => None,
        };
        let rty = oty
            .as_ref()
            .map(|t| quote! { -> <#t as ::hicc_rs::AbiType>::OutputType })
            .unwrap_or(quote! {});

        let abi_params: Vec<_> = extra
            .iter()
            .map(|(p, t)| quote! { #p: <#t as ::hicc_rs::AbiType>::InputType })
            .collect();
        let ec: Vec<_> = extra
            .iter()
            .map(|(p, t)| quote! { let #p = <#t as ::hicc_rs::AbiType>::from_abi(#p); })
            .collect();
        let call_args: Vec<_> = extra.iter().map(|(p, _)| quote! { #p }).collect();

        let body = if has_body {
            if let Some(ref ret_ty) = oty {
                quote! { #(#ec)* <#ret_ty as ::hicc_rs::AbiType>::into_abi({ #block }) }
            } else {
                quote! { #(#ec)* #block }
            }
        } else if let Some(ref ret_ty) = oty {
            quote! {
                #(#ec)* <#ret_ty as ::hicc_rs::AbiType>::into_abi(crate::#fn_name(#(#call_args),*))
            }
        } else {
            quote! { #(#ec)* crate::#fn_name(#(#call_args),*); }
        };

        adapter_fns.push(quote! {
            unsafe extern "C" fn #adapter_name(#(#abi_params),*) #rty { #body }
        });
        initializers.push(quote! { #fn_name: #adapter_name });
    }

    let result = quote! {
        #(#mod_attrs)*
        mod #mod_ident {
            #(#pass_through)*
            #[repr(C)]
            #[allow(dead_code, non_camel_case_types)]
            pub struct #lib_struct { #(#struct_fields),* }
            #(#adapter_fns)*
            impl #lib_struct { const METHODS: Self = Self { #(#initializers),* }; }
            #[unsafe(no_mangle)]
            pub extern "C" fn #create_fn() -> &'static #lib_struct { &#lib_struct::METHODS }
        }
    };
    let result_str = result.to_string();
    let result_str = if in_hicc {
        let re = regex_lite::Regex::new(r"::\s*hicc_rs\s*::").unwrap();
        re.replace_all(&result_str, "crate::").to_string()
    } else {
        result_str
    };
    match result_str.parse::<TokenStream>() {
        Ok(t) => Ok(t),
        Err(e) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            &format!("reparse: {}", e),
        )),
    }
}
