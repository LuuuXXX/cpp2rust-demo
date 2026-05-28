use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, TokenStream as TS2, TokenTree};
use syn::Type;

mod export_class;
mod export_lib;
mod export_pod;

// =====================================================================
// Shared utilities
// =====================================================================

pub(crate) fn replace_semicolons(stream: TS2) -> TS2 {
    replace_semicolons_depth(stream, 0u32, false)
}

struct ParsedAttrs {
    in_hicc: bool,
    name: Option<String>,
}

fn parse_export_attrs(attr_str: &str, allow_name: bool) -> Result<ParsedAttrs, syn::Error> {
    let mut in_hicc = false;
    let mut name = None;

    for part in attr_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part == "in_hicc" {
            in_hicc = true;
            continue;
        }
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].trim();
            if key == "name" {
                if !allow_name {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "attribute `name` is not supported on this macro",
                    ));
                }
                let val = part[eq_pos + 1..].trim().trim_matches('"');
                if !val.is_empty() {
                    name = Some(val.to_string());
                }
                continue;
            }
        }
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("unknown attribute `{}`", part),
        ));
    }

    Ok(ParsedAttrs { in_hicc, name })
}

fn replace_semicolons_depth(stream: TS2, brace_depth: u32, in_non_brace_group: bool) -> TS2 {
    let mut out = TS2::new();
    for token in stream {
        match token {
            TokenTree::Group(g) => match g.delimiter() {
                Delimiter::Brace => {
                    let inner = replace_semicolons_depth(g.stream(), brace_depth + 1, false);
                    let mut ng = Group::new(g.delimiter(), inner);
                    ng.set_span(g.span());
                    out.extend(std::iter::once(TokenTree::Group(ng)));
                }
                _ => {
                    let inner = replace_semicolons_depth(g.stream(), brace_depth, true);
                    let mut ng = Group::new(g.delimiter(), inner);
                    ng.set_span(g.span());
                    out.extend(std::iter::once(TokenTree::Group(ng)));
                }
            },
            TokenTree::Punct(p)
                if p.as_char() == ';' && brace_depth == 1 && !in_non_brace_group =>
            {
                out.extend(std::iter::once(TokenTree::Group(Group::new(
                    Delimiter::Brace,
                    TS2::new(),
                ))));
            }
            o => out.extend(std::iter::once(o)),
        }
    }
    out
}

pub(crate) fn is_unit(ty: &Type) -> bool {
    match ty {
        Type::Tuple(t) => t.elems.is_empty(),
        _ => false,
    }
}

// =====================================================================
// export_class
// =====================================================================

#[proc_macro_attribute]
pub fn export_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: TS2 = item.into();
    let input = replace_semicolons(input);
    let parsed = match parse_export_attrs(&attr.to_string(), false) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };
    match export_class::export_class_inner(input, parsed.in_hicc) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

// =====================================================================
// export_lib
// =====================================================================

#[proc_macro_attribute]
pub fn export_lib(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: TS2 = item.into();
    let input = replace_semicolons(input);
    let parsed = match parse_export_attrs(&attr.to_string(), true) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };
    let export_name = parsed.name.unwrap_or_else(|| "hicc_export_lib".to_string());
    match export_lib::export_lib_inner(input, &export_name, parsed.in_hicc) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

// =====================================================================
// export_pod
// =====================================================================

#[proc_macro_attribute]
pub fn export_pod(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: TS2 = item.into();
    let parsed = match parse_export_attrs(&attr.to_string(), false) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };
    match export_pod::export_pod_inner(input, parsed.in_hicc) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}
