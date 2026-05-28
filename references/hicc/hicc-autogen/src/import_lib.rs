use super::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::discouraged::Speculative;

#[allow(dead_code)]
pub struct ClassInLib {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub generics: Option<Generics>,
    pub colon_token: Option<syn::Token![:]>,
    pub intf: Option<syn::Ident>,
    pub intf_generics: Option<Generics>,
    pub methods: Vec<ImportFn>,
    pub others: Vec<syn::Item>,
}

impl parse::Parse for ClassInLib {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let head: Head = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let generics: Option<Generics> = input.parse().ok();
        let colon_token: Option<syn::Token![:]> = input.parse()?;
        let intf: Option<syn::Ident> = input.parse()?;
        let intf_generics: Option<Generics> = input.parse().ok();
        let content;
        syn::braced!(content in input);
        let mut methods = vec![];
        let mut others = vec![];
        while !content.is_empty() {
            let ahead = content.fork();
            if let Ok(f) = ahead.parse::<ImportFn>() {
                content.advance_to(&ahead);
                methods.push(f);
                continue;
            }
            let item = content.parse::<syn::Item>()?;
            others.push(item);
        }
        Ok(Self {
            attrs: head.attrs,
            vis: head.vis,
            ident,
            generics,
            colon_token,
            intf,
            intf_generics,
            methods,
            others,
        })
    }
}

impl ToTokens for ClassInLib {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        quote!(class).to_tokens(tokens);
        self.ident.to_tokens(tokens);
        if let Some(ref g) = self.generics {
            g.to_tokens(tokens);
        }
        if let Some(ref c) = self.colon_token {
            c.to_tokens(tokens);
        }
        if let Some(ref i) = self.intf {
            i.to_tokens(tokens);
            if let Some(ref ig) = self.intf_generics {
                ig.to_tokens(tokens);
            }
        }
        let mut body = proc_macro2::TokenStream::new();
        for f in &self.methods {
            f.to_tokens(&mut body);
        }
        for item in &self.others {
            item.to_tokens(&mut body);
        }
        quote!({ #body }).to_tokens(tokens);
    }
}

#[allow(dead_code)]
pub struct ImportLib {
    pub attrs: Vec<syn::Attribute>,
    pub funcs: Vec<ImportFn>,
    pub items: Vec<syn::Item>,
    pub cpps: Vec<Cpp>,
    pub decls: Vec<ClassDecl>,
    pub classes: Vec<ClassInLib>,
    pub hicc: syn::Path,
    pub span: Span,
}

impl parse::Parse for ImportLib {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let span = input.span();
        let attrs = input.call(syn::Attribute::parse_inner)?;
        let hicc = if Attr::get_attr("in_hicc", &attrs).is_some() {
            syn::parse2::<syn::Path>(quote! { crate }).unwrap()
        } else {
            syn::parse2::<syn::Path>(quote! { ::hicc }).unwrap()
        };

        let mut funcs = vec![];
        let mut items = vec![];
        let mut decls = vec![];
        let mut cpps = vec![];
        let mut classes = vec![];

        while !input.is_empty() {
            let ahead = input.fork();
            if let Ok(decl) = ahead.parse::<ClassDecl>() {
                input.advance_to(&ahead);
                decls.push(decl);
                continue;
            }
            let ahead = input.fork();
            if let Ok(class_in_lib) = ahead.parse::<ClassInLib>() {
                input.advance_to(&ahead);
                let class_name = class_in_lib.ident.to_string();
                let generics = &class_in_lib.generics;
                let mut remaining = vec![];
                for mut f in class_in_lib.methods {
                    if f.recv.is_some() {
                        replace_self_in_fn(&mut f, &class_in_lib.ident, generics)?;
                        remaining.push(f);
                    } else {
                        replace_self_in_fn(&mut f, &class_in_lib.ident, generics)?;
                        validate_no_generic_params(&f, generics)?;
                        let fn_name = f.ident.to_string();
                        if Attr::get_attr("member", &f.attrs).is_none() {
                            if let Some(ref generics) = class_in_lib.generics {
                                let generics_str = quote!(#generics).to_string();
                                f.attrs.push(syn::parse_quote! {
                                    #[member(class = #class_name, generics = #generics_str, method = #fn_name)]
                                });
                            } else {
                                f.attrs.push(syn::parse_quote! {
                                    #[member(class = #class_name, method = #fn_name)]
                                });
                            }
                        }
                        if f.recv.is_some() {
                            return Err(syn::Error::new(
                                f.recv.span(),
                                "only support global function",
                            ));
                        }
                        // Mangle ident to avoid collisions when multiple classes
                        // have associated functions with the same name (e.g. `new`).
                        // The original fn name is preserved in #[member(method = "...")]
                        // so generate_member can produce the correct impl block name.
                        f.ident = format_ident!("{}_{}", class_name, fn_name);
                        funcs.push(f);
                    }
                }
                classes.push(ClassInLib {
                    methods: remaining,
                    attrs: class_in_lib.attrs,
                    vis: class_in_lib.vis,
                    ident: class_in_lib.ident,
                    generics: class_in_lib.generics,
                    colon_token: class_in_lib.colon_token,
                    intf: class_in_lib.intf,
                    intf_generics: class_in_lib.intf_generics,
                    others: class_in_lib.others,
                });
                continue;
            }
            let ahead = input.fork();
            if let Ok(f) = ahead.parse::<ImportFn>() {
                input.advance_to(&ahead);
                if f.recv.is_some() {
                    return Err(syn::Error::new(
                        f.recv.span(),
                        "only support global function",
                    ));
                }
                funcs.push(f);
                continue;
            }
            let item = input.parse::<syn::Item>()?;
            match Cpp::from_item(&item)? {
                Some(item) => cpps.push(item),
                None => items.push(item),
            }
        }

        let mut class_idents = ClassIdents::new();
        class_idents.set_hicc(hicc.clone());
        class_idents.append_decls(&decls);
        for class_in_lib in &classes {
            class_idents.append_with_generics(class_in_lib.ident.clone(), &class_in_lib.generics);
        }
        for f in funcs.iter_mut() {
            f.class_accept(&class_idents);
        }

        Ok(Self {
            attrs,
            funcs,
            items,
            cpps,
            decls,
            classes,
            hicc,
            span,
        })
    }
}

impl ImportLib {
    pub fn generate(&self) -> parse::Result<Vec<syn::Item>> {
        let mut codes = self
            .items
            .iter()
            .map(|item| {
                syn::parse2::<syn::Item>(quote! {
                    #[allow(non_camel_case_types)]
                    #[allow(non_snake_case)]
                    #item
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        self.generate_decls(&mut codes)?;
        self.generate_struct(&mut codes)?;
        self.generate_link_name(&mut codes)?;
        self.generate_function(&mut codes)?;
        self.generate_member(&mut codes)?;
        self.generate_interface(&mut codes)?;
        self.generate_import_classes(&mut codes)?;
        Ok(codes)
    }

    fn default_link_name(lib: &str) -> String {
        format!("_hicc_export_methods_lib{}", ident_string(lib))
    }

    fn lib_info(&self) -> parse::Result<(Option<String>, String)> {
        match (
            Attr::get_value("lib", &self.attrs),
            Attr::get_value("link_name", &self.attrs),
        ) {
            (Ok(Some(lib)), Ok(Some(link_name))) => {
                Ok((Some(lib.to_string()), Self::default_link_name(&link_name)))
            }
            (Ok(Some(lib)), _) => Ok((Some(lib.to_string()), Self::default_link_name(&lib))),
            (_, Ok(Some(link_name))) => Ok((None, Self::default_link_name(&link_name))),
            _ => Err(parse::Error::new(
                self.span,
                "not found #![lib = ...] and #![link_name = ...]",
            )),
        }
    }

    fn struct_name(&self) -> parse::Result<syn::Ident> {
        let lib = match self.lib_info()? {
            (Some(lib), _) => lib,
            (None, link_name) => link_name,
        };
        Ok(format_ident!("_HiccImportedLib_{}", ident_string(&lib)))
    }

    fn link_name(&self) -> parse::Result<(syn::Ident, String)> {
        let (_, link_name) = self.lib_info()?;
        Ok((format_ident!("__{}", ident_string(&link_name)), link_name))
    }

    fn generate_decls(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        for decl in self.decls.iter() {
            if decl.equal_token.is_some() {
                codes.push(syn::parse2::<syn::Item>(quote! { #decl })?);
            }
        }
        Ok(())
    }

    fn generate_struct(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let mut fields = vec![];
        for f in self.funcs.iter() {
            fields.push(ImportField(f, None));
        }
        let tokens = quote! {
            #[repr(C)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            struct #ident {
                #(#fields),*
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_link_name(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        let (func_name, link_name) = self.link_name()?;
        let ident = self.struct_name()?;
        let (lib, _) = self.lib_info()?;
        let link_lib = lib.map(|lib| quote!(#[link(name = #lib)]));
        #[rustversion::since(1.82.0)]
        fn init_uns() -> Option<TokenStream> {
            Some(quote!(unsafe))
        }
        #[rustversion::before(1.82.0)]
        const fn init_uns() -> Option<TokenStream> {
            None
        }

        let uns = init_uns();
        let tokens = quote! {
            #link_lib
            #uns extern "C" {
                #[link_name = #link_name]
                #[allow(none_snake_case)]
                fn #func_name() -> &'static #ident;
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let hicc = &self.hicc;
        let tokens = quote! {
            impl #hicc::ImportLib for #ident {
                fn import() -> &'static #ident {
                    static HICC_METHODS: ::std::sync::OnceLock<&'static #ident> = ::std::sync::OnceLock::new();
                    HICC_METHODS.get_or_init(|| {
                        unsafe { #func_name() }
                    })
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_function(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        for f in self.funcs.iter() {
            if Attr::get_any_attr(&["member", "virt", "interface"], &f.attrs).is_some() {
                continue;
            }
            if f.variadic.is_none() {
                self.generate_common_function(f, codes)?;
            } else {
                self.generate_variadic_function(f, codes)?;
            }
        }
        Ok(())
    }

    fn generate_common_function(
        &self,
        f: &ImportFn,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let sig = Signature(f, None);
        let args = CallArguments(f);
        let name = &f.ident;
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;
        let tokens = quote! {
            #docs
            #fun_comments
            #[allow(non_snake_case)]
            #sig {
                use #hicc::ImportLib;
                (#ident::import().#name)(#args)
            }
        };

        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_variadic_function(
        &self,
        f: &ImportFn,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let ty = TypeBareFn(f, None);
        let vis = &f.vis;
        let name = &f.ident;
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;
        let tokens = quote! {
            #docs
            #fun_comments
            #vis fn #name() -> #ty {
                    use #hicc::ImportLib;
                    #ident::import().#name
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_member(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        for f in self.funcs.iter() {
            let Some(attr) = Attr::get_attr("member", &f.attrs) else {
                continue;
            };
            let (Ok(Some(class)), Ok(Some(name))) = (attr.value("class"), attr.value("method"))
            else {
                return Err(syn::Error::new(
                    attr.span(),
                    "not found #[member(class = ..., method = ...)]",
                ));
            };

            let generics = attr
                .value("generics")
                .unwrap_or(None)
                .and_then(|s| syn::parse_str::<Generics>(&s).ok());
            let class = string_2_path(class);
            let name = format_ident!("{}", name);
            if f.variadic.is_none() {
                self.generate_common_member(f, &class, &name, generics, codes)?;
            } else {
                self.generate_variadic_member(f, &class, &name, generics, codes)?;
            }
        }
        Ok(())
    }

    fn generate_common_member(
        &self,
        f: &ImportFn,
        class: &syn::Path,
        member: &syn::Ident,
        generics: Option<Generics>,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let sig = Signature(f, Some(member));
        let args = CallArguments(f);
        let name = &f.ident;
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;

        let tokens = if let Some(ref g) = generics {
            let impl_g = Self::impl_generics(g, hicc);
            quote! {
                #[allow(non_snake_case)]
                impl #impl_g #class #g {
                    #docs
                    #fun_comments
                    #sig {
                        use #hicc::ImportLib;
                        (#ident::import().#name)(#args)
                    }
                }
            }
        } else {
            quote! {
                #[allow(non_snake_case)]
                impl #class {
                    #docs
                    #fun_comments
                    #sig {
                        use #hicc::ImportLib;
                        (#ident::import().#name)(#args)
                    }
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn impl_generics(g: &Generics, hicc: &syn::Path) -> proc_macro2::TokenStream {
        let mut stream = proc_macro2::TokenStream::new();
        g.lt_token.to_tokens(&mut stream);
        for (i, ty) in g.types.iter().enumerate() {
            if i > 0 {
                quote!(,).to_tokens(&mut stream);
            }
            ty.to_tokens(&mut stream);
            quote! { : #hicc::AbiType + 'static }.to_tokens(&mut stream);
        }
        g.gt_token.to_tokens(&mut stream);
        stream
    }

    fn generate_variadic_member(
        &self,
        f: &ImportFn,
        class: &syn::Path,
        member: &syn::Ident,
        generics: Option<Generics>,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let name = &f.ident;
        let vis = &f.vis;
        let ty = TypeBareFn(f, None);
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;

        let tokens = if let Some(ref g) = generics {
            let impl_g = Self::impl_generics(g, hicc);
            quote! {
                #[allow(non_snake_case)]
                impl #impl_g #class #g {
                    #docs
                    #fun_comments
                    #vis fn #member() -> #ty {
                        use #hicc::ImportLib;
                        #ident::import().#name
                    }
                }
            }
        } else {
            quote! {
                #[allow(non_snake_case)]
                impl #class {
                    #docs
                    #fun_comments
                    #vis fn #member() -> #ty {
                        use #hicc::ImportLib;
                        #ident::import().#name
                    }
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_interface(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        let ident = self.struct_name()?;
        for f in self.funcs.iter() {
            let Some(attr) = Attr::get_any_attr(&["virt", "interface"], &f.attrs) else {
                continue;
            };
            let Ok(Some(name)) = attr.value("name") else {
                return Err(syn::Error::new(
                    attr.span(),
                    "not found #[interface(name = ...)]",
                ));
            };
            if f.variadic.is_some() {
                return Err(syn::Error::new(
                    f.variadic.span(),
                    "can't support variadic parameter",
                ));
            }

            let intf = string_2_path(name);
            let Some(syn::Type::Path(syn::TypePath {
                qself: None,
                path: ref class,
            })) = f.return_cabi_type()
            else {
                return Err(syn::Error::new(f.output.span(), "should return class type"));
            };
            let class = match attr.value("class") {
                Ok(Some(class)) => {
                    let ident = format_ident!("{class}");
                    let Ok(class) = syn::parse2::<syn::Path>(quote!(#ident)) else {
                        return Err(syn::Error::new(attr.span(), "wrong class value"));
                    };
                    class
                }
                _ => class.clone(),
            };

            let method = attr.value("method").unwrap_or(None);

            let mut it = f.inputs.iter();
            let Some(arg0) = it.next() else {
                return Err(syn::Error::new(
                    f.ident.span(),
                    "should be fn(::hicc::Interface<type>,...) -> type",
                ));
            };
            let inputs = it.collect::<Vec<_>>();
            let args = inputs.iter().map(|arg| &arg.name).collect::<Vec<_>>();
            let arg0 = &arg0.name;

            let name = &f.ident;
            let vis = &f.vis;
            let to_intf = hicc_fn_ident(&format_ident!("to_interface"));

            let func_ident = &f.ident;

            let hicc = &self.hicc;
            if let Some(method) = method {
                let method = format_ident!("{}", method);
                let tokens = quote! {
                    #[allow(non_snake_case)]
                    impl #class {
                        #vis fn #method<T: #intf>(#arg0: T, #(#inputs),*) -> Self {
                            #func_ident(#arg0, #(#args)*)
                        }
                    }
                };
                codes.push(syn::parse2::<syn::Item>(tokens)?);
            }

            let tokens = quote! {
                 #[allow(non_snake_case)]
                 #vis fn #func_ident<T: #intf>(#arg0: T, #(#inputs),*) -> #class {
                    use #hicc::ImportLib;
                    unsafe {
                        (#ident::import().#name)(#class::#to_intf(#arg0), #(#args)*)
                    }
                }
            };
            codes.push(syn::parse2::<syn::Item>(tokens)?);
        }
        Ok(())
    }

    fn generate_import_classes(&self, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        let hicc = &self.hicc;
        for class_in_lib in &self.classes {
            if class_in_lib.methods.is_empty() && class_in_lib.others.is_empty() {
                continue;
            }
            let tokens = quote! { #hicc::import_class! { #class_in_lib } };
            codes.push(syn::parse2(tokens)?);
        }
        Ok(())
    }
}

fn replace_self_in_fn(
    f: &mut ImportFn,
    class_ident: &syn::Ident,
    generics: &Option<Generics>,
) -> parse::Result<()> {
    struct ExpandSelf<'a>(&'a syn::Ident, &'a Option<Generics>);
    impl Visitor for ExpandSelf<'_> {
        fn visit_path(&mut self, path: &mut syn::Path) -> parse::Result<()> {
            if path.is_ident("Self") {
                let ident = self.0;
                let generics = self.1;
                *path = syn::parse2(quote!(#ident #generics)).unwrap();
            }
            Ok(())
        }
    }
    f.accept(&mut ExpandSelf(class_ident, generics))
}

fn validate_no_generic_params(f: &ImportFn, generics: &Option<Generics>) -> parse::Result<()> {
    let Some(ref gens) = generics else {
        return Ok(());
    };
    let params: Vec<&syn::Ident> = gens.types.iter().collect();
    for input in &f.inputs {
        if contains_generic_param(&input.ty, &params) {
            return Err(syn::Error::new(
                input.ty.span(),
                "associated functions in a generic class cannot use generic type parameters",
            ));
        }
    }
    if let syn::ReturnType::Type(_, ref ty) = f.output {
        if contains_generic_param(ty, &params) {
            return Err(syn::Error::new(
                ty.span(),
                "associated functions in a generic class cannot use generic type parameters",
            ));
        }
    }
    Ok(())
}

fn contains_generic_param(ty: &syn::Type, params: &[&syn::Ident]) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            for seg in &type_path.path.segments {
                if params.contains(&&seg.ident) {
                    return true;
                }
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            if contains_generic_param(ty, params) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(r) => contains_generic_param(&r.elem, params),
        syn::Type::Ptr(p) => contains_generic_param(&p.elem, params),
        syn::Type::Tuple(t) => t.elems.iter().any(|e| contains_generic_param(e, params)),
        syn::Type::Slice(s) => contains_generic_param(&s.elem, params),
        syn::Type::Array(a) => contains_generic_param(&a.elem, params),
        _ => false,
    }
}
