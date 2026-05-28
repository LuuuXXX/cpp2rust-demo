use quote::{format_ident, quote};

pub fn path_2_string(path: &syn::Path) -> String {
    format!("{}", quote!(#path))
}

pub fn expr_2_string(expr: &syn::Expr) -> Option<String> {
    let syn::Expr::Lit(lit) = expr else {
        return Some(format!("{}", quote!(#expr)));
    };
    match &lit.lit {
        syn::Lit::Str(s) => Some(s.value()),
        syn::Lit::ByteStr(s) => String::from_utf8(s.value()).ok(),
        syn::Lit::CStr(s) => s.value().into_string().ok(),
        _ => Some(format!("{}", quote!(#expr))),
    }
}

pub fn string_2_path(path: &str) -> syn::Path {
    let mut idents = vec![];
    for ident in path.split("::") {
        if !ident.is_empty() {
            let ident = format_ident!("{}", ident.trim());
            idents.push(quote!(#ident));
        }
    }
    let tokens = if path.starts_with("::") {
        quote!(::#(#idents)::*)
    } else {
        quote!(#(#idents)::*)
    };
    syn::parse::<syn::Path>(tokens.into()).unwrap()
}

pub fn ident_string(s: &str) -> String {
    s.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

pub fn hicc_fn_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!("_hicc_{name}")
}

pub fn hicc_ty_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!("_Hicc_{name}")
}

pub fn hicc_methods_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!("_Hicc_{name}_Methods")
}
