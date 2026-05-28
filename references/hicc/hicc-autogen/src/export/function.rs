use super::*;
use crate::{type_from_exception, Attr, ImportFn};
use proc_macro2::Span;
use syn::spanned::Spanned;

pub(super) struct Arg<'a>(&'a str);

impl<'a> Arg<'a> {
    pub fn new(arg: &'a str) -> Self {
        Self(arg.trim())
    }
}

impl Arg<'_> {
    pub fn with_name(&self, name: &str) -> Option<String> {
        let pos = match parse_func_type(self.0) {
            (_, pos, None) => pos,
            (_, _, Some((pos, _))) => pos,
        };
        let input = self.0[..pos].trim_end();
        if matches!(input.as_bytes().last(), Some(b']')) {
            // 数组
            let (Some(x), _) = last_group(input, (b'[', b']')) else {
                return None;
            };
            return Some(format!("{} {name}{}", &self.0[..x], &self.0[x..]));
        }
        Some(format!("{} {name}{}", input, &self.0[input.len()..]))
    }

    pub fn check(&self, ty: &syn::Type) -> bool {
        let cpp = ArgInfo::new(self.0);
        let rust = ArgInfo::from_rust(ty);
        cpp.check_with_rust(&rust)
    }
}

#[derive(Debug)]
pub(super) struct ArgInfo {
    is_ref: bool,
    is_const: bool,
    is_volatile: bool,
    is_pointer: bool,
    is_rr: bool,
    is_func: bool,
    is_cabi: bool,
}

impl ArgInfo {
    pub fn check_with_rust(&self, rust: &Self) -> bool {
        if !rust.is_cabi {
            return false;
        }
        if self.is_ref {
            rust.is_ref && (self.is_const || !rust.is_const)
        } else if self.is_pointer {
            (rust.is_ref || rust.is_pointer) && (self.is_const || !rust.is_const)
        } else if self.is_func {
            rust.is_func
        } else {
            !rust.is_ref && !rust.is_pointer
        }
    }

    pub fn new(arg: &str) -> Self {
        let (x1, x2, _) = parse_func_type(arg);
        let mut ty = arg[x1..x2].trim();

        let is_rr = ty.ends_with("&&");
        if is_rr {
            ty = ty[..ty.len() - 2].trim();
        }

        let (is_ref, is_const, is_volatile, is_pointer, is_func) = if x1 > 0 {
            (false, false, false, false, true)
        } else {
            'ref_ptr: {
                let mut is_const = false;
                let mut is_volatile = false;
                for token in TokenStream::new(ty) {
                    match token {
                        TokenTree::Ident(Range(x1, x2)) if &ty[x1..x2] == "const" => {
                            is_const = true;
                        }
                        TokenTree::Ident(Range(x1, x2)) if &ty[x1..x2] == "volatile" => {
                            is_volatile = true;
                        }
                        TokenTree::Punct(_, b'*') => {
                            break 'ref_ptr (false, is_const, is_volatile, true, false)
                        }
                        TokenTree::Punct(_, b'&') => {
                            break 'ref_ptr (true, is_const, is_volatile, false, false)
                        }
                        _ => {}
                    }
                }
                (false, false, false, false, false)
            }
        };
        Self {
            is_ref,
            is_const,
            is_volatile,
            is_pointer,
            is_rr,
            is_func,
            is_cabi: true,
        }
    }

    pub fn from_rust(ty: &syn::Type) -> Self {
        let (is_ref, is_const, is_pointer, is_func, is_cabi) = match ty {
            syn::Type::Reference(ref ty) => (
                true,
                ty.mutability.is_none(),
                false,
                false,
                Self::is_cabi(ty.elem.as_ref()),
            ),
            syn::Type::Ptr(ref ty) => (
                false,
                ty.mutability.is_none(),
                true,
                false,
                Self::is_cabi(ty.elem.as_ref()),
            ),
            syn::Type::Path(ref path) => {
                let (is_ref, is_const, is_pointer) = Self::cabi_type(path);
                (is_ref, is_const, is_pointer, false, true)
            }
            syn::Type::BareFn(ref func) => {
                let is_cabi = if let Some(syn::Abi {
                    name: Some(ref lit),
                    ..
                }) = func.abi
                {
                    lit.value() == "C"
                } else {
                    false
                };
                (false, false, false, true, is_cabi)
            }
            _ => (false, false, false, false, false),
        };
        Self {
            is_ref,
            is_const,
            is_pointer,
            is_func,
            is_cabi,
            is_rr: false,
            is_volatile: false,
        }
    }

    pub fn is_cabi(mut ty: &syn::Type) -> bool {
        loop {
            match ty {
                syn::Type::Path(_) => return true,
                syn::Type::Reference(ref ty_ref) => ty = ty_ref.elem.as_ref(),
                syn::Type::Ptr(ref ty_ptr) => ty = ty_ptr.elem.as_ref(),
                _ => return false,
            }
        }
    }

    pub fn cabi_type(path: &syn::TypePath) -> (bool, bool, bool) {
        // 自动生成的时候一定是全路径.
        if path.qself.is_some()
            && path.path.segments.len() == 3
            && path.path.segments[0].ident == "hicc"
            && path.path.segments[1].ident == "AbiType"
        {
            return match path.path.segments[2].ident.to_string().as_str() {
                "InputRef" => (true, true, false),
                "InputRefMut" => (true, false, false),
                "InputPtr" => (false, true, true),
                "InputMutPtr" => (false, false, true),
                _ => (false, false, false),
            };
        }
        (false, false, false)
    }
}

#[derive(Clone)]
pub(super) struct Args<'a> {
    input: TokenStream<'a>,
}

#[allow(dead_code)]
impl<'a> Args<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: TokenStream::new(input),
        }
    }

    pub fn argc(&self) -> usize {
        self.clone().count()
    }
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let beg = self.input.offset;
        let mut end = None;
        for token in &mut self.input {
            if matches!(token, TokenTree::Punct(_, b',')) {
                break;
            }
            end = Some(token);
        }
        if let Some(end) = end {
            let (_, end) = end.range();
            return Some(self.input.input[beg..end].trim());
        }
        None
    }
}

#[derive(Debug)]
pub(super) struct ReturnInfo {
    is_void: bool,
    except: bool,
}

impl ReturnInfo {
    fn new(is_void: bool, except: bool) -> Self {
        Self { is_void, except }
    }

    pub fn from_rust(output: &syn::ReturnType) -> Self {
        let syn::ReturnType::Type(_, ref ty) = output else {
            return Self::new(true, false);
        };
        let (ty, except) = match type_from_exception(ty.as_ref()) {
            Some(ty) => (ty, true),
            _ => (ty.as_ref(), false),
        };
        match ty {
            syn::Type::Tuple(tuple) if tuple.elems.is_empty() => Self::new(true, except),
            _ => Self::new(false, except),
        }
    }
}

#[derive(Debug)]
pub(super) struct FuncInfo {
    pub segments: (usize, usize, usize, usize, usize),
    is_variadic: bool,
    is_const_self: bool,
    is_volatile_self: bool,
    is_ref_self: bool,
    is_rr_self: bool,
    return_info: ReturnInfo,
    argc: usize,
    is_data: bool,
    is_member: bool,
    is_dynamic_cast: bool,
    is_make_proxy: bool,
}

impl FuncInfo {
    pub fn from_rust(f: &ImportFn) -> Self {
        let is_const_self = matches!(
            f.recv,
            Some(syn::Receiver {
                mutability: None,
                ..
            })
        );
        let is_ref_self = matches!(
            f.recv,
            Some(syn::Receiver {
                reference: Some(_),
                ..
            })
        );
        let argc = f.recv.as_ref().map(|_| 1).unwrap_or(0)
            + f.variadic.as_ref().map(|_| 1).unwrap_or(0)
            + f.inputs.len();
        Self {
            is_const_self,
            is_ref_self,
            argc,
            is_rr_self: !is_ref_self && f.recv.is_some(),
            return_info: ReturnInfo::from_rust(&f.output),
            is_variadic: f.variadic.is_some(),
            segments: (0, 0, 0, 0, 0),
            is_volatile_self: false,
            is_data: false,
            is_member: false,
            is_dynamic_cast: false,
            is_make_proxy: false,
        }
    }

    pub fn check_with_rust(&self, rust: &Self) -> bool {
        if self.is_data {
            self.check_data_with_rust(rust)
        } else if self.is_make_proxy {
            self.check_make_proxy_with_rust(rust)
        } else {
            self.check_func_with_rust(rust)
        }
    }

    pub fn check_func_with_rust(&self, rust: &Self) -> bool {
        (self.is_variadic == rust.is_variadic)
            && (!self.is_variadic || self.return_info.is_void == rust.return_info.is_void)
            && (!self.is_dynamic_cast || self.argc == rust.argc)
            && ((self.argc == rust.argc) || (!self.is_variadic && self.argc > rust.argc))
            && (self.is_ref_self == rust.is_ref_self)
            && (!self.is_ref_self || self.is_const_self || !rust.is_const_self)
            && (rust.return_info.is_void || !self.return_info.is_void)
    }

    pub fn check_make_proxy_with_rust(&self, rust: &Self) -> bool {
        !rust.is_ref_self && !rust.is_rr_self && rust.argc == self.argc + 1
    }

    pub fn check_data_with_rust(&self, rust: &Self) -> bool {
        (!self.is_member || rust.is_ref_self)
            && (rust.argc == 0 || (rust.argc == 1 && rust.is_ref_self))
            && !rust.return_info.is_void
            && !rust.return_info.except
    }

    pub fn diff(&self, rust: &Self) -> bool {
        self.argc > rust.argc || self.return_info.is_void != rust.return_info.is_void
    }

    pub fn new(f: &str, is_data: bool, is_member: bool, import_class: bool) -> Option<Self> {
        if !is_data {
            Self::with_func(f, is_member, import_class)
        } else {
            Some(Self::with_data(is_member))
        }
    }

    pub fn with_data(is_member: bool) -> Self {
        Self {
            is_data: true,
            is_member,
            is_ref_self: false,
            is_const_self: false,
            is_rr_self: false,
            is_volatile_self: false,
            is_variadic: false,
            return_info: ReturnInfo {
                is_void: false,
                except: false,
            },
            segments: (0, 0, 0, 0, 0),
            argc: 0,
            is_dynamic_cast: false,
            is_make_proxy: false,
        }
    }

    pub fn with_func(f: &str, is_member: bool, import_class: bool) -> Option<Self> {
        let (x1, x2, x3, x4, x5) = parse_function(f)?;
        let return_info = ReturnInfo {
            is_void: f[..x1].trim() == "void",
            except: true,
        };
        let args = f[x3 + 1..x4].trim();
        let is_variadic = args.ends_with("...") || args.ends_with("va_list");

        let mut argc = Args::new(args).argc();
        let (is_ref_self, is_const_self, is_volatile_self, is_rr_self) = if is_member {
            argc += 1;
            Self::self_info_from_method(&f[x4 + 1..x5])
        } else if import_class {
            Self::self_info_from_args(args)
        } else {
            (false, false, false, false)
        };

        Some(Self {
            is_dynamic_cast: f[x1..x2].starts_with("@dynamic_cast<"),
            is_make_proxy: f[x1..x2].starts_with("@make_proxy<"),
            is_variadic,
            is_ref_self,
            is_const_self,
            is_volatile_self,
            is_rr_self,
            return_info,
            argc,
            segments: (x1, x2, x3, x4, x5),
            is_data: false,
            is_member,
        })
    }

    pub fn self_info_from_method(modification: &str) -> (bool, bool, bool, bool) {
        let modification = modification.trim();
        let is_rr = modification.ends_with("&&");
        (
            !is_rr,
            modification.contains("const"),
            modification.contains("volatile"),
            is_rr,
        )
    }

    pub fn self_info_from_args(args: &str) -> (bool, bool, bool, bool) {
        if let Some(self_arg) = Args::new(args).next() {
            let self_info = ArgInfo::new(self_arg.trim());
            return (
                self_info.is_ref || self_info.is_pointer,
                self_info.is_const,
                self_info.is_volatile,
                self_info.is_rr,
            );
        }
        (false, false, false, false)
    }
}

pub(super) struct Function<'a> {
    pub imported: &'a ImportFn,
    pub input: String,
    pub line: usize,
    pub with_class: Option<String>,
    pub cpp_info: FuncInfo,
    pub rust_info: FuncInfo,
    pub span: Span,
}

#[allow(dead_code)]
impl<'a> Function<'a> {
    pub fn try_from(f: &'a ImportFn) -> syn::Result<Self> {
        let Some(attr) = Attr::get_attr("cpp", &f.attrs) else {
            return Err(syn::Error::new(f.ident.span(), "not found #[cpp(...)]"));
        };
        let with_class = attr.value("with").unwrap_or(None).map(|v| v.to_string());
        let (data, member, input) = match (
            attr.value("func"),
            attr.value("method"),
            attr.value("data"),
            attr.value("field"),
        ) {
            (Ok(Some(input)), Err(_), Err(_), Err(_)) => (false, false, input),
            (Err(_), Ok(Some(input)), Err(_), Err(_)) => (false, true, input),
            (Err(_), Err(_), Ok(Some(input)), Err(_)) => (true, false, input),
            (Err(_), Err(_), Err(_), Ok(Some(input))) => (true, true, input),
            (Err(_), Err(_), Err(_), Err(_)) => {
                return Err(syn::Error::new(
                    attr.span(),
                    "not found #[cpp(func | method | data | field = ...)]",
                ));
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span(),
                    "found more #[cpp(func | method | data | field = ...)]",
                ));
            }
        };
        let line = attr.span().start().line;
        let input = input.trim();
        let Some(cpp_info) = FuncInfo::new(input, data, member, f.recv.is_some()) else {
            return Err(syn::Error::new(
                attr.span(),
                format!("wrong function format: `{input}`"),
            ));
        };
        let rust_info = FuncInfo::from_rust(f);
        Self {
            imported: f,
            input: input.to_string(),
            with_class,
            line,
            cpp_info,
            rust_info,
            span: attr.span(),
        }
        .check_cpp(attr.span())
    }

    pub fn check_cpp(self, span: Span) -> syn::Result<Self> {
        if !self.cpp_info.check_with_rust(&self.rust_info) {
            return Err(syn::Error::new(span, "function type is defferent between rust and cpp"));
        };
        if self.cpp_info.is_data {
            return Ok(self);
        }
        let (_, _, x1, x2, _) = self.cpp_info.segments;
        let mut args = Args::new(&self.input[x1 + 1..x2]);
        if !self.cpp_info.is_member && self.imported.recv.is_some() {
            let _ = args.next();
        }
        let mut off = self.imported.recv.as_ref().map(|_| 1).unwrap_or(0);
        let mut it = self.imported.inputs.iter();
        if self.cpp_info.is_make_proxy {
            self.check_intf_arg(it.next())?;
            off += 1;
        }
        for (n, (cpp_arg, rust_arg)) in args.zip(it).enumerate() {
            let arg = Arg::new(cpp_arg);
            if !arg.check(&rust_arg.ty) {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "`{}`th argument type is different between rust and cpp, cpp type is `{cpp_arg}`",
                        n + off
                    ),
                ));
            }
        }
        Ok(self)
    }

    pub fn check_intf_arg(&self, arg: Option<&crate::FnArg>) -> syn::Result<()> {
        let err = Err(syn::Error::new(
            self.imported.ident.span(),
            "first argument should be `::hicc::Interface<...>`",
        ));
        let Some(arg) = arg else {
            return err;
        };
        let syn::Type::Path(ref path) = arg.ty else {
            return err;
        };
        let segments = &path.path.segments;
        match segments.len() {
            1 if segments[0].ident == "Interface" => {}
            2 if segments[0].ident == "hicc" && segments[1].ident == "Interface" => {}
            _ => return err,
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl<'a> Function<'a> {
    pub fn export_test(&self) -> syn::Result<String> {
        if self.cpp_info.is_data || self.cpp_info.is_dynamic_cast || self.cpp_info.is_make_proxy {
            return Ok(String::new());
        }
        let (x1, x2, _, _, _) = self.cpp_info.segments;
        let scope = self.scope();
        let line = self.line;

        Ok(format!(
            "#line {line}\nstatic void _hicc_test_{line}() {{ {}({scope}* _{line}){} = &{scope}{}; (void)_{line}; }}\n",
            &self.input[..x1], &self.input[x2..], &self.input[x1..x2]
        ))
    }

    pub fn export_default(&self) -> syn::Result<(String, String)> {
        let (x1, x2, x3, x4, x5) = self.cpp_info.segments;
        let name = format!("_{:?}_{}", md5::compute(self.input.as_bytes()), self.line);
        let mut codes = format!("#line {}\nstatic ", self.line);

        if !self.rust_info.return_info.is_void {
            codes.push_str(&self.input[..x1]);
        } else {
            codes.push_str("void ");
        }

        codes.push_str(&name);
        codes.push('(');

        if self.cpp_info.is_member {
            let recv = match &self.cpp_info {
                FuncInfo {
                    is_rr_self: true, ..
                } => "Self&& self",
                FuncInfo {
                    is_const_self: true,
                    is_volatile_self: true,
                    ..
                } => "const volatile Self& self",
                FuncInfo {
                    is_const_self: true,
                    ..
                } => "const Self& self",
                FuncInfo {
                    is_volatile_self: true,
                    ..
                } => "volatile Self& self",
                _ => "Self& self",
            };
            codes.push_str(recv);
        }

        let args = Args::new(&self.input[x3 + 1..x4]);
        let argc = self.imported.inputs.len();
        for (n, arg) in args.enumerate() {
            if n >= argc {
                break;
            }
            let Some(arg) = Arg::new(arg).with_name(&format!("_{n}")) else {
                return Err(syn::Error::new(
                    self.imported.inputs[n].span(),
                    self.error_argument(arg),
                ));
            };
            if n > 0 || self.cpp_info.is_member {
                codes.push_str(", ");
            }
            codes.push_str(&arg);
        }

        if !self.rust_info.return_info.is_void {
            codes.push(')');
            codes.push_str(&self.input[x5..]);
            codes.push_str("{ return ");
        } else {
            codes.push_str("){ ");
        }

        if self.cpp_info.is_member {
            codes.push_str("self.");
        }
        codes.push_str(&self.input[x1..x2]);
        codes.push('(');
        for n in 0..argc {
            if n > 0 {
                codes.push_str(", ");
            }
            codes.push_str(&format!("_{n}"));
        }
        codes.push_str("); }\n");
        if self.imported.recv.is_some() {
            Ok((format!("SelfMethods::{name}"), codes))
        } else {
            Ok((name, codes))
        }
    }

    pub fn export(&self) -> syn::Result<String> {
        self.export_with(None)
    }

    fn container(&self, with_class: Option<&str>) -> String {
        match (with_class, &self.with_class) {
            (Some(c1), None) => ClassType::new(c1, 0).typename(),
            (None, Some(c2)) => ClassType::new(c2, 0).typename(),
            (Some(c1), Some(c2)) => format!(
                "std::tuple<{}, {}>",
                ClassType::new(c1, 0).typename(),
                ClassType::new(c2, 0).typename(),
            ),
            _ => "void".to_string(),
        }
    }

    pub fn export_with(&self, with_class: Option<&str>) -> syn::Result<String> {
        let container = self.container(with_class);
        if self.cpp_info.is_data {
            let mut codes = String::new();
            let data = format!("(&{}{})", self.scope(), self.input);
            self.export_codes("DATA", &container, &data, &mut codes)?;
            Ok(codes)
        } else if self.cpp_info.is_dynamic_cast {
            self.export_dynamic_cast()
        } else if self.cpp_info.is_make_proxy {
            self.export_make_proxy()
        } else if self.cpp_info.diff(&self.rust_info) {
            let (func, mut codes) = self.export_default()?;
            self.export_codes("METHOD", &container, &func, &mut codes)?;
            Ok(codes)
        } else {
            let (x1, x2, _, _, _) = self.cpp_info.segments;
            let mut codes = String::new();
            let (scope, mac) = if self.cpp_info.is_member {
                ("Self::", "MEMBER_METHOD")
            } else {
                ("", "METHOD")
            };
            let func = format!(
                "(({}({}*){})&{}{})",
                &self.input[..x1],
                self.scope(),
                &self.input[x2..],
                scope,
                &self.input[x1..x2]
            );
            self.export_codes(mac, &container, &func, &mut codes)?;
            Ok(codes)
        }
    }

    fn export_container_typedef(
        &self,
        with_class: &str,
        codes: &mut String,
    ) -> syn::Result<String> {
        if with_class.find(',').is_none() {
            return Ok(with_class.to_string());
        }
        let name = format!("_{:?}_{}", md5::compute(with_class.as_bytes()), self.line);
        codes.push_str(&format!("#line {}\n", self.line));
        codes.push_str(&format!("typedef {with_class} {name};\n"));
        Ok(name)
    }

    pub fn export_codes(
        &self,
        mac: &str,
        with_class: &str,
        func: &str,
        codes: &mut String,
    ) -> syn::Result<()> {
        let with_class = self.export_container_typedef(with_class, codes)?;
        let lib_with_class = if self.imported.recv.is_some() {
            format!("{with_class}, Self")
        } else {
            format!("{with_class}, ExportMethods")
        };
        let with_class = if matches!(mac, "METHOD" | "DATA") {
            &lib_with_class
        } else {
            &with_class
        };

        let mac = if !self.rust_info.return_info.except {
            format!("EXPORT_{mac}_IN")
        } else {
            format!("EXPORT_EXCEPT_{mac}_IN")
        };
        let line = self.line;

        codes.push_str(&format!("#line {line}\n{mac}({with_class}, {func});\n"));
        Ok(())
    }

    pub fn export_dynamic_cast(&self) -> syn::Result<String> {
        let dynamic = if self.cpp_info.is_ref_self {
            "EXPORT_DYNAMIC"
        } else {
            "EXPORT_DYNAMIC_MOVE"
        };
        let (x1, _, x3, x4, x5) = self.cpp_info.segments;
        if x5 != self.input.len() {
            return Err(syn::Error::new(self.span, self.error_dynamic()));
        }
        let Some(return_type) = Self::get_cpp_type(&self.input[..x1]) else {
            return Err(syn::Error::new(self.span, self.error_dynamic()));
        };
        let Some(input_type) = Self::get_cpp_type(&self.input[x3 + 1..x4]) else {
            return Err(syn::Error::new(self.span, self.error_dynamic()));
        };
        Ok(format!(
            "#line {}\n{dynamic}({input_type}, {return_type});\n",
            self.line
        ))
    }

    pub fn export_make_proxy(&self) -> syn::Result<String> {
        let mut codes = String::new();
        let (x1, _, x3, x4, x5) = self.cpp_info.segments;
        if x4 + 1 < x5 {
            return Err(syn::Error::new(self.span, self.error_make_proxy()));
        }
        let ty = self.input[..x1].trim();
        codes.push_str(&format!("#line {}\n", self.line));
        codes.push_str(&format!("EXPORT_METHOD(::hicc::make_unique_cast<typename ::hicc::ProxyType<{ty}>::type, {ty}, ::hicc::AbiClass<{ty}>"));
        let args = self.input[x3 + 1..x4].trim();
        if !args.is_empty() {
            codes.push_str(", ");
            codes.push_str(args);
        }
        codes.push_str(">);\n");
        Ok(codes)
    }

    pub fn get_cpp_type(ty: &str) -> Option<&str> {
        let mut stream = TokenStream::new(ty);
        while let Some(path) = stream.next_path() {
            if !matches!(path, "const" | "volatile") {
                return Some(path);
            }
        }
        None
    }

    pub fn error_make_proxy(&self) -> &str {
        "wrong format with @make_proxy"
    }
    pub fn error_dynamic(&self) -> &str {
        "wrong type with dynamic_cast"
    }

    pub fn error_argument(&self, arg: &str) -> String {
        format!("error argument: {arg}")
    }

    pub fn scope(&self) -> &str {
        if self.cpp_info.is_member {
            "Self::"
        } else {
            ""
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let input = "void foo ()";
        let Some((x1, x2, x3, x4, x5)) = parse_function(input) else {
            assert!(false);
            return;
        };
        assert_eq!(&input[x1..x2], "foo");
        assert_eq!(&input[x3..x4 + 1], "()");
        assert_eq!(x4 + 1, x5);

        let input = "void foo <int, float> () const volatile &&";
        let Some((x1, x2, x3, x4, x5)) = parse_function(input) else {
            assert!(false);
            return;
        };
        assert_eq!(&input[x1..x2], "foo <int, float>");
        assert_eq!(&input[x3..x4 + 1], "()");
        assert_eq!(&input[x4 + 1..x5], " const volatile &&");

        let input = "void (*foo ())()";
        let Some((x1, x2, x3, x4, x5)) = parse_function(input) else {
            assert!(false);
            return;
        };
        assert_eq!(&input[x1..x2], "foo");
        assert_eq!(&input[x3..x4 + 1], "()");
        assert_eq!(&input[x4 + 1..x5], "");

        let input = "void (*foo<int, float> (int, int) const volatile)()";
        let Some((x1, x2, x3, x4, x5)) = parse_function(input) else {
            assert!(false);
            return;
        };
        assert_eq!(&input[x1..x2], "foo<int, float>");
        assert_eq!(&input[x3..x4 + 1], "(int, int)");
        assert_eq!(&input[x4 + 1..x5], " const volatile");

        let input = "void (*std::foo<std::int, float> (int(*)(), std::int<i32>))()";
        let Some((x1, x2, x3, x4, x5)) = parse_function(input) else {
            assert!(false);
            return;
        };
        assert_eq!(&input[x1..x2], "std::foo<std::int, float>");
        assert_eq!(&input[x3..x4 + 1], "(int(*)(), std::int<i32>)");
        assert_eq!(&input[x4 + 1..x5], "");
    }

    #[test]
    fn test1() {
        let mut it = Args::new("");
        assert_eq!(it.argc(), 0);
        assert_eq!(it.next(), None);

        let mut it = Args::new("const int, const char* [4]");
        assert_eq!(it.next(), Some("const int"));
        assert_eq!(it.next(), Some("const char* [4]"));

        let mut it = Args::new("const int, const char* [4]");
        assert_eq!(it.argc(), 2);
        assert_eq!(it.next(), Some("const int"));
        assert_eq!(it.next(), Some("const char* [4]"));

        let mut it = Args::new("int n, const char* name");
        assert_eq!(it.next(), Some("int n"));
        assert_eq!(it.next(), Some("const char* name"));

        let mut it = Args::new("int n[3], const char* (*(*name[3])(int))()");
        assert_eq!(it.next(), Some("int n[3]"));
        assert_eq!(it.next(), Some("const char* (*(*name[3])(int))()"));
    }

    #[test]
    fn test2() {
        let ty = "const int";
        assert_eq!(
            Arg::new(ty).with_name("name"),
            Some("const int name".to_string())
        );
        let ty = "const int [4]";
        assert_eq!(
            Arg::new(ty).with_name("name"),
            Some("const int  name[4]".to_string())
        );
        let ty = "int (*)(int)";
        assert_eq!(
            Arg::new(ty).with_name("name"),
            Some("int (* name)(int)".to_string())
        );
        let ty = "int (* [5])(int)";
        assert_eq!(
            Arg::new(ty).with_name("name"),
            Some("int (*  name[5])(int)".to_string())
        );
    }
}
