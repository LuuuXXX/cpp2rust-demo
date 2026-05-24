use super::*;
use quote::{format_ident, quote, ToTokens};
use syn::parse::discouraged::Speculative;
use syn::punctuated::Punctuated;
use syn::{self, parse};

mod kw {
    syn::custom_keyword!(class);
    syn::custom_keyword!(interface);
}

#[allow(dead_code)]
pub struct ClassDecl {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub class_token: kw::class,
    pub ident: syn::Ident,
    pub generics: Option<Generics>,
    pub equal_token: Option<syn::Token![=]>,
    pub origin: Option<syn::Path>,
    pub semi_token: syn::Token![;],
}

#[allow(dead_code)]
pub struct Head {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub class_token: kw::class,
}

#[allow(dead_code)]
pub struct Class {
    pub cpps: Vec<Cpp>,
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub class_token: kw::class,
    pub ident: syn::Ident,
    pub generics: Option<Generics>,
    pub colon_token: Option<syn::Token![:]>,
    pub intf: Option<syn::Ident>,
    pub intf_generics: Option<Generics>,
    pub brace_token: syn::token::Brace,
    pub methods: Vec<ImportFn>,
    pub others: Vec<syn::Item>,
}

#[allow(dead_code)]
pub struct Generics {
    pub lt_token: syn::Token![<],
    pub types: Punctuated<syn::Ident, syn::Token![,]>,
    pub gt_token: syn::Token![>],
}

struct SelfVisitor<'a>(&'a syn::Ident, &'a Option<Generics>);

impl Visitor for SelfVisitor<'_> {
    fn visit_path(&mut self, path: &mut syn::Path) -> parse::Result<()> {
        if path.is_ident("Self") {
            let ident = self.0;
            let generics = self.1;
            *path = syn::parse2::<syn::Path>(quote!(#ident #generics)).unwrap();
        }
        Ok(())
    }
}

impl parse::Parse for Head {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            vis: input.parse()?,
            class_token: input.parse()?,
        })
    }
}

impl parse::Parse for ClassDecl {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let class_token = input.parse()?;
        let ident = input.parse()?;
        let ahead = input.fork();
        let generics: Option<Generics> = ahead.parse().ok();
        if generics.is_some() {
            input.advance_to(&ahead);
        }
        let ahead = input.fork();
        if let Some(equal_token) = ahead.parse()? {
            input.advance_to(&ahead);
            Ok(Self {
                attrs,
                vis,
                class_token,
                ident,
                generics,
                equal_token: Some(equal_token),
                origin: Some(input.parse()?),
                semi_token: input.parse()?,
            })
        } else {
            Ok(Self {
                attrs,
                vis,
                class_token,
                ident,
                generics,
                equal_token: None,
                origin: None,
                semi_token: input.parse()?,
            })
        }
    }
}

impl ToTokens for ClassDecl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.equal_token.is_none() {
            return;
        }
        self.attrs.iter().for_each(|attr| attr.to_tokens(tokens));
        quote! {
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
        }
        .to_tokens(tokens);
        self.vis.to_tokens(tokens);
        quote!(type).to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.equal_token.to_tokens(tokens);
        self.origin.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl parse::Parse for Generics {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let lt_token = input.parse()?;
        let mut types = Punctuated::new();
        loop {
            let ahead = input.fork();
            if let Ok(ty) = ahead.parse() {
                types.push_value(ty);
                input.advance_to(&ahead);
                if let Ok(comma_token) = ahead.parse() {
                    types.push_punct(comma_token);
                    input.advance_to(&ahead);
                }
                continue;
            }
            break;
        }
        let gt_token: syn::Token![>] = input.parse()?;
        if types.is_empty() {
            return Err(syn::Error::new(gt_token.span(), "empty generic types"));
        }
        Ok(Self {
            lt_token,
            types,
            gt_token,
        })
    }
}

impl ToTokens for Generics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.types.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

pub struct ImplGenerics<'a>(pub &'a Option<Generics>, pub &'a syn::Path);

impl ToTokens for ImplGenerics<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Some(generics) = self.0.as_ref() else {
            return;
        };
        generics.lt_token.to_tokens(tokens);
        let hicc = self.1;
        for ty in generics.types.iter() {
            ty.to_tokens(tokens);
            quote! { : #hicc::AbiType + 'static, }.to_tokens(tokens);
        }
        generics.gt_token.to_tokens(tokens);
    }
}

impl parse::Parse for Class {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis: syn::Visibility = input.parse()?;
        let class_token: kw::class = input.parse()?;
        let ident: syn::Ident = input.parse()?;

        let ahead = input.fork();
        let generics: Option<Generics> = ahead.parse().ok();
        if generics.is_some() {
            input.advance_to(&ahead);
        }

        let colon_token: Option<syn::Token![:]> = input.parse()?;
        let intf: Option<syn::Ident> = input.parse()?;

        let ahead = input.fork();
        let intf_generics: Option<Generics> = input.parse().ok();
        if intf_generics.is_some() {
            input.advance_to(&ahead);
        }
        if let Some(ref intf) = intf_generics {
            return Err(syn::Error::new(
                intf.types.span(),
                "Virt Class can't support generics",
            ));
        }

        let mut others = vec![];
        let mut cpps = vec![];

        let content;
        let brace_token = syn::braced!(content in input);
        let mut methods = Vec::new();
        let mut visitor = SelfVisitor(&ident, &generics);
        while !content.is_empty() {
            let ahead = content.fork();
            if let Ok(mut f) = ahead.parse::<ImportFn>() {
                content.advance_to(&ahead);
                if f.recv.is_none() {
                    return Err(syn::Error::new(f.ident.span(), "not found self paramenter"));
                }
                if f.variadic.is_some()
                    && Attr::get_any_attr(&["virt", "interface"], &attrs).is_some()
                {
                    return Err(syn::Error::new(
                        f.variadic.span(),
                        "interface can't support variadic member function",
                    ));
                }
                f.accept(&mut visitor)?;
                methods.push(f);
                continue;
            }
            let item = content.parse::<syn::Item>()?;
            match Cpp::from_item(&item)? {
                Some(item) => cpps.push(item),
                None => others.push(item),
            }
        }
        let this = Self {
            cpps,
            attrs,
            vis,
            class_token,
            ident,
            generics,
            colon_token,
            intf,
            intf_generics,
            brace_token,
            methods,
            others,
        };
        Ok(this)
    }
}

impl Class {
    pub fn class_accept<T: ClassVisitor>(&mut self, visitor: &T) {
        for f in self.methods.iter_mut() {
            f.class_accept(visitor);
        }
    }
}

pub struct Methods<'a>(pub &'a Class, pub &'a [&'a ImportFn], pub &'a syn::Path);

impl ToTokens for Methods<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.0.ident;
        let generics = &self.0.generics;
        let methods = hicc_methods_ident(ident);

        let mut fields = vec![];
        for f in self.1.iter() {
            fields.push(ImportField(f, Some(self.0)));
        }

        for f in self.0.methods.iter() {
            fields.push(ImportField(f, Some(self.0)));
        }
        let destroy = hicc_fn_ident(&format_ident!("destroy"));
        let unique = hicc_fn_ident(&format_ident!("unique"));
        let write = hicc_fn_ident(&format_ident!("write"));
        let make_ref = hicc_fn_ident(&format_ident!("make_ref"));
        let size_of = hicc_fn_ident(&format_ident!("size_of"));
        let impl_generics = ImplGenerics(&self.0.generics, self.2);

        quote! {
            #[repr(C)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            struct #methods #impl_generics {
                #destroy: Option<extern "C" fn(#ident #generics)>,
                #unique: Option<extern "C" fn(#ident #generics) -> #ident #generics>,
                #make_ref: Option<extern "C" fn(*const ()) -> #ident #generics>,
                #size_of: Option<extern "C" fn() -> usize>,
                #write: Option<extern "C" fn(*mut #ident #generics, #ident #generics)>,
                #(#fields),*
            }
        }
        .to_tokens(tokens);
    }
}

struct CppComments<'a>(&'a Class);

impl ToTokens for CppComments<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Some(cpp) = Attr::get_attr("cpp", &self.0.attrs) else {
            return;
        };
        let Ok(Some(cpp)) = cpp.value("class") else {
            return;
        };
        let cpp = format!("cpp class: `{cpp}`\n");
        quote! {
            #[doc = #cpp]
        }
        .to_tokens(tokens);
    }
}

pub struct Struct<'a>(pub &'a Class, pub &'a syn::Path);

impl ToTokens for Struct<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.0.vis;
        let ident = &self.0.ident;
        let methods = hicc_methods_ident(ident);
        let impl_generics = ImplGenerics(&self.0.generics, self.1);
        let generics = &self.0.generics;
        let docs = Comments(&self.0.attrs);
        let cpp_comments = CppComments(self.0);

        quote! {
            #cpp_comments
            #docs
            #[repr(C)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #vis struct #ident #impl_generics {
                methods: &'static #methods #generics,
                obj: *const (),
            }
        }
        .to_tokens(tokens);
    }
}

pub struct Trait<'a>(pub &'a Interface);

impl ToTokens for Trait<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = &self.0.vis;
        let ident = &self.0.ident;

        let mut items = vec![];
        for f in self.0.methods.iter() {
            items.push(TraitFnSignature(f, None));
        }
        let docs = Comments(&self.0.attrs);

        let trait_decl = if let Some(ref super_ident) = self.0.intf {
            quote! { #ident : #super_ident }
        } else {
            quote! { #ident}
        };
        quote! {
            #docs
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #vis trait #trait_decl {
                #(#items;)*
            }

        }
        .to_tokens(tokens);
    }
}

pub struct ImplMember<'a>(pub &'a ImportFn, pub &'a Class);

impl ToTokens for ImplMember<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.0.ident;
        let cpp_comments = CppFnComments(self.0);
        let docs = Comments(&self.0.attrs);
        if self.0.variadic.is_none() {
            let sig = Signature(self.0, None);
            let args = CallArguments(self.0);
            quote! {
                #cpp_comments
                #docs
                #sig
                {
                    (self.methods.#name.unwrap())(#args)
                }
            }
            .to_tokens(tokens);
        } else {
            let vis = &self.0.vis;
            let ty = TypeBareFn(self.0, Some(self.1));
            let member = quote! {
                #cpp_comments
                #docs
                #vis fn #name(&self) -> #ty {
                    self.methods.#name.unwrap()
                }
            };
            member.to_tokens(tokens);
        }
    }
}

pub type Interface = Class;

pub struct TraitImplFn<'a>(pub &'a Class, pub &'a ImportFn);

impl ToTokens for TraitImplFn<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let output = &self.1.output;
        let inputs = self.1.inputs.iter().collect::<Vec<_>>();
        let args = inputs.iter().map(|arg| &arg.name).collect::<Vec<_>>();
        let class = &self.0.ident;
        let name = &self.1.ident;
        let unsafety = &self.1.unsafety;
        let Some(recv) = &self.1.recv else {
            unreachable!();
        };
        let reference = &recv.reference;
        let mutability = &recv.mutability;
        if let Some((ref and_, ref ref_)) = reference {
            quote!{
            #unsafety extern "C" fn #name(_1_: #and_ #ref_ #mutability #class, #(#inputs),*) #output {
                let _1_ = unsafe { &mut *_1_.obj.cast::<T>().cast_mut() };
                _1_.#name(#(#args),*)
            }
            }.to_tokens(tokens);
        } else {
            quote! {
            #unsafety extern "C" fn #name(_1_: #class, #(#inputs),*) #output {
                let _1_ = ::std::mem::ManuallyDrop::new(_1_);
                let _1_ = unsafe { ::std::boxed::Box::from_raw(_1_.obj.cast::<T>().cast_mut()) };
                _1_.#name(#(#args),*)
            }
            }
            .to_tokens(tokens);
        }
    }
}
