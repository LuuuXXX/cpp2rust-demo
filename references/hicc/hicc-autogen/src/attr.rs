use super::*;
use proc_macro2::Span;
use quote::{format_ident, ToTokens};

pub struct Comments<'a>(pub &'a [syn::Attribute]);

impl ToTokens for Comments<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let doc = format_ident!("doc");
        for attr in self.0 {
            if let syn::Meta::NameValue(syn::MetaNameValue { ref path, .. }) = &attr.meta {
                if path.is_ident(&doc) {
                    attr.to_tokens(tokens);
                }
            }
        }
    }
}

pub type AttributeArgs = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

#[derive(Debug)]
pub struct Attr {
    name: String,
    values: Vec<(String, Option<String>)>,
    span: Span,
}

#[allow(dead_code)]
impl Attr {
    pub fn get_attr(name: &str, attrs: &[syn::Attribute]) -> Option<Self> {
        Self::get_any_attr(&[name], attrs)
    }

    pub fn get_any_attr(name: &[&str], attrs: &[syn::Attribute]) -> Option<Self> {
        for attr in attrs {
            let attr = Attr::from_attr(attr);
            if name.contains(&attr.name()) {
                return Some(attr);
            }
        }
        None
    }

    pub fn get_value(name: &str, attrs: &[syn::Attribute]) -> Result<Option<String>, i32> {
        if let Some(attr) = Self::get_attr(name, attrs) {
            return attr.value(name).map(|v| v.map(|v| v.to_string()));
        }
        Err(0)
    }

    pub fn from_attrs(attr: &[syn::Attribute]) -> Option<Self> {
        attr.first().map(Self::from_attr)
    }

    pub fn from_attr(attr: &syn::Attribute) -> Self {
        Self::from_meta(&attr.meta)
    }

    pub fn from_meta(meta: &syn::Meta) -> Self {
        Self {
            name: get_meta_value(meta).0,
            values: get_meta_kv_list(meta),
            span: meta.span(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self, key: &str) -> Result<Option<&str>, i32> {
        for (k, v) in self.values.iter() {
            if k == key {
                return Ok(v.as_deref());
            }
        }
        Err(0)
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn contains(&self, key: &str) -> bool {
        self.value(key).is_ok()
    }
}

pub fn get_meta_value(attr: &syn::Meta) -> (String, Option<String>) {
    match attr {
        syn::Meta::Path(path) => (path_2_string(path), None),
        syn::Meta::NameValue(nv) => (path_2_string(&nv.path), expr_2_string(&nv.value)),
        syn::Meta::List(mlist) => (path_2_string(&mlist.path), None),
    }
}

pub fn get_meta_kv_list(attr: &syn::Meta) -> Vec<(String, Option<String>)> {
    match attr {
        syn::Meta::Path(path) => vec![(path_2_string(path), None)],
        syn::Meta::NameValue(nv) => vec![(path_2_string(&nv.path), expr_2_string(&nv.value))],
        syn::Meta::List(mlist) => get_meta_list(mlist),
    }
}

pub fn get_meta_list(mlist: &syn::MetaList) -> Vec<(String, Option<String>)> {
    let tokens = mlist.tokens.clone();
    let Ok(attrs) = syn::parse::Parser::parse2(AttributeArgs::parse_terminated, tokens) else {
        return vec![];
    };
    let mut output = vec![];
    for attr in &attrs {
        output.push(get_meta_value(attr));
    }
    output
}
