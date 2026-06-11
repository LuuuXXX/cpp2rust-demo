//! FFI 中间表示（IR）— Phase 3～5
//!
//! `FfiSpec` 描述从 C++ AST 提取后、待生成 hicc Rust 代码的完整 FFI 规格。

/// 顶层 FFI 规格
#[derive(Debug, Default, Clone)]
pub struct FfiSpec {
    /// 编译单元名称（如 "class_basic"）
    pub unit_name: String,
    /// hicc::cpp! 块的内容（不含外层大括号）
    pub cpp_block_lines: Vec<String>,
    /// 每个 C++ 类对应一个 ClassSpec（用于生成 import_class! 块）
    pub class_specs: Vec<ClassSpec>,
    /// import_lib! 块规格
    pub lib_spec: LibSpec,
    /// 模板类绑定规格（Phase B）：消费 AST 的模板类信息生成泛型 `import_class!` 骨架。
    /// 仅在 `CPP2RUST_GEN_TEMPLATES` 开启时由生成器消费；默认不消费，故不改变默认产物。
    pub template_classes: Vec<TemplateClassSpec>,
    /// 模板函数绑定规格（Phase B）：消费 AST 的模板函数信息生成泛型 `import_lib!` 骨架。
    pub template_fns: Vec<TemplateFnSpec>,
}

/// 模板类绑定规格（Phase B）。
///
/// 与 [`ClassSpec`] 平行，但额外携带泛型形参名列表 `type_params`，用于生成
/// `pub class Name<T> { ... }` 形式的泛型 `import_class!` 骨架。
///
/// 当前仅生成泛型骨架（成员方法签名沿用 C++ 模板成员签名），具体实例化类型
/// （如 `Stack<hicc::Pod<i32>>`）的别名与工厂绑定需后续阶段补充——生成器为此
/// 输出 `cpp2rust-todo[TPL]` 提示，符合既有降级标记约定。
#[derive(Debug, Default, Clone)]
pub struct TemplateClassSpec {
    /// 泛型类名（如 `"Stack"`）
    pub name: String,
    /// 泛型形参名列表（如 `["T"]`；非类型/模板模板形参同样以名字列出）
    pub type_params: Vec<String>,
    /// 成员方法绑定（复用 [`MethodBinding`]）
    pub methods: Vec<MethodBinding>,
}

impl TemplateClassSpec {
    /// 无任何可生成的成员方法时返回 `true`（与 `ClassSpec::is_empty` 语义一致）。
    pub fn is_empty(&self) -> bool {
        self.methods.is_empty()
    }
}

/// 模板函数绑定规格（Phase B）。
///
/// 与 [`FnBinding`] 平行，但携带泛型形参名列表 `type_params`，`cpp_sig` 保留
/// C++ 模板形参（如 `void do_swap<T>(T*, T*)`）。具体实例化类型需后续阶段补充。
#[derive(Debug, Default, Clone)]
pub struct TemplateFnSpec {
    /// 函数名（如 `"do_swap"`）
    pub name: String,
    /// 泛型形参名列表（如 `["T"]`）
    pub type_params: Vec<String>,
    /// C++ 模板函数签名（含泛型形参），用于 `#[cpp(func = "...")]`
    pub cpp_sig: String,
    /// Rust 函数名（snake_case）
    pub rust_name: String,
    /// 参数列表 (rust_name, rust_type)
    pub params: Vec<(String, String)>,
    /// 返回类型（None 表示 void）
    pub ret_type: Option<String>,
}

/// 单个类的绑定规格
#[derive(Debug, Default, Clone)]
pub struct ClassSpec {
    /// C++ 类名
    pub name: String,
    /// 非 ctor/dtor 方法绑定列表（有 self 的成员方法）
    pub methods: Vec<MethodBinding>,
    /// ctor/dtor/factory 关联函数（无 self）；非空时在 import_lib! 中生成 class body 格式
    pub associated_fns: Vec<FnBinding>,
    /// dtor shim 函数名（如 `foo_delete`）；有值时在 #[cpp(class = "...")] 中生成 destroy = "..."
    pub destroy_fn: Option<String>,
    /// 是否为纯虚接口类（所有 public 方法均为纯虚）；true 时生成 #[interface]
    pub is_interface: bool,
}

impl ClassSpec {
    /// 若 methods、associated_fns 均为空且没有 destroy_fn，则返回 `true`。
    /// 与 `hicc_codegen::generate` 的跳过条件一致：空 `ClassSpec` 不生成 `import_class!` 块。
    pub fn is_empty(&self) -> bool {
        self.methods.is_empty() && self.associated_fns.is_empty() && self.destroy_fn.is_none()
    }
}

/// 类方法绑定
#[derive(Debug, Clone)]
pub struct MethodBinding {
    /// C++ 方法签名（用于 #[cpp(method = "...")] 属性），例如 `int get() const`
    pub cpp_sig: String,
    /// Rust 函数名（snake_case），例如 `get_value`
    pub rust_name: String,
    /// &self 或 &mut self
    pub self_kind: SelfKind,
    /// 参数列表 (rust_name, rust_type)
    pub params: Vec<(String, String)>,
    /// 返回类型（None 表示 void）
    pub ret_type: Option<String>,
    /// 参数或返回类型含 C 函数指针（用于生成 cpp2rust-todo[FP] 注释）
    pub has_fn_ptr_param: bool,
}

/// import_class! 方法的 self 参数类型
///
/// 由 `extractor/class_spec.rs` 的 `build_method_binding` 根据 `MethodInfo::is_const` 字段决定：
/// - `is_const == true`（C++ `const` 成员函数）→ [`SelfKind::Ref`]
/// - `is_const == false`（非 const 成员函数）→ [`SelfKind::RefMut`]
///
/// **关于 `volatile` 方法**：C++ `volatile` 限定的成员函数不映射为任何 SelfKind 变体；
/// 它们在 `class_spec.rs` 的过滤阶段通过 `MethodInfo::is_volatile` 检查被提前排除，
/// 因为 Rust 没有对应的 `volatile self` 语义，无法安全地生成 hicc 绑定。
#[derive(Debug, PartialEq, Clone)]
pub enum SelfKind {
    /// 常量方法对应 `&self`（C++ `const` 成员函数，`is_const == true`）
    Ref,
    /// 可变方法对应 `&mut self`（非 const 成员函数，`is_const == false`）
    RefMut,
}

/// import_lib! 块规格
#[derive(Debug, Default, Clone)]
pub struct LibSpec {
    /// 链接库名（如 "class_basic"）
    pub link_name: String,
    /// 类前向声明（如 `["class Counter;", "class Dog;"]`）
    pub fwd_decls: Vec<String>,
    /// 函数绑定列表
    pub fn_bindings: Vec<FnBinding>,
}

/// import_lib! 中单个函数绑定
#[derive(Debug, Clone)]
pub struct FnBinding {
    /// C++ 函数签名（用于 #[cpp(func = "...")] 属性），例如 `Counter* counter_new()`
    pub cpp_sig: String,
    /// Rust 函数名（snake_case）
    pub rust_name: String,
    /// 参数列表 (rust_name, rust_type)
    pub params: Vec<(String, String)>,
    /// 返回类型（None 表示 void）
    pub ret_type: Option<String>,
    /// 是否需要 unsafe 关键字
    pub is_unsafe: bool,
    /// 参数或返回类型含 C 函数指针（用于生成 cpp2rust-todo[FP] 注释）
    pub has_fn_ptr_param: bool,
}
