use super::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::discouraged::Speculative;

#[allow(dead_code)]
pub struct ImportLib {
    pub attrs: Vec<syn::Attribute>,
    pub funcs: Vec<ImportFn>,
    pub items: Vec<syn::Item>,
    pub cpps: Vec<Cpp>,
    pub decls: Vec<ClassDecl>,
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

        while !input.is_empty() {
            let ahead = input.fork();
            if let Ok(decl) = ahead.parse::<ClassDecl>() {
                input.advance_to(&ahead);
                decls.push(decl);
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
        for f in funcs.iter_mut() {
            f.class_accept(&class_idents);
        }

        Ok(Self {
            attrs,
            funcs,
            items,
            cpps,
            decls,
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

            let class = string_2_path(class);
            let name = format_ident!("{}", name);
            if f.variadic.is_none() {
                self.generate_common_member(f, &class, &name, codes)?;
            } else {
                self.generate_variadic_member(f, &class, &name, codes)?;
            }
        }
        Ok(())
    }

    fn generate_common_member(
        &self,
        f: &ImportFn,
        class: &syn::Path,
        member: &syn::Ident,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let sig = Signature(f, Some(member));
        let args = CallArguments(f);
        let name = &f.ident;
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;

        let tokens = quote! {
            #[allow(non_snake_case)]
            impl #class {
                #docs
                #fun_comments
                #sig {
                    use #hicc::ImportLib;
                    (#ident::import().#name)(#args)
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_variadic_member(
        &self,
        f: &ImportFn,
        class: &syn::Path,
        member: &syn::Ident,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let ident = self.struct_name()?;
        let name = &f.ident;
        let vis = &f.vis;
        let ty = TypeBareFn(f, None);
        let fun_comments = CppFnComments(f);
        let docs = Comments(&f.attrs);
        let hicc = &self.hicc;

        let tokens = quote! {
            #[allow(non_snake_case)]
            impl #class {
                #docs
                #fun_comments
                #vis fn #member() -> #ty {
                    use #hicc::ImportLib;
                    #ident::import().#name
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
}
