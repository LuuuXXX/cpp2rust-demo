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
    /// 模板类绑定规格（v6 Phase B）。v7 起由生成器**默认输出**泛型骨架（IR 非空即输出）。
    pub template_classes: Vec<TemplateClassSpec>,
    /// 模板函数绑定规格（v6 Phase B）。v7 起默认输出。
    pub template_functions: Vec<TemplateFnSpec>,
    /// 模板实例化别名规格（v6 Phase B 增强）。v7 起由生成器**默认输出**类型别名骨架
    /// （如 `pub type StackI32 = Stack<hicc::Pod<i32>>;`）。
    pub template_instances: Vec<TemplateInstanceSpec>,
    /// 模板实例化构造工厂骨架（v6 Phase B 增强（续））。由模板类构造函数派生，`T` 替换为
    /// 具体实例化类型。v7 起由生成器**默认**在 `import_lib!` 中输出工厂骨架。
    pub template_factories: Vec<TemplateFactorySpec>,
    /// `@make_proxy` 代理工厂骨架（v6 Phase C）。由「继承 C++ 抽象接口的具体类」的公有
    /// 构造函数派生，使 Rust 侧可实现 C++ 接口。v7 起由生成器**默认**在 `import_lib!` 中输出。
    pub proxy_factories: Vec<ProxyFactorySpec>,
    /// `@dynamic_cast` 下行转换绑定骨架（v6 Phase C（续））。由「继承多态基类的派生类」派生，
    /// 用于在 RTTI 场景把多态基类指针向下转换为派生类指针，替代 v5 的整数枚举绕过方案。
    /// v7 起由生成器**默认**在 `import_lib!` 中输出。
    pub dynamic_casts: Vec<DynamicCastSpec>,
    /// 头文件中完整定义的 POD 结构体（如 SAX 回调表 `RapidJsonHandlerCallbacks`），
    /// 被 FFI 函数签名以指针引用但不属于不透明句柄。这类类型须以 `#[repr(C)]` Rust 结构体
    /// 输出（而非不透明 `import_class!`），否则会与 hicc 的 `MethodsType` 特化冲突。
    pub repr_c_structs: Vec<ReprCStructSpec>,
}

/// 头文件中完整定义的 POD 结构体规格 — 以 `#[repr(C)]` Rust 结构体输出。
///
/// 用于 SAX 回调表等「调用方按值构造、按指针传入」的纯数据结构（如
/// `RapidJsonHandlerCallbacks`）。这类类型在头文件中有完整字段定义、无成员方法、
/// 无基类，被 FFI 函数签名引用但不应作为不透明句柄（`import_class!`）处理——
/// 后者会让 hicc 生成 `MethodsType` 特化，与头文件中真实的 POD 定义冲突。
#[derive(Debug, Default, Clone)]
pub struct ReprCStructSpec {
    /// 结构体名（如 `RapidJsonHandlerCallbacks`）
    pub name: String,
    /// 字段列表 `(rust_name, rust_type)`，按声明顺序排列以保持 ABI 布局。
    pub fields: Vec<(String, String)>,
}

/// `@dynamic_cast` 下行转换绑定骨架规格 — v6 Phase C（续，高级映射）
///
/// 由提取器从「继承多态基类（含虚函数）的派生类」派生：对每个 `(多态基类, 派生类)` 关系
/// 生成 hicc `import_lib!` 中的下行转换骨架
/// `#[cpp(func = "const Derived* @dynamic_cast<const Derived*>(const Base*)")]`，
/// 用于 RTTI 场景安全地把多态基类指针向下转换为派生类指针（见
/// `references/hicc/examples/dynamic_cast`），替代 v5 的整数枚举绕过方案
/// （对应 v6 方案 §3.2 示例 023 typeid_rtti）。
///
/// 转换失败时 `@dynamic_cast` 返回空指针，因此 Rust 侧返回裸指针 `*const Derived`，
/// 调用方需自行判空。由于该绑定为骨架，生成时附带 `cpp2rust-todo[DCAST]` 提示，
/// 需用户结合实际类型确认（符合 v6 方案 §8 的降级策略）。
///
/// v6 Phase C（收尾）另派生**引用形式**（`&Src -> &Dst`，函数名以 `_ref` 结尾）：
/// hicc 允许同一个指针型 C++ 签名的 Rust 侧返回 `&Derived`（见
/// `references/hicc/examples/dynamic_cast` 的 `as_foo(&self) -> &Foo`）。引用形式更符合
/// Rust 习惯，但**要求转换必定成功**——若转换失败（基类指针实际并非该派生类），将由空指针
/// 构造引用而导致未定义行为，因此仅在调用方能确保类型成立时使用，否则应改用裸指针形式判空。
#[derive(Debug, Default, Clone)]
pub struct DynamicCastSpec {
    /// Rust 函数名（裸指针形式，如 `dynamic_cast_foo_to_bar`）
    pub rust_name: String,
    /// Rust 函数名（引用形式，如 `dynamic_cast_foo_to_bar_ref`）
    pub ref_rust_name: String,
    /// 源类型名（多态基类，如 `Foo`）
    pub src_class: String,
    /// 目标类型名（派生类，如 `Bar`）
    pub dst_class: String,
    /// C++ 转换签名（用于 `#[cpp(func = "...")]`，如
    /// `const Bar* @dynamic_cast<const Bar*>(const Foo*)`）
    pub cpp_sig: String,
}

/// `@make_proxy` 代理工厂骨架规格 — v6 Phase C（高级映射）
///
/// 由提取器从「继承 C++ 抽象接口（纯虚类）的具体类」的公有构造函数派生，生成 hicc
/// `import_lib!` 中结合 `#[interface(name = ...)]` 的 `@make_proxy` 工厂骨架，使 Rust 侧
/// 可通过组合模式实现 C++ 抽象类（见 `references/hicc/examples/interface`）。
///
/// 由于代理工厂需用户在 Rust 侧提供接口实现类，且构造函数参数类型列表须与 `@make_proxy`
/// 一致，本规格生成的是**带 `cpp2rust-todo[PROXY]` 提示的骨架**（与其余高级映射能力一致），
/// 需用户结合实际接口实现补全（符合 v6 方案 §8 的降级策略）。
#[derive(Debug, Default, Clone)]
pub struct ProxyFactorySpec {
    /// Rust 工厂函数名（如 `new_rust_baz`）
    pub rust_name: String,
    /// 具体类名（既作为 `@make_proxy<...>` 的实参，也作为 Rust 返回类型，如 `Baz`）
    pub concrete_class: String,
    /// 直接接口基类名（用于 `#[interface(name = "...")]`，如 `Bar`）
    pub interface_name: String,
    /// C++ 工厂签名（用于 `#[cpp(func = "...")]`，如 `Baz @make_proxy<Baz>(int)`）
    pub cpp_sig: String,
    /// 构造函数参数列表 (rust_name, rust_type)；生成时位于 `intf: hicc::Interface<...>` 之后
    pub params: Vec<(String, String)>,
}

/// 模板实例化构造工厂骨架规格 — v6 Phase B 增强（续）
///
/// 由提取器从模板类的公有构造函数派生，并将类型参数 `T` 替换为某个实例化的具体类型，
/// 生成 hicc `import_lib!` 中的工厂函数骨架，使实例化别名（[`TemplateInstanceSpec`]）
/// 可向真实构造调用靠拢。
///
/// 由于模板类构造函数对应的 C++ 符号通常需用户在 C++ 侧显式实例化 / 包装后才存在，
/// 本规格生成的是**带 `cpp2rust-todo[TMPL]` 提示的骨架**（与 Phase B 其余模板能力一致），
/// 需用户结合实际符号与 hicc 约定补全（符合 v6 方案 §8 的降级策略）。
#[derive(Debug, Default, Clone)]
pub struct TemplateFactorySpec {
    /// Rust 工厂函数名（如 `stack_i32_new`）
    pub rust_name: String,
    /// 实例化别名（作为 Rust 返回类型，如 `StackI32`）
    pub alias_name: String,
    /// C++ 工厂签名（用于 `#[cpp(func = "...")]`，如 `Stack<int>* stack_i32_new(int value)`）
    pub cpp_sig: String,
    /// 参数列表 (rust_name, rust_type)
    pub params: Vec<(String, String)>,
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
    /// 原始的具体 C++ 类型实参（如 `["int"]`），用于派生构造工厂的 C++ 签名（如 `Stack<int>`）。
    pub cpp_args: Vec<String>,
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
    /// C++ 类名（Rust 侧类型名，简单名如 `Counter`）
    pub name: String,
    /// 非 ctor/dtor 方法绑定列表（有 self 的成员方法）
    pub methods: Vec<MethodBinding>,
    /// ctor/dtor/factory 关联函数（无 self）；非空时在 import_lib! 中生成 class body 格式
    pub associated_fns: Vec<FnBinding>,
    /// dtor shim 函数名（如 `foo_delete`）；有值时在 #[cpp(class = "...")] 中生成 destroy = "..."
    pub destroy_fn: Option<String>,
    /// 是否为纯虚接口类（所有 public 方法均为纯虚）；true 时生成 #[interface]
    pub is_interface: bool,
    /// hicc 直出模式：true 时生成 `#[cpp(class = cpp_class)]` 直接绑定真实命名空间类，
    /// 构造由 `ctor_factories`（make_unique 工厂）负责、析构交给 hicc 的 `Drop`，
    /// 不再生成 `destroy =`/opaque 指针 shim。
    pub hicc_direct: bool,
    /// hicc 直出时的 C++ `::` 限定类名（如 `class_basic_ns::Counter`）；
    /// 用于 `#[cpp(class = "...")]`。`None` 时回退到 `name`。
    pub cpp_class: Option<String>,
    /// hicc 直出时的构造工厂（每个公有构造函数一条 make_unique 工厂）。
    pub ctor_factories: Vec<CtorFactory>,
}

/// hicc 直出构造工厂（替代旧的 `*_new` C ABI 桥接）。
///
/// 在 `import_class!` body 内生成关联函数 `pub fn <ctor_fn>(...) -> Self { <factory_rust_name>(...) }`，
/// 并在 `import_lib!` 输出对应的 `hicc::make_unique<T, Args...>` 工厂。
#[derive(Debug, Default, Clone)]
pub struct CtorFactory {
    /// `import_class!` body 内关联函数名（如 `new`、`with_name`）
    pub ctor_fn: String,
    /// `import_lib!` 工厂函数名（如 `counter_new`）
    pub factory_rust_name: String,
    /// 关联函数参数列表 (rust_name, rust_type)，用于 in-class `fn` 签名与转发实参
    pub params: Vec<(String, String)>,
    /// `import_lib!` 工厂的 C++ make_unique 签名（用于 `#[cpp(func = "...")]`）
    pub make_unique_sig: String,
    /// Rust 返回类型名（如 `Counter`）
    pub ret_class: String,
    /// 工厂名是否需要 `#[allow(non_snake_case)]`
    pub non_snake_case: bool,
}

impl ClassSpec {
    /// 若 methods、associated_fns 均为空且没有 destroy_fn，则返回 `true`。
    /// 与 `hicc_codegen::generate` 的跳过条件一致：空 `ClassSpec` 不生成 `import_class!` 块。
    pub fn is_empty(&self) -> bool {
        self.methods.is_empty()
            && self.associated_fns.is_empty()
            && self.destroy_fn.is_none()
            && self.ctor_factories.is_empty()
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
