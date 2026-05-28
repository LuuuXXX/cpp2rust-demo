use super::*;
use quote::{format_ident, quote};
use syn::parse::discouraged::Speculative;
use syn::{self, parse, spanned::Spanned};

#[allow(dead_code)]
pub struct ImportClass {
    pub attrs: Vec<syn::Attribute>,
    pub class: Vec<Class>,
    pub intfs: Vec<Interface>,
    pub items: Vec<syn::Item>,
    pub cpps: Vec<Cpp>,
    pub decls: Vec<ClassDecl>,
    pub hicc: syn::Path,
}

impl parse::Parse for ImportClass {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_inner)?;
        let hicc = if Attr::get_attr("in_hicc", &attrs).is_some() {
            syn::parse2::<syn::Path>(quote!(crate)).unwrap()
        } else {
            syn::parse2::<syn::Path>(quote!(::hicc)).unwrap()
        };
        let mut class = vec![];
        let mut intfs = vec![];
        let mut items = vec![];
        let mut decls = vec![];
        let mut cpps = vec![];

        while !input.is_empty() {
            let ahead = input.fork();
            if ahead.parse::<Head>().is_ok() {
                let ahead = input.fork();
                if let Ok(decl) = ahead.parse::<ClassDecl>() {
                    input.advance_to(&ahead);
                    decls.push(decl);
                    continue;
                }
                let item = input.parse::<Class>()?;

                if Attr::get_any_attr(&["virt", "interface"], &item.attrs).is_some() {
                    intfs.push(item);
                } else {
                    class.push(item);
                }
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
        class_idents.append_class(&class);
        class_idents.append_decls(&decls);
        for item in class.iter_mut() {
            item.class_accept(&class_idents);
        }
        for item in intfs.iter_mut() {
            item.class_accept(&class_idents);
        }

        Ok(Self {
            attrs,
            class,
            intfs,
            items,
            decls,
            cpps,
            hicc,
        })
    }
}

impl ImportClass {
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
        for decl in self.decls.iter() {
            self.generate_class_decl(decl, &mut codes)?;
        }
        for intf in self.intfs.iter() {
            self.generate_interface_decl(intf, &mut codes)?;
        }
        for class in self.class.iter() {
            let intf = self.get_interface(&class.intf)?;
            self.generate_class(class, &mut codes)?;
            if let Some(intf) = intf {
                self.generate_interface_impl(class, intf, &mut codes)?;
            }
        }
        Ok(codes)
    }

    pub fn get_intf_methods<'a>(&'a self, name: &Option<syn::Ident>) -> Vec<&'a ImportFn> {
        let mut methods = vec![];
        self.get_methods(name, &mut methods);
        methods
    }

    fn get_methods<'a>(&'a self, name: &Option<syn::Ident>, methods: &mut Vec<&'a ImportFn>) {
        let Ok(Some(intf)) = self.get_interface(name) else {
            return;
        };
        self.get_methods(&intf.intf, methods);
        for f in intf.methods.iter() {
            methods.push(f);
        }
    }

    pub fn get_interface(&self, name: &Option<syn::Ident>) -> parse::Result<Option<&Interface>> {
        let Some(ident) = name else {
            return Ok(None);
        };
        for intf in self.intfs.iter() {
            if ident == &intf.ident {
                return Ok(Some(intf));
            }
        }
        Err(syn::Error::new(
            name.span(),
            format!("not found interface {ident}"),
        ))
    }

    fn generate_class_decl(
        &self,
        decl: &ClassDecl,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        if decl.equal_token.is_some() {
            codes.push(syn::parse2::<syn::Item>(quote! { #decl })?);
        }
        Ok(())
    }

    fn generate_class(&self, class: &Class, codes: &mut Vec<syn::Item>) -> parse::Result<()> {
        let destroy = hicc_fn_ident(&format_ident!("destroy"));
        let unique = hicc_fn_ident(&format_ident!("unique"));
        let write = hicc_fn_ident(&format_ident!("write"));
        let make_ref = hicc_fn_ident(&format_ident!("make_ref"));
        let size_of = hicc_fn_ident(&format_ident!("size_of"));
        let new = hicc_fn_ident(&format_ident!("new"));
        let ident = &class.ident;
        let intf_methods = self.get_intf_methods(&class.intf);
        let methods = Methods(class, &intf_methods, &self.hicc);
        let methods_ident = hicc_methods_ident(ident);
        let item = Struct(class, &self.hicc);

        let tokens = quote!(#item);
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let tokens = quote!(#methods);
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let mut fields = vec![];

        for f in intf_methods.iter() {
            fields.push(&f.ident);
        }

        for f in class.methods.iter() {
            fields.push(&f.ident);
        }
        let generics = &class.generics;
        let impl_generics = ImplGenerics(generics, &self.hicc);

        let tokens = quote! {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            impl #impl_generics #methods_ident #generics {
                const fn #new() -> Self {
                    Self {
                        #destroy: None,
                        #unique: None,
                        #write: None,
                        #make_ref: None,
                        #size_of: None,
                        #(#fields: None),*
                    }
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let tokens = quote! {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            impl #impl_generics Drop for #ident #generics {
                fn drop(&mut self) {
                    (self.methods.#destroy.unwrap())(Self{methods: self.methods, obj: self.obj, level: self.level})
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let ident_name = ident.to_string();
        let tokens = quote! {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            impl #impl_generics ::std::fmt::Debug for #ident #generics {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    ::std::write!(f, "[AbiClass] {}: {:p}, level={}, {}-methods", #ident_name, self.obj, self.level, ::std::mem::size_of_val(self.methods) / ::std::mem::size_of::<*const ()>())
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let hicc = &self.hicc;
        let tokens = quote! {
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            impl #impl_generics #hicc::AbiClass for #ident #generics {
                fn get_raw_obj(&self) -> *const () {
                    self.obj
                }
                fn get_level(&self) -> usize {
                    self.level
                }
                unsafe fn make_ref(&self, obj: *const (), level: usize) -> Self {
                    (self.methods.#make_ref.unwrap())(obj, level)
                }
                unsafe fn into_unique(self) -> Self {
                    (self.methods.#unique.unwrap())(self)
                }
                fn write(&mut self, other: Self) {
                    (self.methods.#write.unwrap())(self, other)
                }
                fn size_of(&self) -> usize {
                    (self.methods.#size_of.unwrap())()
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let others = &class.others;
        let mut members = vec![];
        for f in intf_methods.iter() {
            members.push(ImplMember(f, class));
        }

        for f in class.methods.iter() {
            members.push(ImplMember(f, class));
        }

        let tokens = quote! {
            #[allow(non_snake_case)]
            #[allow(non_upper_case_globals)]
            impl #impl_generics #ident #generics {
                #(#others)*
                #(#members)*
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        Ok(())
    }

    fn generate_interface_decl(
        &self,
        intf: &Interface,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let trait_decl = Trait(intf);
        let tokens = quote!(#trait_decl);
        codes.push(syn::parse2::<syn::Item>(tokens)?);
        Ok(())
    }

    fn generate_interface_impl(
        &self,
        class: &Class,
        intf: &Interface,
        codes: &mut Vec<syn::Item>,
    ) -> parse::Result<()> {
        let intf_ident = &intf.ident;
        let intf_impl = hicc_ty_ident(&format_ident!("{}_{}", &class.ident, &intf.ident));

        let class_ident = &class.ident;
        let methods = hicc_methods_ident(class_ident);
        let new = hicc_fn_ident(&format_ident!("new"));
        let fname = hicc_fn_ident(&format_ident!("to_interface"));
        let vis = &class.vis;
        let generics = &class.generics;
        let impl_generics = ImplGenerics(generics, &self.hicc);
        let hicc = &self.hicc;
        let tokens = quote! {
            #[allow(non_snake_case)]
            impl #impl_generics #class_ident #generics {
                #vis fn #fname<T: #intf_ident>(val: T) -> #hicc::Interface<Self> {
                    #intf_impl::#new(val)
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let tokens = quote! {
            #[repr(transparent)]
            #[allow(non_camel_case_types)]
            struct #intf_impl<T: #intf_ident>(T);
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let tokens = quote! {
            impl<T: #intf_ident> #intf_impl<T> {
                fn #new(val: T) -> #hicc::Interface<#class_ident> {
                    use std::boxed::Box;
                    unsafe { #hicc::Interface::new(#class_ident {
                        methods: &Self::methods,
                        obj: Box::into_raw(Box::new(val)).cast::<()>(),
                        level: 0,
                    }) }
                }
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        let mut impl_funcs = vec![];
        let mut fields = vec![];
        let intf_methods = self.get_intf_methods(&class.intf);
        for f in intf_methods.iter() {
            impl_funcs.push(TraitImplFn(class, f));
            fields.push(&f.ident);
        }

        let destroy = hicc_fn_ident(&format_ident!("destroy"));
        let tokens = quote! {
            impl<T: #intf_ident> #intf_impl<T> {
                extern "C" fn #destroy(this: #class_ident) {
                    let this = ::std::mem::ManuallyDrop::new(this);
                    let _ = unsafe { ::std::boxed::Box::from_raw(this.obj.cast::<T>().cast_mut()) };
                }
                #(#impl_funcs)*
                const methods: #methods = #methods {
                    #destroy: Some(Self::#destroy),
                    #(#fields: Some(Self::#fields),)*
                    ..#methods::#new()
                };
            }
        };
        codes.push(syn::parse2::<syn::Item>(tokens)?);

        Ok(())
    }
}
