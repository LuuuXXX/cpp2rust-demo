use syn::parse;
use syn::spanned::Spanned;

pub trait Visitor {
    fn visit_fn(&mut self, _f: &mut syn::TypeBareFn) -> parse::Result<()> {
        Ok(())
    }
    fn visit_path(&mut self, _ty: &mut syn::Path) -> parse::Result<()> {
        Ok(())
    }
    fn visit_lif(&mut self, _lif: &mut syn::Lifetime) -> parse::Result<()> {
        Ok(())
    }
    fn visit_lif_opt(&mut self, _lif: &mut Option<syn::Lifetime>) -> parse::Result<()> {
        Ok(())
    }
}

pub trait Acceptor {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()>;
}

impl Acceptor for syn::Type {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        match self {
            syn::Type::Array(ty) => ty.elem.accept(visitor),
            //syn::Type::BareFn(ty) => ty.accept(&mut PathVisitor(visitor)),
            syn::Type::BareFn(ty) => ty.accept(visitor),
            syn::Type::Path(ty) => ty.accept(visitor),
            syn::Type::Ptr(ty) => ty.elem.accept(visitor),
            syn::Type::Reference(ty) => ty.accept(visitor),
            syn::Type::Never(_) => Ok(()),
            syn::Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(()),
            //syn::Type::Macro(_),
            //syn::Type::Paren(_),
            //syn::Type::Infer(_),
            //syn::Type::Slice(_),
            //syn::Type::ImplTrait(_),
            //syn::Type::Tuple(_),
            //syn::Type::Verbatim(_),
            _ => Err(syn::Error::new(self.span(), "cabi can't support this type")),
        }
    }
}

impl Acceptor for syn::TypeReference {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        visitor.visit_lif_opt(&mut self.lifetime)?;
        self.elem.accept(visitor)
    }
}

impl Acceptor for syn::TypeBareFn {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        visitor.visit_fn(self)?;
        for input in self.inputs.iter_mut() {
            input.ty.accept(visitor)?;
        }
        self.output.accept(visitor)
    }
}

impl Acceptor for syn::ReturnType {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        if let syn::ReturnType::Type(_, ref mut ty) = self {
            ty.accept(visitor)?;
        }
        Ok(())
    }
}

impl Acceptor for syn::Path {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        visitor.visit_path(self)?;
        for segment in self.segments.iter_mut() {
            match segment.arguments {
                syn::PathArguments::AngleBracketed(ref mut arg) => {
                    for arg in arg.args.iter_mut() {
                        match arg {
                            syn::GenericArgument::Lifetime(ref mut lif) => {
                                visitor.visit_lif(lif)?
                            }
                            syn::GenericArgument::Type(ref mut ty) => ty.accept(visitor)?,
                            _ => {
                                return Err(syn::Error::new(
                                    arg.span(),
                                    "cabi can't support this type",
                                ))
                            }
                        }
                    }
                }
                syn::PathArguments::Parenthesized(ref mut arg) => {
                    return Err(syn::Error::new(arg.span(), "cabi can't support this type"))
                }
                _ => (),
            }
        }
        Ok(())
    }
}

impl Acceptor for syn::TypePath {
    fn accept<T: Visitor>(&mut self, visitor: &mut T) -> parse::Result<()> {
        if let Some(syn::QSelf { ref mut ty, .. }) = self.qself {
            ty.accept(visitor)?;
        }
        self.path.accept(visitor)
    }
}
