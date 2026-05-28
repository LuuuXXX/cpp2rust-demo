use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS2;
use quote::{format_ident, quote};
use syn::{
    fold::Fold, FnArg, GenericParam, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType,
    Signature, Type,
};

// ---- Helper: extract type identifier from a Type ----
fn extract_type_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            let seg = tp.path.segments.first()?;
            Some(seg.ident.clone())
        }
        _ => None,
    }
}

// ---- Helper: get the self-type's top-level path ident for unsupported detection ----
fn get_self_path_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => tp.path.segments.first().map(|s| s.ident.clone()),
        _ => None,
    }
}

// ---- Helper: collect Ident of all type/const generic params ----
fn generics_idents(generics: &Generics) -> Vec<Ident> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => Some(tp.ident.clone()),
            GenericParam::Const(cp) => Some(cp.ident.clone()),
            _ => None,
        })
        .collect()
}

// ---- Helper: generate bare generic param decls (stripping bounds from type params) ----
fn bare_generic_params(generics: &Generics) -> Vec<TS2> {
    generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                quote! { #i }
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                quote! { #lt }
            }
            GenericParam::Const(cp) => {
                let i = &cp.ident;
                let ty = &cp.ty;
                quote! { const #i: #ty }
            }
        })
        .collect()
}

// ---- Helper: generate `T: 'static` bounds for type params ----
fn static_bounds(generics: &Generics) -> Vec<TS2> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                Some(quote! { #i: 'static })
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                Some(quote! { #lt: 'static })
            }
            _ => None,
        })
        .collect()
}

// ---- Helper: generate `T: ValueType + 'static` bounds for type params ----
fn value_type_static_bounds(generics: &Generics) -> Vec<TS2> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                Some(quote! { #i: ::hicc_rs::ValueType + 'static })
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                Some(quote! { #lt: 'static })
            }
            _ => None,
        })
        .collect()
}

fn mk_where(gp: &[TS2]) -> TS2 {
    if gp.is_empty() {
        TS2::new()
    } else {
        quote! { where #(#gp),* }
    }
}

// ---- Check if a type param has a ValueType bound ----
fn has_value_type_bound(tp: &syn::TypeParam) -> bool {
    tp.bounds.iter().any(|bound| {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            trait_bound
                .path
                .segments
                .last()
                .map(|s| s.ident == "ValueType")
                .unwrap_or(false)
        } else {
            false
        }
    })
}

// ---- Generate bounds for ValueType impl: `T: ValueType<Type = IsClass>` for
// params with ValueType bounds, `T: 'static` for others, `'lt: 'static` for lifetimes ----
fn value_type_bounds(generics: &Generics) -> Vec<TS2> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                if has_value_type_bound(tp) {
                    Some(quote! { #i: ::hicc_rs::ValueType<Type = ::hicc_rs::IsClass> })
                } else {
                    Some(quote! { #i: 'static })
                }
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                Some(quote! { #lt: 'static })
            }
            GenericParam::Const(_) => None,
        })
        .collect()
}

// ---- Generate bounds for ClassMethods impl: `T: ValueType<Type = IsClass> + 'static`
// for params with ValueType bounds, `T: ValueType + 'static` for others ----
fn class_methods_bounds(generics: &Generics) -> Vec<TS2> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                if has_value_type_bound(tp) {
                    Some(quote! { #i: ::hicc_rs::ValueType<Type = ::hicc_rs::IsClass> + 'static })
                } else {
                    Some(quote! { #i: ::hicc_rs::ValueType + 'static })
                }
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                Some(quote! { #lt: 'static })
            }
            GenericParam::Const(_) => None,
        })
        .collect()
}

// ---- Helper: generate fn input type token ----
fn self_input_type(self_type: &Type, receiver: &FnArg) -> TS2 {
    match receiver {
        FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
            quote! { <&mut #self_type as ::hicc_rs::AbiType>::InputType }
        }
        FnArg::Receiver(r) if r.reference.is_some() => {
            quote! { <&#self_type as ::hicc_rs::AbiType>::InputType }
        }
        _ => {
            quote! { <#self_type as ::hicc_rs::AbiType>::InputType }
        }
    }
}

fn param_input_type(ty: &Type) -> TS2 {
    quote! { <#ty as ::hicc_rs::AbiType>::InputType }
}

fn param_from_abi(pat: &syn::Pat, ty: &Type) -> TS2 {
    quote! { let #pat = <#ty as ::hicc_rs::AbiType>::from_abi(#pat); }
}

fn self_from_abi(self_type: &Type, receiver: &FnArg, obj_ident: &Ident) -> TS2 {
    match receiver {
        FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
            quote! { let #obj_ident = <&mut #self_type as ::hicc_rs::AbiType>::from_abi(#obj_ident); }
        }
        FnArg::Receiver(r) if r.reference.is_some() => {
            quote! { let #obj_ident = <&#self_type as ::hicc_rs::AbiType>::from_abi(#obj_ident); }
        }
        _ => {
            quote! { let #obj_ident = <#self_type as ::hicc_rs::AbiType>::from_abi(#obj_ident); }
        }
    }
}

// ---- Check if return type contains references ----
fn contains_ref(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Path(tp) if tp.qself.is_none() => {
            for seg in &tp.path.segments {
                if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                    for arg in &a.args {
                        if let syn::GenericArgument::Type(t) = arg {
                            if contains_ref(t) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        Type::Tuple(tup) => tup.elems.iter().any(|t| contains_ref(t)),
        _ => false,
    }
}

// ---- Check if a method is just `panic!()` ----
fn is_panic_method(f: &ImplItemFn) -> bool {
    if f.block.stmts.len() != 1 {
        return false;
    }
    match &f.block.stmts[0] {
        syn::Stmt::Expr(syn::Expr::Macro(m), _) => m.mac.path.is_ident("panic"),
        syn::Stmt::Macro(m) => m.mac.path.is_ident("panic"),
        _ => false,
    }
}

// ---- Check if method is a declaration (no body) ----
fn is_empty_body(f: &ImplItemFn) -> bool {
    f.block.stmts.is_empty()
}

// ---- Check for unsupported pattern: return type is same generic as self with ref/ptr args ----
fn check_unsupported_pattern(self_ident: &Ident, sig: &Signature) -> Result<(), syn::Error> {
    if let ReturnType::Type(_, ret_ty) = &sig.output {
        if let Some(ret_ident) = get_self_path_ident(ret_ty) {
            if &ret_ident == self_ident {
                // Return type has same path as self. Check if any type arg is a ref or ptr.
                if has_ref_ptr_arg(ret_ty) {
                    return Err(syn::Error::new(
                        sig.ident.span(),
                        format!(
                            "unsupported: method `{}` returns the same generic type with a reference/pointer type argument, which would cause monomorphization recursion",
                            sig.ident
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn has_ref_ptr_arg(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            for seg in &tp.path.segments {
                if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                    for arg in &a.args {
                        if let syn::GenericArgument::Type(t) = arg {
                            match t {
                                Type::Reference(_) => return true,
                                Type::Ptr(_) => return true,
                                _ => {
                                    if has_ref_ptr_arg(t) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

// ---- Convert `&T` or `&mut T` to `&'static T` or `&'static mut T` ----
fn make_static_ref(ty: &Type) -> TS2 {
    match ty {
        Type::Reference(r) => {
            let inner = make_static_ref(&r.elem);
            if r.mutability.is_some() {
                quote! { &'static mut #inner }
            } else {
                quote! { &'static #inner }
            }
        }
        Type::Path(tp) if tp.qself.is_none() => {
            let segs: Vec<TS2> = tp
                .path
                .segments
                .iter()
                .map(|seg| {
                    let id = &seg.ident;
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        let args: Vec<TS2> = ab
                            .args
                            .iter()
                            .map(|arg| match arg {
                                syn::GenericArgument::Type(t) => make_static_ref(t),
                                syn::GenericArgument::Lifetime(_) => {
                                    quote! { 'static }
                                }
                                other => quote! { #other },
                            })
                            .collect();
                        quote! { #id<#(#args),*> }
                    } else {
                        quote! { #id }
                    }
                })
                .collect();
            quote! { #(#segs)::* }
        }
        Type::Tuple(tup) => {
            let elems: Vec<TS2> = tup.elems.iter().map(|t| make_static_ref(t)).collect();
            quote! { ( #(#elems),* ) }
        }
        Type::Ptr(p) => {
            let inner = make_static_ref(&p.elem);
            if p.mutability.is_some() {
                quote! { *mut #inner }
            } else {
                quote! { *const #inner }
            }
        }
        Type::Array(arr) => {
            let inner = make_static_ref(&arr.elem);
            quote! { [#inner; #arr.len] }
        }
        Type::Slice(s) => {
            let inner = make_static_ref(&s.elem);
            quote! { [#inner] }
        }
        other => quote! { #other },
    }
}

// ---- Generate methods struct ----
fn gen_methods_struct(
    struct_ident: &Ident,
    self_type: &Type,
    generics: &Generics,
    methods: &[&ImplItemFn],
) -> TS2 {
    let gp = bare_generic_params(generics);
    let sb = static_bounds(generics);
    let wc = mk_where(&sb);

    let fields: Vec<TS2> = methods
        .iter()
        .map(|f| {
            let name = &f.sig.ident;
            let fn_ty = gen_fn_ptr_type(&f.sig, self_type);
            if is_panic_method(f) {
                quote! { pub #name: Option<#fn_ty> }
            } else {
                quote! { pub #name: #fn_ty }
            }
        })
        .collect();

    if gp.is_empty() {
        quote! {
            #[repr(C)]
            #[allow(dead_code)]
            pub struct #struct_ident {
                #(#fields),*
            }
        }
    } else {
        quote! {
            #[repr(C)]
            #[allow(dead_code)]
            pub struct #struct_ident<#(#gp),*> #wc {
                #(#fields),*
            }
        }
    }
}

// ---- Generate function pointer type for a method ----
fn gen_fn_ptr_type(sig: &Signature, self_type: &Type) -> TS2 {
    let has_ref_ret = match &sig.output {
        ReturnType::Type(_, ty) => contains_ref(ty),
        _ => false,
    };
    let mut its = Vec::new();
    if let Some(first) = sig.inputs.first() {
        if let FnArg::Receiver(r) = first {
            let input_ty = self_input_type(self_type, first);
            its.push(if has_ref_ret {
                // If return has refs, also add 'static to self input refs
                let expanded =
                    make_static_ref(&syn::parse2::<Type>(quote! { #self_type }).unwrap());
                match r {
                    _ if r.reference.is_some() && r.mutability.is_some() => {
                        quote! { <&'static mut #expanded as ::hicc_rs::AbiType>::InputType }
                    }
                    _ if r.reference.is_some() => {
                        quote! { <&'static #expanded as ::hicc_rs::AbiType>::InputType }
                    }
                    _ => input_ty,
                }
            } else {
                input_ty
            });
        }
    }
    for input in sig.inputs.iter().skip(1) {
        if let FnArg::Typed(pt) = input {
            let ty = &*pt.ty;
            let input_ty = param_input_type(ty);
            if has_ref_ret && contains_ref(ty) {
                let static_ty = make_static_ref(ty);
                its.push(param_input_type(&syn::parse2(static_ty).unwrap()));
            } else {
                its.push(input_ty);
            }
        }
    }
    let rt = match &sig.output {
        ReturnType::Type(_, ty) => {
            if is_unit(ty) {
                TS2::new()
            } else if has_ref_ret {
                let static_ty = make_static_ref(ty);
                let parsed: Type = syn::parse2(static_ty).unwrap();
                quote! { -> <#parsed as ::hicc_rs::AbiType>::OutputType }
            } else {
                quote! { -> <#ty as ::hicc_rs::AbiType>::OutputType }
            }
        }
        _ => TS2::new(),
    };
    quote! { unsafe extern "C" fn(#(#its),*) #rt }
}

// ---- Generate global wrapper function for each method ----
fn gen_wrapper_fn(
    fn_ident: &Ident,
    self_type: &Type,
    generics: &Generics,
    method: &ImplItemFn,
) -> TS2 {
    let has_ref_ret = match &method.sig.output {
        ReturnType::Type(_, ty) => contains_ref(ty),
        _ => false,
    };
    let gp = bare_generic_params(generics);
    let sb = static_bounds(generics);
    let wc = mk_where(&sb);

    // Build parameter list for the extern "C" fn
    let obj_ident = Ident::new("obj", proc_macro2::Span::call_site());
    let mut abi_params = Vec::new();
    let mut extra_params = Vec::new();

    if let Some(first) = method.sig.inputs.first() {
        if matches!(first, FnArg::Receiver(_)) {
            let input_ty = if has_ref_ret {
                let static_ty = make_static_ref(self_type);
                let parsed: Type = syn::parse2(static_ty).unwrap();
                match first {
                    FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                        quote! { <&'static mut #parsed as ::hicc_rs::AbiType>::InputType }
                    }
                    FnArg::Receiver(r) if r.reference.is_some() => {
                        quote! { <&'static #parsed as ::hicc_rs::AbiType>::InputType }
                    }
                    _ => self_input_type(self_type, first),
                }
            } else {
                self_input_type(self_type, first)
            };
            abi_params.push(quote! { #obj_ident: #input_ty });
        }
    }

    for input in method.sig.inputs.iter().skip(1) {
        if let FnArg::Typed(pt) = input {
            let pat = &pt.pat;
            let ty = &*pt.ty;
            let input_ty = if has_ref_ret && contains_ref(ty) {
                let static_ty = make_static_ref(ty);
                let parsed: Type = syn::parse2(static_ty).unwrap();
                param_input_type(&parsed)
            } else {
                param_input_type(ty)
            };
            abi_params.push(quote! { #pat: #input_ty });
            extra_params.push((pat.clone(), ty.clone()));
        }
    }

    let ret_ty = match &method.sig.output {
        ReturnType::Type(_, ty) => {
            let inner: Type = (**ty).clone();
            if is_unit(&inner) {
                None
            } else if has_ref_ret {
                let static_ty = make_static_ref(&inner);
                let parsed: Type = syn::parse2(static_ty).unwrap();
                Some((inner, parsed))
            } else {
                Some((inner.clone(), inner))
            }
        }
        _ => None,
    };

    let rty = ret_ty
        .as_ref()
        .map(|(_, output_ty)| quote! { -> <#output_ty as ::hicc_rs::AbiType>::OutputType })
        .unwrap_or(TS2::new());

    let body = if is_empty_body(method) {
        // Declaration: generate method call
        let fn_name = &method.sig.ident;
        let call_args: Vec<TS2> = extra_params.iter().map(|(p, _)| quote! { #p }).collect();
        let method_self_call = if let Some(first) = method.sig.inputs.first() {
            match first {
                FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                    quote! { #obj_ident.#fn_name(#(#call_args),*) }
                }
                FnArg::Receiver(r) if r.reference.is_some() => {
                    quote! { #obj_ident.#fn_name(#(#call_args),*) }
                }
                _ => quote! { #obj_ident.#fn_name(#(#call_args),*) },
            }
        } else {
            quote! { #obj_ident.#fn_name(#(#call_args),*) }
        };

        let from_abi_self =
            self_from_abi(self_type, method.sig.inputs.first().unwrap(), &obj_ident);
        let from_abi_extra: Vec<TS2> = extra_params
            .iter()
            .map(|(p, t)| param_from_abi(p, t))
            .collect();

        if let Some((orig_ty, _)) = &ret_ty {
            quote! {
                #from_abi_self
                #(#from_abi_extra)*
                <#orig_ty as ::hicc_rs::AbiType>::into_abi({ #method_self_call })
            }
        } else {
            quote! {
                #from_abi_self
                #(#from_abi_extra)*
                #method_self_call
            }
        }
    } else {
        // Has custom body: generate ReplaceSelf + from_abi conversions
        let from_abi_self =
            self_from_abi(self_type, method.sig.inputs.first().unwrap(), &obj_ident);
        let from_abi_extra: Vec<TS2> = extra_params
            .iter()
            .map(|(p, t)| param_from_abi(p, t))
            .collect();

        // Replace `self` with `obj_ident` in the method body
        let body = method.block.clone();
        struct ReplaceSelf {
            replacement: Ident,
        }
        impl syn::fold::Fold for ReplaceSelf {
            fn fold_expr(&mut self, expr: syn::Expr) -> syn::Expr {
                match &expr {
                    syn::Expr::Path(p)
                        if p.attrs.is_empty() && p.qself.is_none() && p.path.is_ident("self") =>
                    {
                        syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::Path::from(self.replacement.clone()),
                        })
                    }
                    _ => syn::fold::fold_expr(self, expr),
                }
            }
        }
        let replaced = ReplaceSelf {
            replacement: obj_ident.clone(),
        }
        .fold_block(body);

        if let Some((orig_ty, _)) = &ret_ty {
            quote! {
                #from_abi_self
                #(#from_abi_extra)*
                <#orig_ty as ::hicc_rs::AbiType>::into_abi({ #replaced })
            }
        } else {
            quote! {
                #from_abi_self
                #(#from_abi_extra)*
                #replaced
            }
        }
    };

    let is_panic = is_panic_method(method);
    if is_panic {
        // panic methods still need the wrapper, but it'll never be called
        if gp.is_empty() {
            quote! {
                #[allow(unreachable_code)]
                unsafe extern "C" fn #fn_ident(#(#abi_params),*) #rty { #body }
            }
        } else {
            quote! {
                #[allow(unreachable_code)]
                unsafe extern "C" fn #fn_ident<#(#gp),*>(#(#abi_params),*) #rty #wc { #body }
            }
        }
    } else if gp.is_empty() {
        quote! {
            unsafe extern "C" fn #fn_ident(#(#abi_params),*) #rty { #body }
        }
    } else {
        quote! {
            unsafe extern "C" fn #fn_ident<#(#gp),*>(#(#abi_params),*) #rty #wc { #body }
        }
    }
}

// ---- Generate constructor function ----
fn gen_constructor(
    fn_ident: &Ident,
    struct_ident: &Ident,
    _self_type: &Type,
    generics: &Generics,
    methods: &[&ImplItemFn],
    wrapper_fn_idents: &[Ident],
) -> TS2 {
    let gp = bare_generic_params(generics);
    let vtsb = value_type_static_bounds(generics);
    let wc = mk_where(&vtsb);
    let type_args: Vec<TS2> = generics_idents(generics)
        .iter()
        .map(|i| quote! { #i })
        .collect();

    let fields: Vec<TS2> = methods
        .iter()
        .zip(wrapper_fn_idents.iter())
        .map(|(f, wrapper_ident)| {
            let name = &f.sig.ident;
            if is_panic_method(f) {
                quote! { #name: None }
            } else if type_args.is_empty() {
                quote! { #name: #wrapper_ident }
            } else {
                quote! { #name: #wrapper_ident::<#(#type_args),*> }
            }
        })
        .collect();

    let struct_init = if gp.is_empty() {
        quote! { #struct_ident { #(#fields),* } }
    } else {
        quote! { #struct_ident::<#(#type_args),*> { #(#fields),* } }
    };

    if gp.is_empty() {
        quote! {
            const fn #fn_ident() -> #struct_ident {
                #struct_init
            }
        }
    } else {
        quote! {
            const fn #fn_ident<#(#gp),*>() -> #struct_ident<#(#type_args),*> #wc {
                #struct_init
            }
        }
    }
}

// ---- Generate ValueType impl ----
fn gen_value_type(self_type: &Type, generics: &Generics) -> TS2 {
    let gp = bare_generic_params(generics);
    let vt = value_type_bounds(generics);
    let wc = mk_where(&vt);

    if gp.is_empty() {
        quote! {
            impl ::hicc_rs::ValueType for #self_type {
                const N: usize = 0;
                type Type = ::hicc_rs::IsClass;
                type Value = ::hicc_rs::IsValue;
            }
        }
    } else {
        quote! {
            impl<#(#gp),*> ::hicc_rs::ValueType for #self_type #wc {
                const N: usize = 0;
                type Type = ::hicc_rs::IsClass;
                type Value = ::hicc_rs::IsValue;
            }
        }
    }
}

// ---- Generate ClassMethods impl ----
fn gen_class_methods(
    self_type: &Type,
    generics: &Generics,
    methods_ident: &Ident,
    ctor_ident: &Ident,
) -> TS2 {
    let gp = bare_generic_params(generics);
    let cmb = class_methods_bounds(generics);
    let wc = mk_where(&cmb);

    let gpi: Vec<Ident> = generics_idents(generics);
    let type_args: Vec<TS2> = gpi.iter().map(|i| quote! { #i }).collect();

    let ctor_call = if type_args.is_empty() {
        quote! { #ctor_ident() }
    } else {
        quote! { #ctor_ident::<#(#type_args),*>() }
    };

    let methods_ty = if type_args.is_empty() {
        quote! { #methods_ident }
    } else {
        quote! { #methods_ident<#(#type_args),*> }
    };

    if gp.is_empty() {
        quote! {
            impl ::hicc_rs::ClassMethods for #self_type {
                type Methods = #methods_ty;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_methods(#ctor_call);
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(#ctor_call);
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(#ctor_call);
            }
        }
    } else {
        quote! {
            impl<#(#gp),*> ::hicc_rs::ClassMethods for #self_type #wc {
                type Methods = #methods_ty;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_methods(#ctor_call);
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(#ctor_call);
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(#ctor_call);
            }
        }
    }
}

fn is_unit(ty: &Type) -> bool {
    match ty {
        Type::Tuple(t) => t.elems.is_empty(),
        _ => false,
    }
}

// =====================================================================
// Core generate function
// =====================================================================

fn generate(imp: ItemImpl, _in_hicc: bool) -> Result<TS2, syn::Error> {
    let self_type = &imp.self_ty;
    let type_ident = extract_type_ident(self_type).ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "cannot extract type name from impl",
        )
    })?;

    let self_ident = get_self_path_ident(self_type).ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "cannot determine self type path",
        )
    })?;

    // Collect methods and validate
    let mut methods: Vec<&ImplItemFn> = Vec::new();

    for item in &imp.items {
        if let ImplItem::Fn(f) = item {
            // Check for unsupported pattern
            check_unsupported_pattern(&self_ident, &f.sig)?;
            methods.push(f);
        }
    }

    if methods.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "no methods found in impl block",
        ));
    }

    let generics = &imp.generics;
    let struct_ident = format_ident!("Hicc{}Methods", type_ident);

    // Wrapper function names: hicc_{typename}_{methodname}
    let type_name_lower = type_ident.to_string().to_lowercase();
    let wrapper_fn_idents: Vec<Ident> = methods
        .iter()
        .map(|f| format_ident!("hicc_{}_{}", type_name_lower, f.sig.ident))
        .collect();

    let ctor_ident = format_ident!("hicc_{}_methods", type_name_lower);

    // Generate all parts
    let methods_struct = gen_methods_struct(&struct_ident, self_type, generics, &methods);

    let wrapper_fns: Vec<TS2> = methods
        .iter()
        .zip(&wrapper_fn_idents)
        .map(|(m, id)| gen_wrapper_fn(id, self_type, generics, m))
        .collect();

    let constructor = gen_constructor(
        &ctor_ident,
        &struct_ident,
        self_type,
        generics,
        &methods,
        &wrapper_fn_idents,
    );

    let value_type = gen_value_type(self_type, generics);

    let class_methods = gen_class_methods(self_type, generics, &struct_ident, &ctor_ident);

    let output = quote! {
        #methods_struct
        #(#wrapper_fns)*
        #constructor
        #value_type
        #class_methods
    };

    Ok(output)
}

// =====================================================================
// Entry points
// =====================================================================

pub(crate) fn export_class_inner(input: TS2, in_hicc: bool) -> Result<TokenStream, syn::Error> {
    if let Ok(imp) = syn::parse2::<ItemImpl>(input.clone()) {
        let out = generate(imp, in_hicc)?;
        let out_str = out.to_string();
        let out_str = if in_hicc {
            let re = regex_lite::Regex::new(r"::\s*hicc_rs\s*::").unwrap();
            re.replace_all(&out_str, "crate::").to_string()
        } else {
            out_str
        };
        match out_str.parse::<TS2>() {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                &format!("reparse: {}", e),
            )),
        }
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "export_class requires an impl block",
        ))
    }
}
