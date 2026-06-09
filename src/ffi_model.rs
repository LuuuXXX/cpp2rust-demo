//! FFI 中间表示（IR）— Phase 3～5
//!
//! `FfiSpec` 描述从 C++ AST 提取后、待生成 hicc Rust 代码的完整 FFI 规格。

/// 顶层 FFI 规格
#[derive(Debug, Default)]
pub struct FfiSpec {
    /// 编译单元名称（如 "class_basic"）
    pub unit_name: String,
    /// hicc::cpp! 块的内容（不含外层大括号）
    pub cpp_block_lines: Vec<String>,
    /// 每个 C++ 类对应一个 ClassSpec（用于生成 import_class! 块）
    pub class_specs: Vec<ClassSpec>,
    /// import_lib! 块规格
    pub lib_spec: LibSpec,
}

/// 单个类的绑定规格
#[derive(Debug, Default)]
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

/// 类方法绑定
#[derive(Debug)]
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
#[derive(Debug, PartialEq)]
pub enum SelfKind {
    /// 常量方法（`&self`）
    Ref,
    /// 可变方法（`&mut self`）
    RefMut,
}

/// import_lib! 块规格
#[derive(Debug, Default)]
pub struct LibSpec {
    /// 链接库名（如 "class_basic"）
    pub link_name: String,
    /// 类前向声明（如 `["class Counter;", "class Dog;"]`）
    pub fwd_decls: Vec<String>,
    /// 函数绑定列表
    pub fn_bindings: Vec<FnBinding>,
}

/// import_lib! 中单个函数绑定
#[derive(Debug)]
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
