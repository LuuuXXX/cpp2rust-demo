use super::*;
use crate::{Attr, ImportClass, ImportFn};
use proc_macro2::Span;

pub(super) struct ClassType {
    input: String,
    line: usize,
}

impl ClassType {
    pub fn new(input: &str, line: usize) -> Self {
        Self {
            input: input.to_string(),
            line,
        }
    }
}

#[allow(dead_code)]
impl ClassType {
    fn split(&self) -> (Option<&str>, &str) {
        let (x1, x2) = first_ident(&self.input);
        if &self.input[x1..x2] != "template" {
            return (None, self.input.trim_end());
        }
        let (Some(_), Some(y)) = first_group(&self.input, (b'<', b'>')) else {
            return (None, self.input.trim_end());
        };
        (Some(&self.input[..y + 1]), self.input[y + 1..].trim())
    }

    fn template(&self) -> Option<&str> {
        self.split().0
    }

    fn template_args(&self) -> Option<&str> {
        if let Some(template) = self.template() {
            if let (Some(beg), Some(end)) = first_group(template, (b'<', b'>')) {
                return Some(&template[beg..end + 1]);
            }
        }
        None
    }

    fn args(&self) -> Option<String> {
        let args = self.template_args()?;
        let mut s = String::with_capacity(args.len());
        let args = args[1..args.len() - 1].split(',').map(|ty| {
            let ty = ty.trim_end();
            if let Some(n) = ty.rfind(|c: char| c.is_ascii_whitespace()) {
                return &ty[n + 1..];
            }
            ty
        });

        s.push('<');
        for arg in args {
            if s.len() > 1 {
                s.push(',');
            }
            s.push_str(arg);
        }
        s.push('>');
        Some(s)
    }

    pub fn typename(&self) -> String {
        if !self.need_typename() {
            self.split().1.to_string()
        } else {
            format!("typename {}", self.split().1)
        }
    }

    fn name(&self) -> &str {
        self.split().1
    }

    // 判断是否需要`typename`
    fn need_typename(&self) -> bool {
        // not template class
        let (Some(_), name) = self.split() else {
            return false;
        };

        let (Some(x1), Some(x2)) = last_group(name, (b'<', b'>')) else {
            return false;
        };
        // inner class
        if x2 < name.len() - 1 {
            return true;
        }
        if let (Some(_), Some(_)) = last_group(&name[..x1], (b'<', b'>')) {
            return true;
        }
        false
    }

    fn container(&self) -> Option<&str> {
        // not template class
        let (Some(_), name) = self.split() else {
            return None;
        };

        let (Some(x1), Some(x2)) = last_group(name, (b'<', b'>')) else {
            return None;
        };

        if x2 < name.len() - 1 {
            // xxxx<...>::yyyy
            if let Some(n) = name.rfind(':') {
                return Some(name[..n - 1].trim_end());
            }
            return None;
        }

        if let (Some(_), Some(_)) = last_group(&name[..x1], (b'<', b'>')) {
            // xxx<...>::yyy<...>
            if let Some(n) = name[..x1].rfind(':') {
                return Some(name[..n - 1].trim_end());
            }
        }
        None
    }
}

pub(super) struct Class<'a> {
    imported: &'a crate::Class,
    methods_class: String,
    ty: ClassType,
    with_class: Option<String>,
    destroy: Option<String>,
    ctor: Option<String>,
}

impl<'a> Class<'a> {
    pub fn try_from(c: &'a crate::Class) -> syn::Result<Self> {
        let Some(attr) = Attr::get_attr("cpp", &c.attrs) else {
            return Err(syn::Error::new(c.ident.span(), "not found #[cpp(...)]"));
        };
        let line = attr.span().start().line;
        let Ok(Some(class)) = attr.value("class") else {
            return Err(syn::Error::new(
                c.ident.span(),
                "not found #[cpp(class = ...)]",
            ));
        };
        let ctor = attr.value("ctor").unwrap_or(None).map(|v| v.to_string());
        let with_class = attr.value("with").unwrap_or(None).map(|v| v.to_string());
        let destroy = attr.value("destroy").unwrap_or(None).map(|v| v.to_string());
        let methods_class = format!("{}_{line}", c.ident);
        Ok(Self {
            imported: c,
            methods_class,
            destroy,
            ctor,
            ty: ClassType::new(class, line),
            with_class,
        })
    }

    fn full_methods_class(&self) -> String {
        if let Some(args) = self.ty.args() {
            format!("{}{}", self.methods_class, args)
        } else {
            self.methods_class.clone()
        }
    }

    fn export_deleter(&self, codes: &mut String) -> syn::Result<()> {
        let Some(ref destroy) = self.destroy else {
            return Ok(());
        };
        let template = self.ty.template().unwrap_or("template<>");
        let tyname = self.ty.typename();
        codes.push_str(&format!("#line {}\n", self.ty.line));
        codes.push_str(&format!(
            "namespace hicc {{ {template} struct Deleter<{}> {{ static void destroy({}* obj) {{ ({})(obj); }} }}; }}\n",
            tyname, tyname, destroy
        ));
        Ok(())
    }

    fn export_methods_class_forward(&self, codes: &mut String) -> syn::Result<()> {
        let template = self.ty.template().unwrap_or("");
        codes.push_str(&format!("#line {}\n", self.ty.line));
        codes.push_str(&format!("{template} struct {};\n", self.methods_class));
        Ok(())
    }

    fn container(&self) -> String {
        match (self.ty.container(), &self.with_class) {
            (Some(c1), None) => ClassType::new(c1, 0).typename(),
            (None, Some(c2)) => ClassType::new(c2, 0).typename(),
            (Some(c1), Some(c2)) => format!(
                "std::tuple<{}, {}>",
                ClassType::new(c1, 0).typename(),
                ClassType::new(c2, 0).typename()
            ),
            _ => "void".to_string(),
        }
    }

    pub fn export_methods_class(&self) -> syn::Result<String> {
        let mut codes = String::new();
        self.export_deleter(&mut codes)?;
        self.export_methods_class_forward(&mut codes)?;

        let container = self.container();
        let template = self.ty.template().unwrap_or("template<>");
        let tyname = self.ty.typename();
        let methods_class = self.full_methods_class();
        codes.push_str(&format!("#line {}\n", self.ty.line));
        codes.push_str(&format!(
            "namespace hicc {{ {} struct MethodsType<{}, {}> {{ typedef {} methods_type; }}; }}\n",
            template, tyname, container, methods_class
        ));
        Ok(codes)
    }

    fn export_typedef(&self, codes: &mut String) -> syn::Result<()> {
        let container = self.container();
        let typename = self.ty.typename();
        let methods_class = self.full_methods_class();
        codes.push_str(&format!("#line {}\n", self.ty.line));
        codes.push_str(&format!(
            "typedef {} Self; typedef {} SelfContainer; typedef {} SelfMethods;\n",
            typename, container, methods_class,
        ));
        Ok(())
    }

    fn export_methods(&self, intfs: &ImportClass, codes: &mut String) -> syn::Result<()> {
        let intf_methods = intfs.get_intf_methods(&self.imported.intf);
        for f in intf_methods.into_iter().chain(self.imported.methods.iter()) {
            let method = Function::try_from(f)?;
            codes.push_str(&method.export_test()?);
            codes.push_str(&method.export_with(Some("SelfContainer"))?);
        }
        Ok(())
    }

    pub fn export(&self, intfs: &ImportClass) -> syn::Result<String> {
        let mut codes = String::new();
        let template = self.ty.template().unwrap_or("");
        codes.push_str(&format!("#line {}\n", self.ty.line));
        codes.push_str(&format!("{template} struct {} {{\n", self.methods_class));

        self.export_typedef(&mut codes)?;
        for cpp in self.imported.cpps.iter() {
            codes.push_str(&cpp.export());
        }
        self.export_methods(intfs, &mut codes)?;
        codes.push_str(&format!("#line {}\n}};\n", self.ty.line));
        Ok(codes)
    }
}

pub(super) struct RemoteProxy<'a> {
    class: &'a Class<'a>,
    imported: &'a ImportClass,
}

impl<'a> RemoteProxy<'a> {
    pub fn new(class: &'a Class, imported: &'a ImportClass) -> Self {
        Self { class, imported }
    }

    pub fn export(&self) -> syn::Result<String> {
        let mut codes = String::new();
        let Some(ref ctor) = self.class.ctor else {
            return Ok(codes);
        };
        let intf_methods = self.imported.get_intf_methods(&self.class.imported.intf);
        if intf_methods.is_empty() {
            return Ok(codes);
        }

        let line = self.class.ty.line;
        let proxy_class = format!("{}_proxy_{line}", self.class.imported.ident);
        let template = self.class.ty.template().unwrap_or("");
        let typename = self.class.ty.typename();
        codes.push_str(&format!("#line {line}\n"));
        codes.push_str(&format!("{template} struct {proxy_class}: public ::hicc::RemoteProxy<{typename}> {{ typedef {typename} Self;\n"));

        codes.push_str(&self.export_ctor(ctor, &proxy_class)?);
        self.export_intf(&intf_methods, &mut codes)?;

        codes.push_str(&format!("#line {line}\n}};\n"));

        self.export_proxy_type(&typename, &proxy_class, &mut codes)?;
        Ok(codes)
    }

    fn span(&self) -> Span {
        self.class.imported.ident.span()
    }
    fn export_ctor(&self, ctor: &str, proxy_class: &str) -> syn::Result<String> {
        let mut codes = String::new();
        let (Some(x1), Some(x2)) = first_group(ctor, (b'(', b')')) else {
            return Err(syn::Error::new(self.span(), "ctor attr: wrong format"));
        };
        let args = Args::new(&ctor[x1 + 1..x2]);
        codes.push_str(&format!("#line {}\n", self.class.ty.line));
        codes.push_str(&format!("{proxy_class}(::hicc::AbiClass<Self> remote"));
        let mut argc = 0;
        for (n, arg) in args.enumerate() {
            codes.push_str(", ");
            let Some(arg) = Arg::new(arg).with_name(&format!("_{n}")) else {
                return Err(syn::Error::new(
                    self.span(),
                    format!("wrong ctor arg: {arg}"),
                ));
            };
            codes.push_str(&arg);
            argc += 1;
        }
        codes.push_str("): ::hicc::RemoteProxy<Self>(remote");
        for n in 0..argc {
            codes.push_str(&format!(", _{n}"));
        }
        codes.push_str(") {}\n");
        Ok(codes)
    }

    fn export_proxy_type(
        &self,
        typename: &str,
        proxy_class: &str,
        codes: &mut String,
    ) -> syn::Result<()> {
        codes.push_str(&format!("#line {}\n", self.class.ty.line));
        codes.push_str(&format!("namespace hicc {{ template<> struct ProxyType<{typename}> {{ typedef {proxy_class} type; }}; }}\n"));
        Ok(())
    }

    fn export_intf(&self, methods: &[&ImportFn], codes: &mut String) -> syn::Result<()> {
        for (n, f) in methods.iter().enumerate() {
            let f = Function::try_from(f)?;
            codes.push_str(&self.export_virtual_method(&f, n)?);
        }
        Ok(())
    }

    fn export_virtual_method(&self, f: &Function<'_>, idx: usize) -> syn::Result<String> {
        let mut codes = String::new();
        codes.push_str(&format!("#line {}\n", f.line));
        let (x1, x2, x3, x4, x5) = f.cpp_info.segments;
        codes.push_str("virtual ");
        codes.push_str(&f.input[..x3 + 1]);
        let args = Args::new(&f.input[x3 + 1..x4]);
        let mut argc = 0;
        for (n, arg) in args.enumerate() {
            let Some(arg) = Arg::new(arg).with_name(&format!("_{n}")) else {
                return Err(syn::Error::new(f.span, format!("wrong ctor arg: {arg}")));
            };
            if n > 0 {
                codes.push_str(", ");
            }
            codes.push_str(&arg);
            argc += 1;
        }
        codes.push_str(&f.input[x4..x5]);
        codes.push_str(" override ");
        codes.push_str(&f.input[x5..]);
        codes.push('{');
        codes.push_str(&format!(
            "CALL_REMOTE_METHOD_BY_IDX(({}(Self::*){})&Self::{}, {idx}",
            &f.input[..x1],
            &f.input[x3..],
            &f.input[x1..x2]
        ));
        for n in 0..argc {
            codes.push_str(&format!(", _{n}"));
        }
        codes.push_str("); }\n");
        Ok(codes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let class = ClassType::new("std::vector", 0);
        assert_eq!(class.template(), None);
        assert_eq!(class.template_args(), None);
        assert_eq!(class.args(), None);
        assert_eq!(class.container(), None);
        assert_eq!(class.split(), (None, "std::vector"));
        assert_eq!(class.need_typename(), false);

        let class = ClassType::new(
            "template<class T, class Allocator> std::vector<T, Allocator> ",
            0,
        );
        assert_eq!(
            class.split(),
            (
                Some("template<class T, class Allocator>"),
                "std::vector<T, Allocator>"
            )
        );
        assert_eq!(class.template(), Some("template<class T, class Allocator>"));
        assert_eq!(class.template_args(), Some("<class T, class Allocator>"));
        assert_eq!(class.args(), Some("<T,Allocator>".to_string()));
        assert_eq!(class.container(), None);
        assert_eq!(class.need_typename(), false);

        let class = ClassType::new(
            "template<class T, class Allocator> std::vector<T, Allocator>::iterator ",
            0,
        );
        assert_eq!(
            class.split(),
            (
                Some("template<class T, class Allocator>"),
                "std::vector<T, Allocator>::iterator"
            )
        );
        assert_eq!(class.template(), Some("template<class T, class Allocator>"));
        assert_eq!(class.template_args(), Some("<class T, class Allocator>"));
        assert_eq!(class.args(), Some("<T,Allocator>".to_string()));
        assert_eq!(class.container(), Some("std::vector<T, Allocator>"));
        assert_eq!(class.need_typename(), true);

        let class = ClassType::new(
            "template<class T, class Allocator> std::vector<T, Allocator>::iterator<T> ",
            0,
        );
        assert_eq!(
            class.split(),
            (
                Some("template<class T, class Allocator>"),
                "std::vector<T, Allocator>::iterator<T>"
            )
        );
        assert_eq!(class.template(), Some("template<class T, class Allocator>"));
        assert_eq!(class.template_args(), Some("<class T, class Allocator>"));
        assert_eq!(class.args(), Some("<T,Allocator>".to_string()));
        assert_eq!(class.container(), Some("std::vector<T, Allocator>"));
        assert_eq!(class.need_typename(), true);
    }
}
