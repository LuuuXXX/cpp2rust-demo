use super::*;
use quote::{format_ident, quote, ToTokens};
use syn::parse;
use syn::parse::discouraged::Speculative;
use syn::punctuated::Punctuated;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FnArg {
    pub name: syn::Ident,
    pub colon_token: syn::Token![:],
    pub ty: syn::Type,
}

impl parse::Parse for FnArg {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let colon_token: syn::Token![:] = input.parse()?;
        let ty: syn::Type = input.parse()?;
        Ok(Self {
            name,
            colon_token,
            ty,
        })
    }
}

impl ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[allow(dead_code)]
pub struct Lifetime {
    pub lt_token: syn::Token![<],
    pub lifetimes: Punctuated<syn::Lifetime, syn::Token![,]>,
    pub gt_token: syn::Token![>],
}

impl parse::Parse for Lifetime {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let lt_token = input.parse()?;
        let mut lifetimes = Punctuated::new();
        let ahead = input.fork();
        loop {
            if let Ok(lif) = ahead.parse::<syn::Lifetime>() {
                lifetimes.push_value(lif);
                input.advance_to(&ahead);
                if let Ok(comma) = ahead.parse() {
                    lifetimes.push_punct(comma);
                    input.advance_to(&ahead);
                    continue;
                }
            }
            break;
        }
        let gt_token: syn::Token![>] = input.parse()?;
        if lifetimes.is_empty() {
            return Err(syn::Error::new(gt_token.span(), "empty lifetime"));
        }
        Ok(Self {
            lt_token,
            lifetimes,
            gt_token,
        })
    }
}

impl ToTokens for Lifetime {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.lifetimes.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

#[allow(dead_code)]
pub struct ImportFn {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub unsafety: Option<syn::Token![unsafe]>,
    pub fn_token: syn::Token![fn],
    pub ident: syn::Ident,
    pub lifetime: Option<Lifetime>,
    pub paren_token: syn::token::Paren,
    pub recv: Option<syn::Receiver>,
    pub comma_token: Option<syn::Token![,]>,
    pub inputs: Punctuated<FnArg, syn::Token![,]>,
    pub variadic: Option<syn::Token![...]>,
    pub output: syn::ReturnType,
    pub semi_token: syn::Token![;],
}

impl Acceptor for ImportFn {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        for input in self.inputs.iter_mut() {
            input.ty.accept(visitor)?;
        }
        if let syn::ReturnType::Type(_, ref mut ty) = self.output {
            ty.accept(visitor)?;
        }
        Ok(())
    }
}

pub struct CppFnComments<'a>(pub &'a ImportFn);

impl ToTokens for CppFnComments<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Some(attr) = Attr::get_attr("cpp", &self.0.attrs) else {
            return;
        };
        let comments = if let Ok(Some(member)) = attr.value("member") {
            format!("cpp member function: `{member}`\n")
        } else if let Ok(Some(func)) = attr.value("func") {
            format!("cpp global function: `{func}`\n")
        } else if let Ok(Some(field)) = attr.value("field") {
            format!("cpp member data: `{field}`\n")
        } else if let Ok(Some(data)) = attr.value("data") {
            format!("cpp global data: `{data}`\n")
        } else {
            return;
        };
        quote! {
            #[doc = #comments]
        }
        .to_tokens(tokens);
    }
}

pub struct TraitFnSignature<'a>(pub &'a ImportFn, pub Option<&'a syn::Ident>);

impl ToTokens for TraitFnSignature<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.unsafety.to_tokens(tokens);
        self.0.fn_token.to_tokens(tokens);
        if let Some(ident) = &self.1 {
            ident.to_tokens(tokens);
        } else {
            self.0.ident.to_tokens(tokens);
        }
        self.0.lifetime.to_tokens(tokens);
        self.0.paren_token.surround(tokens, |tokens| {
            self.0.recv.to_tokens(tokens);
            self.0.comma_token.to_tokens(tokens);
            self.0.inputs.to_tokens(tokens);
            // 不应该有，如果有，rustc会报错
            self.0.variadic.to_tokens(tokens);
        });
        self.0.output.to_tokens(tokens);
    }
}

pub struct Signature<'a>(pub &'a ImportFn, pub Option<&'a syn::Ident>);

impl ToTokens for Signature<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.vis.to_tokens(tokens);
        TraitFnSignature(self.0, self.1).to_tokens(tokens);
    }
}

impl ToTokens for ImportFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        TraitFnSignature(self, None).to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

pub struct CallArguments<'a>(pub &'a ImportFn);

impl ToTokens for CallArguments<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0.recv.is_some() {
            quote! {self,}.to_tokens(tokens);
        }
        for (n, arg) in self.0.inputs.iter().enumerate() {
            if n > 0 {
                quote!(,).to_tokens(tokens);
            }
            arg.name.to_tokens(tokens);
        }
    }
}

pub struct TypeBareFn<'a>(pub &'a ImportFn, pub Option<&'a Class>);

impl ToTokens for TypeBareFn<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(lifetime) = &self.0.lifetime {
            quote!(for #lifetime).to_tokens(tokens);
        }
        self.0.unsafety.to_tokens(tokens);
        quote!(extern "C").to_tokens(tokens);
        self.0.fn_token.to_tokens(tokens);

        self.0.paren_token.surround(tokens, |tokens| {
            if let Some(recv) = self.0.recv.as_ref() {
                let class = self.1.unwrap();
                let ident = &class.ident;
                let generics = &class.generics;
                if let Some((ref and_, ref ref_)) = recv.reference {
                    let mutability = &recv.mutability;
                    quote!(#and_ #ref_ #mutability #ident #generics).to_tokens(tokens);
                } else {
                    quote!(#ident #generics).to_tokens(tokens);
                }
                self.0.comma_token.to_tokens(tokens);
            }
            for (n, arg) in self.0.inputs.iter().enumerate() {
                if n > 0 {
                    quote!(,).to_tokens(tokens);
                }
                arg.ty.to_tokens(tokens);
            }
            if self.0.variadic.is_some() {
                quote!(,).to_tokens(tokens);
                self.0.variadic.to_tokens(tokens);
            }
        });
        self.0.output.to_tokens(tokens);
    }
}

pub struct ImportField<'a>(pub &'a ImportFn, pub Option<&'a Class>);

impl ToTokens for ImportField<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.ident.to_tokens(tokens);
        quote!(:).to_tokens(tokens);
        if self.0.recv.is_some() {
            let f = TypeBareFn(self.0, self.1);
            quote!(Option<#f>).to_tokens(tokens);
        } else {
            TypeBareFn(self.0, self.1).to_tokens(tokens);
        }
    }
}

pub fn mut_type_from_exception(ty: &mut syn::Type) -> Option<&mut syn::Type> {
    let syn::Type::Path(syn::TypePath { qself: None, path }) = ty else {
        return None;
    };
    let segments = &path.segments;
    let segments = (segments.get(0), segments.get(1), segments.get(2));
    let segment = match segments {
        (Some(p1), Some(p2), None) if p1.ident == "hicc" && p2.ident == "Exception" => {
            &mut path.segments[1]
        }
        (Some(p1), None, None) if p1.ident == "Exception" => &mut path.segments[0],
        _ => return None,
    };
    let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match &mut args.args[0] {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

pub fn type_from_exception(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(syn::TypePath { qself: None, path }) = ty else {
        return None;
    };
    let segments = &path.segments;
    let segments = (segments.get(0), segments.get(1), segments.get(2));
    let segment = match segments {
        (Some(p1), Some(p2), None) if p1.ident == "hicc" && p2.ident == "Exception" => p2,
        (Some(p1), None, None) if p1.ident == "Exception" => p1,
        _ => return None,
    };
    let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
        return None;
    };
    match (args.args.get(0), args.args.get(1)) {
        (Some(syn::GenericArgument::Type(ref ty)), None) => Some(ty),
        _ => None,
    }
}

fn type_from_abitype(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(syn::TypePath {
        qself: Some(ref qself),
        path,
    }) = ty
    else {
        return None;
    };
    let segs = &path.segments;
    match (segs.get(0), segs.get(1), segs.get(2), segs.get(3)) {
        (Some(p1), Some(p2), Some(_), None) if p1.ident == "hicc" && p2.ident == "AbiType" => {
            Some(&qself.ty)
        }
        (Some(p1), Some(_), None, None) if p1.ident == "AbiType" => Some(&qself.ty),
        _ => None,
    }
}

impl ImportFn {
    // 可能由AbiType定义, 并被包装到Exception中.
    pub fn return_cabi_type(&self) -> Option<&syn::Type> {
        let syn::ReturnType::Type(_, ref ty) = self.output else {
            return None;
        };
        let ty = type_from_exception(ty).unwrap_or(ty);
        type_from_abitype(ty).or(Some(ty))
    }

    pub fn return_except(&self) -> bool {
        let syn::ReturnType::Type(_, ref ty) = self.output else {
            return false;
        };
        let syn::Type::Path(ref path) = **ty else {
            return false;
        };
        let s = path_2_string(&path.path);
        s.starts_with("Exception <") | s.starts_with("hicc :: Exception <")
            || s.starts_with(":: hicc :: Exception <")
    }

    pub fn class_accept<T: ClassVisitor>(&mut self, visitor: &T) {
        for input in self.inputs.iter_mut() {
            input.ty.class_accept(visitor);
        }
        self.output.class_accept(visitor);
    }

    fn lifetime(mut self) -> Self {
        if self.lifetime.is_some() || !self.recv_ref() || !self.return_ref() {
            return self;
        }
        let tokens = quote!(<'a>);
        self.lifetime = Some(syn::parse2::<Lifetime>(tokens).unwrap());
        self.recv_ref_set();
        self.return_ref_set();
        self
    }

    fn recv_ref(&self) -> bool {
        if let Some(recv) = self.recv.as_ref() {
            return recv.reference.is_some();
        }
        false
    }

    fn recv_ref_set(&mut self) {
        let lifetime = self.lifetime.as_ref().unwrap().lifetimes[0].clone();
        if let Some(ref mut recv) = self.recv {
            if let Some((_, ref mut lif)) = recv.reference {
                *lif = Some(lifetime);
            }
        }
    }

    fn return_ref(&mut self) -> bool {
        struct LifetimeVisitor(bool);
        impl Visitor for LifetimeVisitor {
            fn visit_fn(&mut self, _: &mut syn::TypeBareFn) -> parse::Result<()> {
                Err(syn::Error::new(Span::call_site(), ""))
            }
            fn visit_lif(&mut self, lif: &mut syn::Lifetime) -> parse::Result<()> {
                self.0 = lif.ident == format_ident!("_");
                Err(syn::Error::new(Span::call_site(), ""))
            }
            fn visit_lif_opt(&mut self, lif: &mut Option<syn::Lifetime>) -> parse::Result<()> {
                if let Some(ref lif) = lif {
                    self.0 = lif.ident == format_ident!("_");
                } else {
                    self.0 = true;
                }
                Err(syn::Error::new(Span::call_site(), ""))
            }
        }
        let mut visitor = LifetimeVisitor(false);
        let _ = self.output.accept(&mut visitor);
        visitor.0
    }

    fn return_ref_set(&mut self) {
        let lifetime = self.lifetime.as_ref().unwrap().lifetimes[0].clone();
        struct LifetimeVisitor<'a>(&'a syn::Lifetime);
        impl Visitor for LifetimeVisitor<'_> {
            fn visit_fn(&mut self, _: &mut syn::TypeBareFn) -> parse::Result<()> {
                Err(syn::Error::new(Span::call_site(), ""))
            }
            fn visit_lif(&mut self, lif: &mut syn::Lifetime) -> parse::Result<()> {
                *lif = self.0.clone();
                Ok(())
            }
            fn visit_lif_opt(&mut self, lif: &mut Option<syn::Lifetime>) -> parse::Result<()> {
                *lif = Some(self.0.clone());
                Ok(())
            }
        }
        let _ = self.output.accept(&mut LifetimeVisitor(&lifetime));
    }
}

impl parse::Parse for ImportFn {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis: syn::Visibility = input.parse()?;
        let mut unsafety: Option<syn::Token![unsafe]> = input.parse()?;
        let fn_token: syn::Token![fn] = input.parse()?;
        let ident: syn::Ident = input.parse()?;

        let ahead = input.fork();
        let lifetime = ahead.parse::<Lifetime>().ok();
        if lifetime.is_some() {
            input.advance_to(&ahead);
        };

        let args;
        let paren_token = syn::parenthesized!(args in input);

        let ahead = args.fork();
        let recv = ahead.parse::<syn::Receiver>().ok();
        if recv.is_some() {
            args.advance_to(&ahead);
        }
        let comma_token = args.parse::<Option<syn::Token![,]>>()?;
        let mut variadic = None;
        let mut inputs = Punctuated::new();
        while !args.is_empty() {
            let ahead = args.fork();
            if let Ok(arg) = ahead.parse::<FnArg>() {
                inputs.push_value(arg);
                args.advance_to(&ahead);
                if args.is_empty() {
                    break;
                }
                let comma = args.parse()?;
                inputs.push_punct(comma);
            } else {
                variadic = args.parse::<Option<syn::Token![...]>>()?;
                if !args.is_empty() {
                    return Err(syn::Error::new(
                        args.span(),
                        "variadic not the last parameter",
                    ));
                }
                if unsafety.is_none() {
                    unsafety = syn::parse2(quote!(unsafe)).unwrap();
                    //return Err(syn::Error::new(variadic.span(), "not unsafe function"));
                }
                break;
            }
        }
        let output: syn::ReturnType = input.call(syn::ReturnType::without_plus)?;
        let semi_token: syn::Token![;] = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            unsafety,
            fn_token,
            ident,
            lifetime,
            paren_token,
            recv,
            comma_token,
            inputs,
            output,
            variadic,
            semi_token,
        }
        .lifetime())
    }
}
