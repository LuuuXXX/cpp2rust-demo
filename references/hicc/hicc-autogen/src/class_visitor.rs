use crate::{mut_type_from_exception, Class, ClassDecl, Generics};
use quote::{format_ident, quote};

pub trait ClassVisitor {
    fn is_class(&self, ident: &syn::Ident) -> bool;
    fn hicc(&self) -> &syn::Path;
}

pub trait ClassAcceptor {
    fn class_accept<T: ClassVisitor>(&mut self, visitor: &T);
}

impl ClassAcceptor for syn::Type {
    fn class_accept<T: ClassVisitor>(&mut self, visitor: &T) {
        if let Some(ty) = mut_type_from_exception(self) {
            ty.class_accept(visitor);
            return;
        }
        let info = self.class_info();
        if let Some(true) = info
            .path
            .as_ref()
            .map(|path| visitor.is_class(&path.segments[0].ident))
        {
            if let Some(ty) = info.to_input(visitor.hicc()) {
                *self = ty;
            }
        }
    }
}

pub struct ClassIdents {
    idents: Vec<syn::Ident>,
    hicc: syn::Path,
}

impl ClassIdents {
    pub fn new() -> Self {
        Self {
            idents: vec![format_ident!("Self")],
            hicc: syn::parse2(quote! { ::hicc }).unwrap(),
        }
    }
    pub fn append_decls(&mut self, decls: &[ClassDecl]) {
        for item in decls {
            self.idents.push(item.ident.clone());
        }
    }

    pub fn append_with_generics(&mut self, ident: syn::Ident, generics: &Option<Generics>) {
        self.append_generics(generics);
        self.idents.push(ident);
    }

    pub fn append_class(&mut self, class: &[Class]) {
        for item in class {
            self.append_generics(&item.generics);
            self.idents.push(item.ident.clone());
        }
    }

    pub fn set_hicc(&mut self, hicc: syn::Path) {
        self.hicc = hicc;
    }

    fn append_generics(&mut self, generics: &Option<Generics>) {
        if let Some(generics) = generics {
            for ty in generics.types.iter() {
                self.idents.push(ty.clone());
            }
        }
    }
}

impl ClassVisitor for ClassIdents {
    fn is_class(&self, ident: &syn::Ident) -> bool {
        self.idents.contains(ident)
    }
    fn hicc(&self) -> &syn::Path {
        &self.hicc
    }
}

impl ClassAcceptor for syn::ReturnType {
    fn class_accept<T: ClassVisitor>(&mut self, visitor: &T) {
        let syn::ReturnType::Type(_, ref mut ty) = self else {
            return;
        };
        let ty = match mut_type_from_exception(ty.as_mut()) {
            Some(ty) => ty,
            _ => ty.as_mut(),
        };
        let info = ty.class_info();
        if let Some(true) = info
            .path
            .as_ref()
            .map(|path| visitor.is_class(&path.segments[0].ident))
        {
            if let Some(class_ty) = info.to_output(visitor.hicc()) {
                *ty = class_ty;
            }
        }
    }
}

trait ClassInfoGetter {
    fn class_info(&self) -> ClassInfo;
}

#[derive(Debug, Clone)]
enum ClassType {
    None,
    Value,
    Reference(syn::Lifetime, bool),
    Ptr(usize, bool),
}

#[derive(Debug, Clone)]
struct ClassInfo {
    pub ty: ClassType,
    pub path: Option<syn::Path>,
}

impl ClassInfo {
    fn new() -> Self {
        Self {
            ty: ClassType::None,
            path: None,
        }
    }
    fn to_output(&self, hicc: &syn::Path) -> Option<syn::Type> {
        let path = self.path.as_ref()?;
        let tokens = match self.ty {
            ClassType::Value => quote! { <#path as #hicc::AbiType>::OutputType },
            ClassType::Reference(ref lif, true) => {
                quote! { <#path as #hicc::AbiType>::OutputRef<#lif> }
            }
            ClassType::Reference(ref lif, false) => {
                quote! { <#path as #hicc::AbiType>::OutputRefMut<#lif> }
            }
            ClassType::Ptr(n, true) => {
                let ptr = self.origin_ptr(n, true, hicc);
                quote! { <#path as #hicc::AbiType>::OutputPtr<'static, #ptr, #n> }
            }
            ClassType::Ptr(n, false) => {
                let ptr = self.origin_ptr(n, false, hicc);
                quote! { <#path as #hicc::AbiType>::OutputMutPtr<'static, #ptr, #n> }
            }
            _ => return None,
        };
        syn::parse2::<syn::Type>(tokens).ok()
    }

    fn origin_ptr(&self, n: usize, is_const: bool, hicc: &syn::Path) -> syn::Type {
        let ty = self.path.as_ref().unwrap();
        let mut token = quote! { <#ty as #hicc::AbiType>::InputType };
        for _ in 0..n {
            token = if is_const {
                quote! { *const #token }
            } else {
                quote! { *mut #token }
            };
        }
        syn::parse2::<syn::Type>(token).unwrap()
    }

    fn to_input(&self, hicc: &syn::Path) -> Option<syn::Type> {
        let path = self.path.as_ref()?;
        let tokens = match self.ty {
            ClassType::Value => quote! { <#path as #hicc::AbiType>::InputType },
            ClassType::Reference(ref lif, true) => {
                quote! { &#lif <#path as #hicc::AbiType>::InputType }
            }
            ClassType::Reference(ref lif, false) => {
                quote! { &#lif mut <#path as #hicc::AbiType>::InputType }
            }
            ClassType::Ptr(n, true) => {
                let ptr = self.origin_ptr(n, true, hicc);
                quote! { <#path as #hicc::AbiType>::InputPtr<'_, #ptr, #n> }
            }
            ClassType::Ptr(n, false) => {
                let ptr = self.origin_ptr(n, false, hicc);
                quote! { <#path as #hicc::AbiType>::InputMutPtr<'_, #ptr, #n> }
            }
            _ => return None,
        };
        syn::parse2::<syn::Type>(tokens).ok()
    }
}

trait TypeVisitor {
    fn visit_path(&mut self, path: &syn::Path) -> Result<(), ()>;
    fn visit_ref(&mut self, lif: Option<&syn::Lifetime>, is_const: bool) -> Result<(), ()>;
    fn visit_ptr(&mut self, is_const: bool) -> Result<(), ()>;
}

trait TypeAcceptor {
    fn ty_accept<T: TypeVisitor>(&self, visitor: &mut T) -> Result<(), ()>;
}

impl ClassInfoGetter for syn::Type {
    fn class_info(&self) -> ClassInfo {
        let mut info = ClassInfo::new();
        let _ = self.ty_accept(&mut info);
        info
    }
}

impl TypeVisitor for ClassInfo {
    fn visit_path(&mut self, path: &syn::Path) -> Result<(), ()> {
        if matches!(self.ty, ClassType::None) {
            self.ty = ClassType::Value;
        }
        self.path = Some(path.clone());
        Ok(())
    }

    fn visit_ref(&mut self, lif: Option<&syn::Lifetime>, is_const: bool) -> Result<(), ()> {
        if !matches!(self.ty, ClassType::None) {
            return Err(());
        }
        let lif = if let Some(lif) = lif {
            lif.clone()
        } else {
            syn::parse2::<syn::Lifetime>(quote! {'_}).unwrap()
        };
        self.ty = ClassType::Reference(lif, is_const);
        Ok(())
    }
    fn visit_ptr(&mut self, is_const: bool) -> Result<(), ()> {
        match self.ty {
            ClassType::None => self.ty = ClassType::Ptr(1, is_const),
            ClassType::Ptr(n, _) => self.ty = ClassType::Ptr(n + 1, is_const),
            _ => return Err(()),
        }
        Ok(())
    }
}

impl TypeAcceptor for syn::Type {
    fn ty_accept<T: TypeVisitor>(&self, visitor: &mut T) -> Result<(), ()> {
        match self {
            syn::Type::Path(ref path) => path.ty_accept(visitor),
            syn::Type::Reference(ref refer) => refer.ty_accept(visitor),
            syn::Type::Ptr(ref ptr) => ptr.ty_accept(visitor),
            _ => Err(()),
        }
    }
}

impl TypeAcceptor for syn::TypePath {
    fn ty_accept<T: TypeVisitor>(&self, visitor: &mut T) -> Result<(), ()> {
        if self.path.leading_colon.is_some() || self.path.segments.len() != 1 {
            return Ok(());
        }
        visitor.visit_path(&self.path)
    }
}

impl TypeAcceptor for syn::TypeReference {
    fn ty_accept<T: TypeVisitor>(&self, visitor: &mut T) -> Result<(), ()> {
        visitor.visit_ref(self.lifetime.as_ref(), self.mutability.is_none())?;
        self.elem.ty_accept(visitor)
    }
}

impl TypeAcceptor for syn::TypePtr {
    fn ty_accept<T: TypeVisitor>(&self, visitor: &mut T) -> Result<(), ()> {
        visitor.visit_ptr(self.const_token.is_some())?;
        self.elem.ty_accept(visitor)
    }
}
