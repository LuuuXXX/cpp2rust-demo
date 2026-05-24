use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, TokenStream as TS2, TokenTree};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashSet};
use syn::{
    fold::Fold, Expr, FnArg, GenericArgument, GenericParam, Generics, Ident, ImplItem, ImplItemFn,
    ItemImpl, ItemMod, PathArguments, ReturnType, Signature, Type,
};

const MAX_DEPTH: u32 = 4;
const ERR_DEPTH: &str = "ref/ptr depth exceeds maximum supported (4 levels)";

// =====================================================================
// export_class — wraps Rust type methods as C-compatible functions
// =====================================================================

#[proc_macro_attribute]
pub fn export_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: TS2 = item.into();
    let input = replace_semicolons(input);
    let in_hicc = attr.to_string().contains("in_hicc");
    match export_class_inner(input, in_hicc) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

fn replace_semicolons(stream: TS2) -> TS2 {
    replace_semicolons_depth(stream, 0u32, false)
}

fn replace_semicolons_depth(stream: TS2, brace_depth: u32, in_non_brace_group: bool) -> TS2 {
    let mut out = TS2::new();
    for token in stream {
        match token {
            TokenTree::Group(g) => {
                match g.delimiter() {
                    Delimiter::Brace => {
                        let inner = replace_semicolons_depth(g.stream(), brace_depth + 1, false);
                        let mut ng = Group::new(g.delimiter(), inner);
                        ng.set_span(g.span());
                        out.extend(std::iter::once(TokenTree::Group(ng)));
                    }
                    _ => {
                        // Non-brace groups (brackets, parens, angle) disable replacement inside
                        let inner = replace_semicolons_depth(g.stream(), brace_depth, true);
                        let mut ng = Group::new(g.delimiter(), inner);
                        ng.set_span(g.span());
                        out.extend(std::iter::once(TokenTree::Group(ng)));
                    }
                }
            }
            // Replace ; only at brace_depth == 1 and NOT inside any non-brace group
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

fn export_class_inner(input: TS2, in_hicc: bool) -> Result<TokenStream, syn::Error> {
    // Try ItemImpl first (existing behavior: #[export_class] impl Type { ... })
    if let Ok(imp) = syn::parse2::<ItemImpl>(input.clone()) {
        let out = generate(imp, in_hicc)?;
        // in_hicc replacement is done in generate
        return Ok(out.into());
    }
    // Try ItemMod (new behavior: #[export_class] mod name { impl ... })
    let item_mod: ItemMod = syn::parse2(input)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), &e.to_string()))?;
    let out = generate_mod(item_mod, in_hicc)?;
    Ok(out.into())
}

// ---- Depth classification ----

type DepthGroup = u32;

fn depth_trait_name(d: DepthGroup) -> Ident {
    Ident::new(
        match d {
            1 => "Depth0_3",
            2 => "Depth0_2",
            3 => "Depth0_1",
            4 => "Depth0_0",
            _ => "Depth0_3", // fallback, should not be called with 0
        },
        proc_macro2::Span::call_site(),
    )
}

// ---- Generate all boilerplate for export_class ----

/// Compute which type params are used in method signatures.
fn compute_used_type_params(
    methods: &[(&ImplItemFn, DepthGroup)],
    gs: &HashSet<Ident>,
) -> HashSet<Ident> {
    let mut used = HashSet::new();
    for (f, _) in methods {
        for input in &f.sig.inputs {
            let ty: Type = match input {
                FnArg::Receiver(r) => {
                    let m = r.mutability;
                    syn::parse2(quote! { &#m Self }).unwrap()
                }
                FnArg::Typed(pt) => *pt.ty.clone(),
            };
            collect_used(&ty, gs, &mut used);
        }
        if let ReturnType::Type(_, ty) = &f.sig.output {
            collect_used(ty, gs, &mut used);
        }
    }
    used
}

fn collect_used(ty: &Type, gs: &HashSet<Ident>, acc: &mut HashSet<Ident>) {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            if tp.path.segments.len() == 1
                && tp.path.leading_colon.is_none()
                && gs.contains(&tp.path.segments[0].ident)
            {
                acc.insert(tp.path.segments[0].ident.clone());
            }
            for seg in &tp.path.segments {
                if let PathArguments::AngleBracketed(ref a) = seg.arguments {
                    for arg in &a.args {
                        if let GenericArgument::Type(t) = arg {
                            collect_used(t, gs, acc);
                        }
                    }
                }
            }
        }
        Type::Reference(r) => {
            collect_used(&r.elem, gs, acc);
        }
        Type::Ptr(p) => {
            collect_used(&p.elem, gs, acc);
        }
        _ => {}
    }
}

fn generate(imp: ItemImpl, in_hicc: bool) -> Result<TS2, syn::Error> {
    let self_type = &imp.self_ty;
    let type_ident = extract_type_ident(self_type).ok_or_else(|| {
        syn::Error::new(proc_macro2::Span::call_site(), "cannot determine type name")
    })?;

    let type_param_idents: Vec<Ident> = imp
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => Some(tp.ident.clone()),
            _ => None,
        })
        .collect();
    let generic_set: HashSet<Ident> = type_param_idents.iter().cloned().collect();

    let mut methods: Vec<(&ImplItemFn, DepthGroup)> = Vec::new();
    for item in &imp.items {
        if let ImplItem::Fn(f) = item {
            if f.sig.generics.params.len() > 0 {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    &format!(
                        "method `{}` has generic parameters, not supported",
                        f.sig.ident
                    ),
                ));
            }
            let depth = method_depth(&f.sig, &generic_set);
            if depth > MAX_DEPTH {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    &format!("method `{}`: {}", f.sig.ident, ERR_DEPTH),
                ));
            }
            methods.push((f, depth));
        }
    }
    if methods.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "no methods found",
        ));
    }

    let max_depth = *methods.iter().map(|(_, d)| d).max().unwrap_or(&0);
    let has_specialization = max_depth > 0;
    let class_ident = format_ident!("{}Class", type_ident);
    let methods_ident = format_ident!("{}Methods", type_ident);
    let trait_ident = format_ident!("{}ClassMethods", type_ident);
    let array_ident: Ident = format_ident!("{}MethodArray", type_ident);

    let used_set: HashSet<Ident> = compute_used_type_params(&methods, &generic_set);
    let all_arg_idents: Vec<TS2> = imp
        .generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                quote! { #i }
            }
            GenericParam::Const(cp) => {
                let i = &cp.ident;
                quote! { #i }
            }
            GenericParam::Lifetime(lp) => {
                let lt = &lp.lifetime;
                quote! { #lt }
            }
        })
        .collect();
    // For struct/class type arguments, exclude lifetimes (structs don't store them)
    let st_arg_idents: Vec<TS2> = all_arg_idents
        .iter()
        .filter(|t| {
            let s = t.to_string();
            !s.starts_with('\'')
        })
        .cloned()
        .collect();
    let _type_idents: Vec<_> = imp
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                Some(quote! { #i })
            }
            _ => None,
        })
        .collect();

    let vt = gen_value_type(self_type, &imp.generics, &used_set);
    let wr = gen_wrapper(&class_ident, &imp.generics);
    // Use st_arg_idents for struct/class type arguments (no lifetimes)
    let ad = gen_adapter_blocks(
        &class_ident,
        &imp.generics,
        &methods,
        self_type,
        &used_set,
        &st_arg_idents,
    );
    let at = gen_array_type(&array_ident, methods.len());
    let ms = gen_methods_struct(
        &methods_ident,
        &imp.generics,
        &methods,
        self_type,
        &used_set,
    );
    let ac = if has_specialization {
        gen_array_specialised(
            &trait_ident,
            &class_ident,
            &array_ident,
            &imp.generics,
            &methods,
            max_depth,
            &used_set,
            &st_arg_idents,
        )
    } else {
        gen_array_simple(
            &class_ident,
            &array_ident,
            &imp.generics,
            &methods,
            &used_set,
            max_depth,
            &st_arg_idents,
        )
    };
    let cm = if has_specialization {
        gen_class_specialised(
            self_type,
            &imp.generics,
            &st_arg_idents,
            &array_ident,
            &trait_ident,
            &class_ident,
            max_depth,
            &used_set,
        )
    } else {
        gen_class_simple(
            self_type,
            &imp.generics,
            &st_arg_idents,
            &array_ident,
            &class_ident,
            &used_set,
            max_depth,
        )
    };

    let output = quote! { #vt #wr #(#ad)* #at #ms #ac #cm };
    let out_str = output.to_string();
    if out_str.contains("AnyClass") {
        eprintln!("=== ANY OUTPUT ===");
        eprintln!("{}", out_str);
        eprintln!("=== END ===");
    }
    let out_str = if in_hicc {
        // Replace ALL instances of :: hicc_rs :: (with any spacing) with crate::
        use regex_lite::Regex;
        let re = Regex::new(r"::\s*hicc_rs\s*::").unwrap();
        re.replace_all(&out_str, "crate::").to_string()
    } else {
        out_str
    };
    match out_str.parse::<TS2>() {
        Ok(t) => Ok(t),
        Err(e) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            &format!("reparse: {}", e),
        )),
    }
}

/// Extract the type name from path types and trait objects.
/// Handles:
///   - `MyType<T>` -> "MyType"
///   - `&'a dyn Foo<T>` -> "Foo"
///   - `dyn Bar` -> "Bar"
fn extract_type_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            let segs: Vec<_> = tp.path.segments.iter().collect();
            if segs.is_empty() {
                return None;
            }
            let name = segs
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("_");
            Some(Ident::new(&name, proc_macro2::Span::call_site()))
        }
        Type::Reference(r) => extract_type_ident(&r.elem),
        Type::TraitObject(to) => to.bounds.first().and_then(|bound| match bound {
            syn::TypeParamBound::Trait(t) => {
                let segs: Vec<_> = t.path.segments.iter().collect();
                if segs.is_empty() {
                    return None;
                }
                let name = segs
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("_");
                Some(Ident::new(&name, proc_macro2::Span::call_site()))
            }
            _ => None,
        }),
        _ => None,
    }
}

// ---- Generating where clause helpers ----

fn value_where(generics: &Generics, used_set: &HashSet<Ident>) -> TS2 {
    let mut b: Vec<TS2> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) if used_set.contains(&tp.ident) => {
                let i = &tp.ident;
                Some(quote! { #i: ::hicc_rs::ValueType })
            }
            _ => None,
        })
        .collect();
    if let Some(wc) = &generics.where_clause {
        for pred in &wc.predicates {
            b.push(quote! { #pred });
        }
    }
    if b.is_empty() {
        TS2::new()
    } else {
        quote! { where #(#b),* }
    }
}

fn combined_where(generics: &Generics, depth: DepthGroup, used_set: &HashSet<Ident>) -> TS2 {
    let mut b: Vec<TS2> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) if used_set.contains(&tp.ident) => {
                let i = &tp.ident;
                Some(quote! { #i: ::hicc_rs::ValueType })
            }
            _ => None,
        })
        .collect();
    if depth > 0 {
        let dt = depth_trait_name(depth);
        b.extend(generics.params.iter().filter_map(|p| match p {
            GenericParam::Type(tp) if used_set.contains(&tp.ident) => {
                let i = &tp.ident;
                Some(quote! { #i::Depth: ::hicc_rs::#dt })
            }
            _ => None,
        }));
    }
    if let Some(wc) = &generics.where_clause {
        for pred in &wc.predicates {
            b.push(quote! { #pred });
        }
    }
    if b.is_empty() {
        TS2::new()
    } else {
        quote! { where #(#b),* }
    }
}

/// Generate class_ident<args> if args non-empty, else just class_ident
fn class_ty(class_ident: &Ident, arg_idents: &[TS2]) -> TS2 {
    if arg_idents.is_empty() {
        quote! { #class_ident }
    } else {
        quote! { #class_ident<#(#arg_idents),*> }
    }
}

// ---- ValueType impl ----

fn gen_combined_where(generics: &Generics, depth: DepthGroup, used_set: &HashSet<Ident>) -> TS2 {
    let mut b: Vec<TS2> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                Some(quote! { #i: ::hicc_rs::ValueType })
            }
            _ => None,
        })
        .collect();
    if depth > 0 {
        let dt = depth_trait_name(depth);
        b.extend(generics.params.iter().filter_map(|p| match p {
            GenericParam::Type(tp) if used_set.contains(&tp.ident) => {
                let i = &tp.ident;
                Some(quote! { #i::Depth: ::hicc_rs::#dt })
            }
            _ => None,
        }));
    }
    if let Some(wc) = &generics.where_clause {
        for pred in &wc.predicates {
            b.push(quote! { #pred });
        }
    }
    if b.is_empty() {
        TS2::new()
    } else {
        quote! { where #(#b),* }
    }
}

fn gen_depth_where(generics: &Generics, depth: DepthGroup, used_set: &HashSet<Ident>) -> TS2 {
    if depth == 0 {
        return TS2::new();
    }
    let dt = depth_trait_name(depth);
    let b: Vec<_> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) if used_set.contains(&tp.ident) => {
                let i = &tp.ident;
                Some(quote! { #i::Depth: ::hicc_rs::#dt })
            }
            _ => None,
        })
        .collect();
    if b.is_empty() {
        TS2::new()
    } else {
        quote! { where #(#b),* }
    }
}

fn gen_value_type(self_type: &Type, generics: &Generics, used_set: &HashSet<Ident>) -> TS2 {
    let gp: Vec<_> = generics.params.iter().map(|p| quote! { #p }).collect();
    if gp.is_empty() {
        return quote! { impl ::hicc_rs::ValueType for #self_type { ::hicc_rs::ExportClass!(); } };
    }
    let wc = value_where(generics, used_set);
    quote! { impl<#(#gp),*> ::hicc_rs::ValueType for #self_type #wc { ::hicc_rs::ExportClass!(); } }
}

// ---- Wrapper struct ----

fn gen_wrapper(class_ident: &Ident, generics: &Generics) -> TS2 {
    // Filter out lifetime params — they're unused in the struct fields
    let gp: Vec<_> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Lifetime(_) => None,
            other => Some(quote! { #other }),
        })
        .collect();
    let type_fields: Vec<_> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => {
                let i = &tp.ident;
                Some(quote! { #i })
            }
            _ => None,
        })
        .collect();
    if gp.is_empty() {
        return quote! { #[allow(dead_code, non_camel_case_types)] struct #class_ident; };
    }
    if type_fields.is_empty() {
        quote! { #[allow(dead_code, non_camel_case_types)] struct #class_ident<#(#gp),*> (); }
    } else {
        quote! { #[allow(dead_code, non_camel_case_types)] struct #class_ident<#(#gp),*> ( #(#type_fields),* ); }
    }
}

// ---- Adapter function blocks ----

fn gen_adapter_blocks(
    class_ident: &Ident,
    generics: &Generics,
    methods: &[(&ImplItemFn, DepthGroup)],
    self_type: &Type,
    used_set: &HashSet<Ident>,
    arg_idents: &[TS2],
) -> Vec<TS2> {
    let mut by_depth: BTreeMap<DepthGroup, Vec<&ImplItemFn>> = BTreeMap::new();
    for (f, d) in methods {
        by_depth.entry(*d).or_default().push(f);
    }
    by_depth
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(d, fns)| {
            gen_adapter_block(
                class_ident,
                generics,
                fns,
                *d,
                self_type,
                used_set,
                arg_idents,
            )
        })
        .collect()
}

fn gen_adapter_block(
    class_ident: &Ident,
    generics: &Generics,
    methods: &[&ImplItemFn],
    depth: DepthGroup,
    self_type: &Type,
    used_set: &HashSet<Ident>,
    arg_idents: &[TS2],
) -> TS2 {
    let has_a = generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Lifetime(lp) if lp.lifetime.ident == "a"));
    let fns: Vec<_> = methods
        .iter()
        .map(|f| build_adapter(f, self_type, has_a))
        .collect();
    let gp: Vec<_> = generics.params.iter().map(|p| quote! { #p }).collect();
    if gp.is_empty() {
        return quote! { impl #class_ident { #(#fns)* } };
    }
    let wc = combined_where(generics, depth, used_set);
    let ct = class_ty(class_ident, arg_idents);
    quote! { impl<#(#gp),*> #ct #wc { #(#fns)* } }
}

fn build_adapter(method: &ImplItemFn, self_type: &Type, use_lt: bool) -> TS2 {
    let fn_name = &method.sig.ident;
    let obj: Ident = format_ident!("obj");
    let has_body = !method.block.stmts.is_empty();
    let by_val = method.sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.reference.is_none()),
    );
    let is_mut = method.sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.mutability.is_some()),
    );
    let extra: Vec<_> = method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Typed(pt) => Some((pt.pat.clone(), pt.ty.clone())),
            _ => None,
        })
        .collect();

    let rabi = if by_val {
        quote! { <#self_type as ::hicc_rs::AbiType>::InputType }
    } else if use_lt {
        // Expose 'a explicitly in the parameter type to constrain the return lifetime
        if is_mut {
            quote! { <&'a mut #self_type as ::hicc_rs::AbiType>::InputType }
        } else {
            quote! { <&'a #self_type as ::hicc_rs::AbiType>::InputType }
        }
    } else if is_mut {
        quote! { <&mut #self_type as ::hicc_rs::AbiType>::InputType }
    } else {
        quote! { <&#self_type as ::hicc_rs::AbiType>::InputType }
    };
    let mut abi_params = vec![quote! { #obj: #rabi }];
    let en: Vec<_> = extra.iter().map(|(p, _)| p.clone()).collect();
    for (pat, ty) in &extra {
        abi_params.push(quote! { #pat: <#ty as ::hicc_rs::AbiType>::InputType });
    }

    let oty = match &method.sig.output {
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
        .map(|t| {
            if use_lt {
                let transformed = add_lt_to_type(t);
                quote! { -> <#transformed as ::hicc_rs::AbiType>::OutputType }
            } else {
                quote! { -> <#t as ::hicc_rs::AbiType>::OutputType }
            }
        })
        .unwrap_or(quote! {});

    let saf = if by_val {
        quote! { #self_type }
    } else if is_mut {
        quote! { &mut #self_type }
    } else {
        quote! { &#self_type }
    };
    let ec: Vec<_> = extra
        .iter()
        .map(|(p, t)| quote! { let #p = <#t as ::hicc_rs::AbiType>::from_abi(#p); })
        .collect();
    let sc = quote! { let #obj = <#saf as ::hicc_rs::AbiType>::from_abi(#obj); };

    let body = if has_body {
        let mut rp = ReplaceSelf {
            replacement: obj.clone(),
        };
        let rb = rp.fold_block(method.block.clone());
        if let Some(ref t) = oty {
            quote! { #sc #(#ec)* <#t as ::hicc_rs::AbiType>::into_abi({ #rb }) }
        } else {
            quote! { #sc #(#ec)* #rb }
        }
    } else if let Some(ref t) = oty {
        quote! { #sc #(#ec)* <#t as ::hicc_rs::AbiType>::into_abi(#obj.#fn_name(#(#en),*)) }
    } else {
        quote! { #sc #(#ec)* #obj.#fn_name(#(#en),*); }
    };

    let attrs: Vec<_> = method.attrs.iter().map(|a| quote! { #a }).collect();
    quote! { #(#attrs)* unsafe extern "C" fn #fn_name(#(#abi_params),*) #rty { #body } }
}

// ---- Array type + Methods struct ----

fn gen_array_type(ident: &Ident, count: usize) -> TS2 {
    quote! { type #ident = [*const (); #count]; }
}

fn gen_methods_struct(
    struct_ident: &Ident,
    generics: &Generics,
    methods: &[(&ImplItemFn, DepthGroup)],
    self_type: &Type,
    used_set: &HashSet<Ident>,
) -> TS2 {
    let needs_lt = methods.iter().any(|(f, _)| type_returns_ref(&f.sig));
    fn type_returns_ref(sig: &Signature) -> bool {
        if let ReturnType::Type(_, ty) = &sig.output {
            contains_ref(ty)
        } else {
            false
        }
    }
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
    // Only add 'a if the generics don't already have a lifetime named 'a
    // Keep all generic params (including lifetimes) since field types reference self_type generics
    let mut gp: Vec<TS2> = generics.params.iter().map(|p| quote! { #p }).collect();
    // Only add 'a if not already present (for fn_ptr_type_with_lt injected lifetime)
    let has_a = generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Lifetime(lp) if lp.lifetime.ident == "a"));
    if needs_lt && !has_a {
        gp.insert(0, quote! { 'a });
    }
    let is_panic_method = |f: &&ImplItemFn| -> bool {
        if f.block.stmts.len() != 1 {
            return false;
        }
        match &f.block.stmts[0] {
            syn::Stmt::Expr(syn::Expr::Macro(m), _) => m.mac.path.is_ident("panic"),
            syn::Stmt::Macro(m) => m.mac.path.is_ident("panic"),
            _ => false,
        }
    };
    let fields: Vec<_> = methods
        .iter()
        .map(|(f, d)| {
            let n = &f.sig.ident;
            let needs_option = *d > 0 || is_panic_method(f);
            if needs_lt {
                let ft = fn_ptr_type_with_lt(&f.sig, self_type);
                if needs_option {
                    quote! { pub(crate) #n: Option<#ft> }
                } else {
                    quote! { pub(crate) #n: #ft }
                }
            } else {
                let ft = fn_ptr_type(&f.sig, self_type);
                if needs_option {
                    quote! { pub(crate) #n: Option<#ft> }
                } else {
                    quote! { pub(crate) #n: #ft }
                }
            }
        })
        .collect();
    if gp.is_empty() {
        return quote! { #[repr(C)] #[allow(dead_code)] pub(crate) struct #struct_ident { #(#fields),* } };
    }
    let wc_base = value_where(generics, used_set);
    let needs_lifetime_bound = needs_lt
        && generics
            .params
            .iter()
            .any(|p| matches!(p, GenericParam::Type(_)));
    let wc = if needs_lifetime_bound {
        let lt_bounds: Vec<TS2> = generics
            .params
            .iter()
            .filter_map(|p| {
                if let GenericParam::Type(tp) = p {
                    let i = &tp.ident;
                    Some(quote! { #i: 'a })
                } else {
                    None
                }
            })
            .collect();
        if wc_base.is_empty() {
            quote! { where #(#lt_bounds),* }
        } else {
            quote! { #wc_base, #(#lt_bounds),* }
        }
    } else {
        wc_base
    };
    quote! { #[repr(C)] #[allow(dead_code)] pub(crate) struct #struct_ident<#(#gp),*> #wc { #(#fields),* } }
}

fn add_lt_to_type(ty: &Type) -> TS2 {
    match ty {
        Type::Reference(r) => {
            let inner = add_lt_to_type(&r.elem);
            if r.mutability.is_some() {
                quote! { &'a mut #inner }
            } else {
                quote! { &'a #inner }
            }
        }
        Type::Path(tp) if tp.qself.is_none() => {
            let segs: Vec<TS2> = tp
                .path
                .segments
                .iter()
                .map(|seg| {
                    let ident = &seg.ident;
                    if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                        let new_args: Vec<TS2> = a
                            .args
                            .iter()
                            .map(|arg| match arg {
                                syn::GenericArgument::Type(t) => add_lt_to_type(t),
                                other => quote! { #other },
                            })
                            .collect();
                        quote! { #ident < #(#new_args),* > }
                    } else {
                        quote! { #ident }
                    }
                })
                .collect();
            quote! { #(#segs)::* }
        }
        Type::Tuple(tup) => {
            let elems: Vec<TS2> = tup.elems.iter().map(|t| add_lt_to_type(t)).collect();
            quote! { ( #(#elems),* ) }
        }
        _ => quote! { #ty },
    }
}

fn fn_ptr_type_with_lt(sig: &Signature, self_type: &Type) -> TS2 {
    let mut its = Vec::new();
    let bv = sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.reference.is_none()),
    );
    let mt = sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.mutability.is_some()),
    );
    if sig.inputs.first().is_some() {
        its.push(if bv {
            quote! { <#self_type as ::hicc_rs::AbiType>::InputType }
        } else if mt {
            quote! { <&mut #self_type as ::hicc_rs::AbiType>::InputType }
        } else {
            quote! { <&#self_type as ::hicc_rs::AbiType>::InputType }
        });
    }
    for i in sig.inputs.iter() {
        if let FnArg::Typed(pt) = i {
            let ty = &*pt.ty;
            its.push(quote! { <#ty as ::hicc_rs::AbiType>::InputType });
        }
    }
    let rt = match &sig.output {
        ReturnType::Type(_, ty) => {
            if is_unit(ty) {
                quote! {}
            } else {
                let transformed = add_lt_to_type(ty);
                quote! { -> <#transformed as ::hicc_rs::AbiType>::OutputType }
            }
        }
        _ => quote! {},
    };
    quote! { unsafe extern "C" fn(#(#its),*) #rt }
}

fn fn_ptr_type(sig: &Signature, self_type: &Type) -> TS2 {
    let mut its = Vec::new();
    let bv = sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.reference.is_none()),
    );
    let mt = sig.inputs.first().map_or(
        false,
        |a| matches!(a, FnArg::Receiver(r) if r.mutability.is_some()),
    );
    if sig.inputs.first().is_some() {
        its.push(if bv {
            quote! { <#self_type as ::hicc_rs::AbiType>::InputType }
        } else if mt {
            quote! { <&mut #self_type as ::hicc_rs::AbiType>::InputType }
        } else {
            quote! { <&#self_type as ::hicc_rs::AbiType>::InputType }
        });
    }
    for i in sig.inputs.iter() {
        if let FnArg::Typed(pt) = i {
            let ty = &*pt.ty;
            its.push(quote! { <#ty as ::hicc_rs::AbiType>::InputType });
        }
    }
    let rt = match &sig.output {
        ReturnType::Type(_, ty) => {
            if is_unit(ty) {
                quote! {}
            } else {
                quote! { -> <#ty as ::hicc_rs::AbiType>::OutputType }
            }
        }
        _ => quote! {},
    };
    quote! { unsafe extern "C" fn(#(#its),*) #rt }
}

// ---- Method array constant ----

fn gen_array_simple(
    class_ident: &Ident,
    array_ident: &Ident,
    generics: &Generics,
    methods: &[(&ImplItemFn, DepthGroup)],
    used_set: &HashSet<Ident>,
    max_depth: DepthGroup,
    arg_idents: &[TS2],
) -> TS2 {
    let fp: Vec<_> = methods
        .iter()
        .map(|(f, _)| {
            let n = &f.sig.ident;
            quote! { Self::#n as *const () }
        })
        .collect();
    let gp: Vec<_> = generics.params.iter().map(|p| quote! { #p }).collect();
    if gp.is_empty() {
        return quote! { impl #class_ident { const fn new_methods() -> #array_ident { [#(#fp),*] } } };
    }
    let wc = gen_combined_where(generics, max_depth, used_set);
    let ct = class_ty(class_ident, arg_idents);
    quote! { impl<#(#gp),*> #ct #wc { const fn new_methods() -> #array_ident { [#(#fp),*] } } }
}

fn gen_array_specialised(
    trait_ident: &Ident,
    class_ident: &Ident,
    array_ident: &Ident,
    generics: &Generics,
    methods: &[(&ImplItemFn, DepthGroup)],
    max_depth: DepthGroup,
    used_set: &HashSet<Ident>,
    arg_idents: &[TS2],
) -> TS2 {
    // Preserve original generic params (including bounds like Flag1 = IsClass)
    let gp: Vec<TS2> = generics.params.iter().map(|p| quote! { #p }).collect();
    let is_panic_method = |f: &&ImplItemFn| -> bool {
        if f.block.stmts.len() != 1 {
            return false;
        }
        match &f.block.stmts[0] {
            syn::Stmt::Expr(syn::Expr::Macro(m), _) => m.mac.path.is_ident("panic"),
            syn::Stmt::Macro(m) => m.mac.path.is_ident("panic"),
            _ => false,
        }
    };
    let defs: Vec<_> = methods
        .iter()
        .map(|(f, d)| {
            let n = &f.sig.ident;
            if *d == 0 && !is_panic_method(f) {
                quote! { Self::#n as *const () }
            } else {
                quote! { 0 as *const () }
            }
        })
        .collect();
    let specs: Vec<_> = methods
        .iter()
        .map(|(f, _)| {
            let n = &f.sig.ident;
            if is_panic_method(f) {
                quote! { 0 as *const () }
            } else {
                quote! { Self::#n as *const () }
            }
        })
        .collect();
    let wc_v = value_where(generics, used_set);
    let wc_c = combined_where(generics, max_depth, used_set);
    let t = quote! { #[allow(non_camel_case_types)] trait #trait_ident { const METHODS: #array_ident; } };
    let ct = class_ty(class_ident, arg_idents);
    let d = if gp.is_empty() {
        quote! { impl #trait_ident for #class_ident { default const METHODS: #array_ident = [#(#defs),*]; } }
    } else {
        quote! { impl<#(#gp),*> #trait_ident for #ct #wc_v { default const METHODS: #array_ident = [#(#defs),*]; } }
    };
    let s = if gp.is_empty() {
        quote! { impl #trait_ident for #class_ident { const METHODS: #array_ident = [#(#specs),*]; } }
    } else {
        quote! { impl<#(#gp),*> #trait_ident for #ct #wc_c { const METHODS: #array_ident = [#(#specs),*]; } }
    };
    quote! { #t #d #s }
}
// ---- ClassMethods impl ----

fn gen_class_simple(
    self_type: &Type,
    generics: &Generics,
    arg_idents: &[TS2],
    array_ident: &Ident,
    class_ident: &Ident,
    used_set: &HashSet<Ident>,
    _max_depth: DepthGroup,
) -> TS2 {
    let gp: Vec<_> = generics.params.iter().map(|p| quote! { #p }).collect();
    let wc = value_where(generics, used_set);
    if gp.is_empty() {
        quote! {
            impl ::hicc_rs::ClassMethods for #self_type {
                type Methods = #array_ident;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_methods(#class_ident::new_methods());
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(#class_ident::new_methods());
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(#class_ident::new_methods());
            }
        }
    } else {
        // Use turbofish syntax when args present (to avoid < as comparison operator)
        let new_ct = if arg_idents.is_empty() {
            quote! { #class_ident::new_methods() }
        } else {
            quote! { #class_ident::<#(#arg_idents),*>::new_methods() }
        };
        quote! {
            impl<#(#gp),*> ::hicc_rs::ClassMethods for #self_type #wc {
                type Methods = #array_ident;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_methods(#new_ct);
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(#new_ct);
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(#new_ct);
            }
        }
    }
}

fn gen_class_specialised(
    self_type: &Type,
    generics: &Generics,
    arg_idents: &[TS2],
    array_ident: &Ident,
    trait_ident: &Ident,
    class_ident: &Ident,
    _max_depth: DepthGroup,
    used_set: &HashSet<Ident>,
) -> TS2 {
    let gp: Vec<_> = generics.params.iter().map(|p| quote! { #p }).collect();
    let wc = value_where(generics, used_set);
    if gp.is_empty() {
        quote! {
            impl ::hicc_rs::ClassMethods for #self_type {
                type Methods = #array_ident;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_methods(<#class_ident as #trait_ident>::METHODS);
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(<#class_ident as #trait_ident>::METHODS);
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(<#class_ident as #trait_ident>::METHODS);
            }
        }
    } else {
        // Use class_ty for the trait bound (inside < >, so no ambiguity)
        let ct = class_ty(class_ident, arg_idents);
        quote! {
            impl<#(#gp),*> ::hicc_rs::ClassMethods for #self_type #wc {
                type Methods = #array_ident;
                const METHODS: &'static ::hicc_rs::AbiMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_methods(<#ct as #trait_ident>::METHODS);
                const REF_METHODS: &'static ::hicc_rs::AbiRefMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_methods(<#ct as #trait_ident>::METHODS);
                const REF_MUT_METHODS: &'static ::hicc_rs::AbiRefMutMethods<Self::Methods> = &::hicc_rs::AbiClass::<Self>::new_ref_mut_methods(<#ct as #trait_ident>::METHODS);
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

// ---- Depth classification ----

fn method_depth(sig: &Signature, gs: &HashSet<Ident>) -> DepthGroup {
    let mut max_d = 0u32;
    for input in &sig.inputs {
        let ty: Type = match input {
            FnArg::Receiver(r) => {
                let m = r.mutability;
                syn::parse2(quote! { &#m Self }).unwrap()
            }
            FnArg::Typed(pt) => *pt.ty.clone(),
        };
        max_d = max_d.max(ref_ptr_depth(&ty, gs, 0));
    }
    if let ReturnType::Type(_, ty) = &sig.output {
        max_d = max_d.max(ref_ptr_depth(ty, gs, 0));
    }
    max_d
}

fn ref_ptr_depth(ty: &Type, gs: &HashSet<Ident>, depth: u32) -> u32 {
    match ty {
        Type::Reference(r) => {
            if is_generic(&r.elem, gs) {
                return depth + 1;
            }
            ref_ptr_depth(&r.elem, gs, depth + 1)
        }
        Type::Ptr(p) => {
            if is_generic(&p.elem, gs) {
                return depth + 1;
            }
            ref_ptr_depth(&p.elem, gs, depth + 1)
        }
        Type::Path(tp) if tp.qself.is_none() => {
            let mut max_d = 0u32;
            for seg in &tp.path.segments {
                if let PathArguments::AngleBracketed(ref a) = seg.arguments {
                    for arg in &a.args {
                        if let GenericArgument::Type(t) = arg {
                            max_d = max_d.max(ref_ptr_depth(t, gs, depth));
                        }
                    }
                }
            }
            max_d
        }
        Type::Tuple(tup) => {
            let mut m = 0u32;
            for t in &tup.elems {
                m = m.max(ref_ptr_depth(t, gs, depth));
            }
            m
        }
        _ => 0,
    }
}

fn is_generic(ty: &Type, gs: &HashSet<Ident>) -> bool {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => {
            tp.path.segments.len() == 1
                && tp.path.leading_colon.is_none()
                && gs.contains(&tp.path.segments[0].ident)
        }
        _ => false,
    }
}

struct ReplaceSelf {
    replacement: Ident,
}
impl Fold for ReplaceSelf {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match &expr {
            Expr::Path(p) if p.attrs.is_empty() && p.qself.is_none() && p.path.is_ident("self") => {
                Expr::Path(syn::ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: syn::Path::from(self.replacement.clone()),
                })
            }
            _ => syn::fold::fold_expr(self, expr),
        }
    }
}

// ---- export_class on mod ----

fn generate_mod(item_mod: ItemMod, in_hicc: bool) -> Result<TS2, syn::Error> {
    let mod_ident = item_mod.ident;
    let mod_attrs: Vec<&syn::Attribute> = item_mod
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("export_class"))
        .collect();
    let content = item_mod.content.as_ref().ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "export_class on mod requires a module with body",
        )
    })?;

    let mut generated = Vec::new();
    for item in &content.1 {
        if let syn::Item::Impl(imp) = item {
            generated.push(generate(imp.clone(), in_hicc)?);
        }
    }
    if generated.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "no impl blocks found in mod",
        ));
    }

    let output = quote! {
        #(#mod_attrs)*
        mod #mod_ident {
            #(#generated)*
        }
    };
    let out_str = output.to_string();
    if in_hicc {
        let replaced = out_str.replace("::hicc_rs::", "crate::");
        match replaced.parse::<TS2>() {
            Ok(t) => {
                // Check if the parsed output still has crate:: that should work
                Ok(t)
            }
            Err(e) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                &format!("reparse after in_hicc: {}", e),
            )),
        }
    } else {
        match out_str.parse::<TS2>() {
            Ok(t) => Ok(t),
            Err(e) => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                &format!("reparse: {}", e),
            )),
        }
    }
}

// =====================================================================
// export_lib — wraps standalone functions as C-compatible library
// =====================================================================

#[proc_macro_attribute]
pub fn export_lib(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: TS2 = item.into();
    let input = replace_semicolons(input);
    let attr_str = attr.to_string();
    let in_hicc = attr_str.contains("in_hicc");
    let export_name: String = if attr_str.contains("export_name") {
        let parts: Vec<&str> = attr_str.split('=').collect();
        if parts.len() >= 2 {
            let val = parts[1].trim().trim_matches('"').trim_matches(')').trim();
            val.to_string()
        } else {
            "exported_lib".to_string()
        }
    } else {
        "exported_lib".to_string()
    };
    match export_lib_inner(input, &export_name, in_hicc) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error().into(),
    }
}

fn export_lib_inner(
    input: TS2,
    export_name_from_attr: &str,
    _in_hicc: bool,
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
    // Filter out the export_lib attribute to avoid recursive expansion
    let mod_attrs: Vec<&syn::Attribute> = item_mod
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("export_lib"))
        .collect();
    let lib_struct = format_ident!("_Hicc_Rs_{}", export_name);
    let create_fn = format_ident!("{}", export_name);

    // Separate function items from other items (export_class, etc.)
    let mut lib_fns: Vec<&syn::ItemFn> = Vec::new();
    let mut pass_through: Vec<TS2> = Vec::new();

    for item in &content.1 {
        match item {
            syn::Item::Fn(f) => {
                lib_fns.push(f);
            }
            other => {
                pass_through.push(quote! { #other });
            }
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
        let adapter_name = format_ident!("_hicc_rs_{}_{}", mod_ident, fn_name);
        let has_body = !f.block.stmts.is_empty();
        let params: Vec<_> = f
            .sig
            .inputs
            .iter()
            .filter_map(|i| match i {
                FnArg::Typed(pt) => Some(pt.ty.clone()),
                _ => None,
            })
            .collect();

        let mut abi_params = Vec::new();
        let mut param_convs = Vec::new();
        for (idx, ty) in params.iter().enumerate() {
            let an = format_ident!("a{}", idx + 1);
            abi_params.push(quote! { #an: <#ty as ::hicc_rs::AbiType>::InputType });
            param_convs.push(quote! { let #an = <#ty as ::hicc_rs::AbiType>::from_abi(#an); });
        }
        let call_args: Vec<_> = (0..params.len())
            .map(|i| {
                let an = format_ident!("a{}", i + 1);
                quote! { #an }
            })
            .collect();

        let fn_ty = {
            let mut its = Vec::new();
            for ty in &params {
                its.push(quote! { <#ty as ::hicc_rs::AbiType>::InputType });
            }
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
            quote! { unsafe extern "C" fn(#(#its),*) #rt }
        };
        struct_fields.push(quote! { pub #fn_name: #fn_ty });

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

        let body = if has_body {
            if let Some(ref ret_ty) = oty {
                quote! { #(#param_convs)* <#ret_ty as ::hicc_rs::AbiType>::into_abi({ #f.block }) }
            } else {
                quote! { #(#param_convs)* #f.block }
            }
        } else if let Some(ref ret_ty) = oty {
            quote! { #(#param_convs)* <#ret_ty as ::hicc_rs::AbiType>::into_abi(crate::#fn_name(#(#call_args),*)) }
        } else {
            quote! { #(#param_convs)* crate::#fn_name(#(#call_args),*); }
        };

        adapter_fns
            .push(quote! { unsafe extern "C" fn #adapter_name(#(#abi_params),*) #rty { #body } });
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
    match result.to_string().parse::<TokenStream>() {
        Ok(t) => Ok(t),
        Err(e) => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            &format!("reparse: {}", e),
        )),
    }
}
