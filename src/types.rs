#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppAst {
    pub source_file: String,
    pub classes: Vec<CppClass>,
    pub functions: Vec<CppFunction>,
    pub enums: Vec<CppEnum>,
    pub namespaces: Vec<String>,
    pub template_instantiations: Vec<TemplateInstantiation>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppClass {
    pub name: String,
    pub namespace: Vec<String>,
    pub methods: Vec<CppMethod>,
    pub constructors: Vec<CppConstructor>,
    pub has_destructor: bool,
    pub bases: Vec<String>,
    pub is_abstract: bool,
    pub is_template_specialization: bool,
    pub template_args: Vec<String>,
    pub static_members: Vec<CppStaticMember>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppMethod {
    pub name: String,
    pub return_type: CppType,
    pub params: Vec<CppParam>,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_override: bool,
    pub is_static: bool,
    pub is_volatile: bool,
    pub is_friend: bool,
    pub operator_kind: Option<OperatorKind>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppConstructor {
    pub params: Vec<CppParam>,
    pub is_copy: bool,
    pub is_move: bool,
    pub is_explicit: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppStaticMember {
    pub name: String,
    pub type_: CppType,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppFunction {
    pub name: String,
    pub namespace: Vec<String>,
    pub return_type: CppType,
    pub params: Vec<CppParam>,
    pub is_inline: bool,
    pub is_variadic: bool,
    pub is_extern_c: bool,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppEnum {
    pub name: String,
    pub namespace: Vec<String>,
    pub is_scoped: bool,
    pub variants: Vec<CppEnumVariant>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppEnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppParam {
    pub name: String,
    pub type_: CppType,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CppType {
    pub name: String,
    pub is_pointer: bool,
    pub is_reference: bool,
    pub is_const: bool,
    pub is_rvalue_ref: bool,
}

impl CppType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_pointer: false,
            is_reference: false,
            is_const: false,
            is_rvalue_ref: false,
        }
    }

    pub fn to_rust_ffi(&self) -> String {
        let base = cpp_type_to_rust(&self.name);
        if self.is_pointer || self.is_reference || self.is_rvalue_ref {
            if self.is_const {
                format!("*const {base}")
            } else {
                format!("*mut {base}")
            }
        } else {
            base
        }
    }
}

pub fn cpp_type_to_rust(cpp_type: &str) -> String {
    match cpp_type.trim() {
        "void" => "()".to_string(),
        "bool" => "bool".to_string(),
        "char" => "i8".to_string(),
        "unsigned char" | "uint8_t" => "u8".to_string(),
        "short" | "short int" | "int16_t" => "i16".to_string(),
        "unsigned short" | "uint16_t" => "u16".to_string(),
        "int" | "int32_t" => "i32".to_string(),
        "unsigned int" | "unsigned" | "uint32_t" => "u32".to_string(),
        "long" | "long int" | "int64_t" => "i64".to_string(),
        "unsigned long" | "unsigned long int" | "uint64_t" => "u64".to_string(),
        "long long" | "long long int" => "i64".to_string(),
        "unsigned long long" | "unsigned long long int" => "u64".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "size_t" => "usize".to_string(),
        "ptrdiff_t" | "ssize_t" => "isize".to_string(),
        "const char*" | "char*" => "*const i8".to_string(),
        t => {
            let clean = t.split("::").last().unwrap_or(t);
            let clean = if let Some(idx) = clean.find('<') {
                &clean[..idx]
            } else {
                clean
            };
            clean.trim().to_string()
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TemplateInstantiation {
    pub template_name: String,
    pub type_args: Vec<String>,
    pub instantiated_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum OperatorKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Shl,
    Shr,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Index,
    Call,
    Arrow,
    Deref,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
    Other(String),
}

impl Default for OperatorKind {
    fn default() -> Self {
        Self::Other(String::new())
    }
}

impl OperatorKind {
    pub fn from_cpp_name(name: &str) -> Option<Self> {
        match name {
            "operator+" => Some(Self::Add),
            "operator-" => Some(Self::Sub),
            "operator*" => Some(Self::Mul),
            "operator/" => Some(Self::Div),
            "operator%" => Some(Self::Mod),
            "operator==" => Some(Self::Eq),
            "operator!=" => Some(Self::Ne),
            "operator<" => Some(Self::Lt),
            "operator<=" => Some(Self::Le),
            "operator>" => Some(Self::Gt),
            "operator>=" => Some(Self::Ge),
            "operator&&" => Some(Self::And),
            "operator||" => Some(Self::Or),
            "operator!" => Some(Self::Not),
            "operator&" => Some(Self::BitAnd),
            "operator|" => Some(Self::BitOr),
            "operator^" => Some(Self::BitXor),
            "operator~" => Some(Self::BitNot),
            "operator<<" => Some(Self::Shl),
            "operator>>" => Some(Self::Shr),
            "operator=" => Some(Self::Assign),
            "operator+=" => Some(Self::AddAssign),
            "operator-=" => Some(Self::SubAssign),
            "operator*=" => Some(Self::MulAssign),
            "operator/=" => Some(Self::DivAssign),
            "operator[]" => Some(Self::Index),
            "operator()" => Some(Self::Call),
            "operator->" => Some(Self::Arrow),
            "operator++" => Some(Self::PreInc),
            "operator--" => Some(Self::PreDec),
            s if s.starts_with("operator") => Some(Self::Other(s.to_string())),
            _ => None,
        }
    }

    pub fn to_rust_name(&self, class_name: &str) -> String {
        let class_lower = class_name.to_lowercase();
        match self {
            Self::Add => format!("{class_lower}_add"),
            Self::Sub => format!("{class_lower}_sub"),
            Self::Mul => format!("{class_lower}_mul"),
            Self::Div => format!("{class_lower}_div"),
            Self::Mod => format!("{class_lower}_mod_"),
            Self::Eq => format!("{class_lower}_eq"),
            Self::Ne => format!("{class_lower}_ne"),
            Self::Lt => format!("{class_lower}_lt"),
            Self::Le => format!("{class_lower}_le"),
            Self::Gt => format!("{class_lower}_gt"),
            Self::Ge => format!("{class_lower}_ge"),
            Self::And => format!("{class_lower}_and"),
            Self::Or => format!("{class_lower}_or"),
            Self::Not => format!("{class_lower}_not"),
            Self::BitAnd => format!("{class_lower}_bitand"),
            Self::BitOr => format!("{class_lower}_bitor"),
            Self::BitXor => format!("{class_lower}_bitxor"),
            Self::BitNot => format!("{class_lower}_bitnot"),
            Self::Shl => format!("{class_lower}_shl"),
            Self::Shr => format!("{class_lower}_shr"),
            Self::Assign => format!("{class_lower}_assign"),
            Self::AddAssign => format!("{class_lower}_add_assign"),
            Self::SubAssign => format!("{class_lower}_sub_assign"),
            Self::MulAssign => format!("{class_lower}_mul_assign"),
            Self::DivAssign => format!("{class_lower}_div_assign"),
            Self::Index => format!("{class_lower}_index"),
            Self::Call => format!("{class_lower}_call"),
            Self::Arrow => format!("{class_lower}_arrow"),
            Self::Deref => format!("{class_lower}_deref"),
            Self::PreInc => format!("{class_lower}_pre_inc"),
            Self::PreDec => format!("{class_lower}_pre_dec"),
            Self::PostInc => format!("{class_lower}_post_inc"),
            Self::PostDec => format!("{class_lower}_post_dec"),
            Self::Other(s) => format!("{}_{}", class_lower, s.replace("operator", "op_")),
        }
    }
}
