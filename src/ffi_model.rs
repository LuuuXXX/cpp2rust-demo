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
    /// 模板类绑定规格（v6 Phase B）。仅在 `CPP2RUST_GEN_TEMPLATES` 开启时由生成器输出泛型骨架；
    /// 默认关闭时不影响产物。
    pub template_classes: Vec<TemplateClassSpec>,
    /// 模板函数绑定规格（v6 Phase B）。同上。
    pub template_functions: Vec<TemplateFnSpec>,
    /// 模板实例化别名规格（v6 Phase B 增强）。仅在 `CPP2RUST_GEN_TEMPLATES` 开启时由生成器
    /// 输出类型别名骨架（如 `pub type StackI32 = Stack<hicc::Pod<i32>>;`），默认关闭时不影响产物。
    pub template_instances: Vec<TemplateInstanceSpec>,
}

/// 模板实例化别名规格（生成具体实例化类型别名）— v6 Phase B 增强
///
/// 由提取器从当前编译单元中「以具体类型实例化某模板类」的使用点（目前为包装类的字段类型，
/// 如 `Stack<int> impl;`）收集而来，生成 hicc 形式的类型别名骨架：
/// `pub type <alias_name> = <template_name><<hicc_args>>;`。
#[derive(Debug, Default, Clone)]
pub struct TemplateInstanceSpec {
    /// Rust 侧别名（如 `StackI32`）
    pub alias_name: String,
    /// 模板类名（如 `Stack`）
    pub template_name: String,
    /// hicc 形式的实例化类型实参（POD 标量为 `hicc::Pod<i32>`；类类型保留原 C++ 名并附 TODO）
    pub hicc_args: Vec<String>,
    /// 是否含无法判定为 POD 的类类型实参（true 时生成 cpp2rust-todo[TMPL] 提示用户确认 hicc 类型）
    pub needs_class_type: bool,
}

/// 模板类绑定规格（生成泛型 `import_class!` 骨架）— v6 Phase B
#[derive(Debug, Default, Clone)]
pub struct TemplateClassSpec {
    /// 模板类名（如 `Stack`）
    pub name: String,
    /// 类型参数名（如 `["T"]`）
    pub type_params: Vec<String>,
    /// 成员方法绑定（复用 [`MethodBinding`]，签名中保留泛型 `T`）
    pub methods: Vec<MethodBinding>,
}

/// 模板函数绑定规格（生成泛型 `import_lib!` 骨架）— v6 Phase B
#[derive(Debug, Default, Clone)]
pub struct TemplateFnSpec {
    /// 模板函数名（如 `do_swap`）
    pub name: String,
    /// 类型参数名（如 `["T"]`）
    pub type_params: Vec<String>,
    /// C++ 模板函数签名（含实例化占位，如 `void do_swap<T>(T*, T*)`）
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
