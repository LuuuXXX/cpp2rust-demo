# 一、hicc 功能全览

## 1.1 核心原理

hicc 是一个 **C++ → Rust FFI 互操作框架**，核心思路：
- C++ 对象在 Rust 侧用 `struct CppObject { methods: &'static VTable, obj: *const () }` 代理，屏蔽 C++ 内存布局和 MOVE 语义差异
- 通过过程宏（`hicc::cpp!` / `hicc::import_lib!` / `hicc::import_class!`）在 `.rs` 文件中内嵌 C++ 代码，由 `hicc-build` 在 `build.rs` 阶段自动生成适配层并编译为静态库
- 利用 Rust 生命周期约束消除 C++ 悬垂引用等内存安全风险

## 1.2 hicc 支持 / 不支持特性表

| 类别 | C++ 特性 | 状态 | 说明 |
|------|---------|:----:|------|
| **数据类型** | 值类型 `T` | ✅ 支持 | |
| | `const T&` / `T&` | ✅ 支持 | |
| | `T&&`（右值引用） | ✅ 支持 | Rust 侧等同 `T`（所有权转移） |
| | `const T*` / `T*` | ✅ 支持 | 生命周期由程序员管理 |
| | `const T**` / `T**` 多重指针 | ✅ 支持 | `ClassPtr<'a, T, N>` |
| | `std::function<R(Args...)>` | ✅ 支持 | 对应 Rust 闭包 |
| | POD 类型 | ✅ 支持 | `hicc::Pod<T>` 包装 |
| **函数** | 自由函数（外/内/无链接） | ✅ 支持 | `import_lib!` + `#[cpp(func)]` |
| | 函数重载 | ✅ 支持 | Rust 侧不同函数名映射 |
| | 默认参数（可忽略） | ✅ 支持 | Rust 函数参数可少于 C++ |
| | 忽略返回值 | ✅ 支持 | Rust 函数返回 `()` |
| | 捕获 C++ 异常 | ✅ 支持 | `hicc::Exception<T>` 包装返回值 |
| | 模板函数（需具体实例化） | ⚠️ 部分支持 | 需提供完整 `func<T, ...>(...)` 签名 |
| | `va_list` 可变参 | ✅ 支持 | C++ 最后参数为 `va_list` |
| | `...` variadic（C 风格） | ⚠️ 部分支持 | 仅全局函数，参数/返回值**不能**含 C++ 类类型 |
| **类** | 类成员函数（实例方法） | ✅ 支持 | `import_class!` + `#[cpp(method)]` |
| | `const` 方法 | ✅ 支持 | 映射为 `&self` |
| | 非 `const` 方法 | ✅ 支持 | 映射为 `&mut self` |
| | `&&` 右值引用方法 | ✅ 支持 | 映射为 `self`（消耗所有权） |
| | 静态方法 | ✅ 支持 | 在 `import_lib!` 中声明，可用 `#[method(class=..., name=...)]` 绑定为关联函数 |
| | 构造函数 | ✅ 支持 | 在 `import_lib!` 中声明，`#[cpp(class=..., ctor=...)]` |
| | 私有析构（自定义 destroy） | ✅ 支持 | `#[cpp(class=..., destroy=...)]` |
| | 普通/非纯虚方法 | ✅ 支持 | hicc 通过 vtable 透明调用 |
| | 全纯虚抽象类（接口） | ✅ 支持 | `#[interface]` 映射为 Rust Trait，`@make_proxy` 反向实现 |
| | 模板类（需实例化） | ✅ 支持 | `template<class T> ClassName<T>`，需要 `AbiType` 约束 |
| | `dynamic_cast` | ✅ 支持 | `@dynamic_cast` 内置函数 |
| | public 单继承 | ✅ 支持 | `class Derived: Base` 语法 |
| | 多重继承 | ❌ 不支持 | — |
| | 虚继承（菱形） | ❌ 不支持 | — |
| | 友元函数 | ❌ 不支持 | — |
| | 运算符重载（`operator`） | ❌ 不支持 | hicc 不支持运算符符号作为绑定名 |
| | 析构函数显式绑定 | ❌ 不支持 | 由 C++ RAII 自动管理 |
| **变量** | 类成员变量 | ✅ 支持 | `#[cpp(field=...)]`，返回只读/可写借用 |
| | 全局变量 / 静态变量 | ✅ 支持 | `#[cpp(data=...)]` |
| **STL** | 全部标准容器（`vector/map/set/...`） | ✅ 支持（`hicc-std`） | 18 种容器，提供安全 Rust API，迭代器二次封装 |
| | `std::string` 参数/返回 | ⚠️ 有限 | `hicc_std::string` 代理类可用，但直接 ABI 传递需 shim |
| **高级** | C++ 容器存储 Rust 数据 | ✅ 支持 | `RustAny` / `RustKey` / `RustHashKey` |
| | Rust 内存空间构造 C++ 对象（placement new） | ✅ 支持 | 返回值生命周期关联输入内存 |
| | `hicc::cpp!` 内嵌 C++ 灵活适配 | ✅ 支持 | 可直接在 `.rs` 文件内写 C++ shim |
| | 引用/指针返回自动适配 | ✅ 支持 | `class T;` 声明后 `&T` 自动转换 `ClassRef<'_, T>` |
| **构建** | C++11+ 自动编译 | ✅ 支持 | `hicc-build` + `build.rs` |

---

# 二、cpp2rust-demo 功能全览

## 2.1 核心原理

`cpp2rust-demo` 是一个 **hicc FFI 脚手架自动生成工具**（不是语义翻译器）：

1. **`init` 阶段**：通过 `LD_PRELOAD` 注入 `hook/libhook.so` 拦截真实 C++ 构建命令，捕获编译单元并生成 `.cpp2rust` 预处理中间件；对选中文件执行 `clang -ast-dump=json`，解析 AST 后通过 `codegen.rs` 输出分组的 Rust 绑定脚手架（`include/types/free/class/method` 分层）
2. **`merge` 阶段**：将多个 `mod_<group>` 模块合并为统一的 `merged_ffi.rs`

## 2.2 cpp2rust-demo 支持 / 不支持特性及对应 hicc 状态

| C++ 特性 | cpp2rust-demo 状态 | 输出位置 | hicc 是否支持 | 说明 |
|---------|:-----------------:|---------|:------------:|------|
| 自由函数（非模板） | ✅ 自动提取 | `free/fn_*.rs` | ✅ | `import_lib!` + `#[cpp(func)]` |
| 函数重载 | ✅ 自动提取 | `free/fn_*.rs` | ✅ | 自动追加 `_2`, `_3` 后缀 |
| 命名空间函数 | ✅ 自动提取 | `free/fn_*.rs` | ✅ | 限定名嵌入 `#[cpp(func)]` |
| 类实例方法（非虚） | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | `import_class!` + `#[cpp(method)]` |
| `const` 方法 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | 映射为 `&self` |
| 非 `const` 方法 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | 映射为 `&mut self` |
| 非纯 `virtual` 方法 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | vtable 透明调用 |
| 全纯虚抽象类 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | `#[interface]` + `@make_proxy` |
| 混合类（纯虚 + 普通方法） | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | 普通方法正常提取；纯虚方法生成 companion interface |
| 构造函数 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | 主构造函数 `ctor="..."`；额外构造为工厂函数 |
| 静态方法 | ✅ 自动提取 | `free/fn_*.rs` | ✅ | `#[cpp(func = "ClassName::method(...)")]` |
| public 单继承 | ✅ 自动提取 | `method/mtd_*.rs` | ✅ | `class Derived: Base` |
| `@make_proxy` 反向绑定 | ✅ 自动生成 | `free/fn_*.rs` | ✅ | 全纯虚类自动生成 |
| 全局变量 | ✅ 自动提取 | `free/fn_*.rs` | ✅ | `#[cpp(data)]` |
| 枚举 `enum` / `enum class` | ✅ 自动提取 | `types/mod.rs` | ✅ | `#[repr(C)] enum` |
| `typedef` / `using` 别名 | ✅ 自动提取 | `types/mod.rs` | ✅ | AliasRegistry，解锁模板提取 |
| 模板特化类（有 typedef/using 别名） | ⚠️ 有条件支持 | `method/mtd_*.rs` | ✅ | 需要别名存在于 AST；`ToolConservative` |
| 模板类（无别名） | ⚠️ 跳过 | — | ✅（hicc 支持） | 添加 `typedef`/`using` 别名后可解锁；`ToolConservative` |
| `std::` 容器参数（无别名） | ⚠️ 跳过 | — | ✅（hicc-std 支持） | 添加 `using` 别名可解锁；`ToolConservative` |
| 函数模板（无显式特化） | ⚠️ 跳过 | — | ⚠️（需实例化） | AST 中需有 concrete specialization；`ToolConservative` |
| 运算符重载 | ⚠️ 半自动 | `free/shim_ops.rs` | ❌ | 生成 `operator_shims.hpp` starter；需手写 C++ shim 再绑定 |
| 析构函数 | ❌ 跳过 | — | ❌ | `HiccLimitation`；由 RAII 管理 |
| 多重继承 | ✅ 全部 public 基类提取 | `method/mtd_*.rs` | ❌ | 所有 public 基类均提取至 `ClassIR.bases`，`render_import_class()` 以 `, ` 分隔列出（P3 已实现） |
| 虚继承（菱形继承） | ⚠️ 跳过并报告 | 接口报告 | ❌ | 虚基类被跳过，接口报告列出警告（P3 已实现） |
| 友元函数 | ❌ 跳过 | — | ❌ | AST 不可靠提取；`HiccLimitation` |
| 函数指针参数 | ⚠️ 跳过并生成接口建议 | 接口报告 | ❌ | `ToolConservative`；接口报告自动生成虚函数接口骨架 + `@make_proxy` 使用提示（P3 已实现） |
| `std::string` 参数/返回值 | ⚠️ 跳过并生成 shim 建议 | 接口报告 / `operator_shims.hpp` | ❌ | 跳过；接口报告和 `operator_shims.hpp` 自动生成 `const char*` shim 原型（P2 已实现） |
| `std::function` / lambda 参数 | ⚠️ 跳过并生成接口建议 | 接口报告 | ✅（hicc 支持） | 跳过；接口报告自动生成虚函数接口 + `@make_proxy` 使用骨架（P2 已实现） |
| `auto` / `decltype` 返回类型 | ❌ 跳过 | — | ❌ | `HiccLimitation`；需手写包装函数 |
| `va_list` / variadic `...` | ✅ 自动提取（`va_list` 最后参数） | `free/fn_*.rs` | ⚠️（hicc 部分支持） | `va_list` 作为最后参数时提取为 `unsafe fn`，Rust 绑定追加 `...`（P3 已实现） |
| 链式类型别名（`using B = A; using A = T<...>`） | ✅ 已支持 | — | ✅（hicc 支持） | AliasRegistry 传递性闭合解析（P1 已实现） |
| 方法模板（类内函数模板） | ❌ 跳过 | — | ❌ | `HiccLimitation` |
| `dynamic_cast` | ✅ 骨架自动生成 | `free/dynamic_casts.rs` | ✅（hicc 支持） | 识别继承关系，在 `free/dynamic_casts.rs` 输出注释掉的 `@dynamic_cast` 绑定骨架供用户解注释使用（P3 已实现） |
| 类成员变量 / 静态变量 | ✅ 自动提取（非静态字段） | `method/mtd_*.rs` | ✅ | `#[cpp(field=...)]` 生成 `get_<name>` / `get_<name>_mut` 访问器（P2 已实现） |
| placement new（Rust 内存构造 C++ 对象） | ✅ 已实现 | `free/placement_new.rs` | ✅（hicc 支持） | 识别构造函数签名，在 `free/placement_new.rs` 输出注释掉的 `@placement_new` 绑定骨架供用户解注释使用（P4 已实现） |
| C++ 容器存储 Rust 数据（RustAny） | ✅ 已实现 | `types/mod.rs` + 接口报告 | ✅（hicc 支持） | 识别 STL 容器实例化类型（`std::vector<T>` 等），在 `types/mod.rs` 末尾和接口报告中生成 `hicc::RustAny<T>` 类型映射建议（P4 已实现） |
| `hicc::cpp!` 灵活适配 | ❌ 未生成 | — | ✅（hicc 支持） | 工具不自动生成，需手写 |

---

# 三、cpp2rust-demo 可支持但现阶段未支持的特性 —— 改进方案

> 以下为 **hicc 本身支持** 但 **cpp2rust-demo 工具层尚未处理** 的特性（可在工具侧落地，无需改动 hicc）。

| C++ 特性 | 当前状态 | 分类 | 改进方案 | 实现入口 | 优先级 |
|---------|---------|:----:|---------|---------|:------:|
| **模板类（无别名）** | 跳过（`tool_conservative`）；接口报告和 `suggest-aliases` 子命令自动输出 `using` 别名建议 ✅ 已实现 | ToolConservative | 新增 `suggest-aliases` 子命令；在接口报告中自动输出 `using Alias = FullType<...>;` 建议；用户补充后重跑解锁 | `ast.rs` `SkippedDecl.suggested_alias` + `codegen.rs` 报告渲染 + `main.rs` 新子命令 | P1 ✅ |
| **链式类型别名** (`using B = A`) | ✅ 已支持；AliasRegistry 传递性闭合解析 | ToolLimit | AliasRegistry 增加传递性解析（transitive closure），收集完毕后迭代闭合直到稳定 | `ast.rs` `AliasRegistry::resolve_transitive()` + `is_alias_of_template()` + `is_supported_cpp_type()` | P1 ✅ |
| **`std::function` / lambda 参数** | 跳过（无生成） | ToolLimit | AST 中识别 `std::function<R(Args)>` 类型，生成对应虚函数接口 + `@make_proxy` 绑定骨架建议到接口报告 | `ast.rs` 类型识别 + `codegen.rs` 报告输出 | P2 ✅ |
| **类成员变量 / 静态变量** | 未提取 | ToolLimit | AST 中提取 `FieldDecl` / `VarDecl`（static），生成 `#[cpp(field)]` / `#[cpp(data)]` 绑定到 `free/` 或 `method/` | `ast.rs` 新增 `FieldIR` + `codegen.rs` render | P2 ✅ |
| **`std::string` 参数/返回（shim 建议）** | 跳过（`hicc_limitation`） | ToolConservative | 在接口报告和 `operator_shims.hpp` 中自动生成可复制的 C++ shim 函数原型（`static inline const char* foo_shim(...)`） | `ast.rs` `SkippedDecl.suggested_shim` + `codegen.rs` | P2 ✅ |
| **多重继承（全部 public 基类）** | ✅ 已实现：`ClassIR.bases` 为 `Vec<String>`，所有 public 基类均提取，`render_import_class()` 以 `, ` 分隔列出 | ToolLimit | `ClassIR.bases` 改为 `Vec<String>` 存全部 public 基类，`render_import_class()` 生成 `class C: A, B`（hicc 多重继承不支持，骨架仍有参考价值） | `ast.rs` `ClassIR` + `codegen.rs` | P3 ✅ |
| **虚继承检测与提示** | ✅ 已实现：`BaseSpecifier.is_virtual` 跳过虚基类，接口报告列出警告 | ToolLimit | `BaseSpecifier` 增加 `is_virtual: bool`，跳过虚基类并在接口报告中列出 `Virtual bases (skipped)` | `ast.rs` `BaseSpecifier` + `codegen.rs` | P3 ✅ |
| **函数指针参数（接口建议）** | ✅ 已实现：识别含 `(*)` 的类型，分类为 `ToolConservative`，在接口报告中生成虚函数接口骨架 + `@make_proxy` 调用示例 | ToolConservative | 识别含 `(*)` 的类型，在接口报告中生成对应纯虚接口类模板 + `@make_proxy` 调用示例 | `ast.rs` skip 分支 + `codegen.rs` | P3 ✅ |
| **`dynamic_cast` 绑定** | ✅ 已实现：识别继承关系中可做 downcast 的类对，在 `free/dynamic_casts.rs` 生成注释掉的 `@dynamic_cast` 绑定骨架 | ToolLimit | 识别继承关系中可做 downcast 的类对，在 `free/` 生成 `@dynamic_cast` 绑定骨架 | `ast.rs` 继承链分析 + `codegen.rs` | P3 ✅ |
| **`va_list` / variadic 函数** | ✅ 已实现：识别 `va_list` 最后参数，提取为 `unsafe fn foo(fixed_params, ...) -> T` 绑定；`is_variadic = true` 标记在 `FunctionIR` | ToolConservative | 识别 `va_list` 最后参数，生成对应 `unsafe fn foo(name: &T, ...)` 绑定（hicc 支持，参数/返回无类类型限制需校验） | `ast.rs` 参数类型识别 + `codegen.rs` | P3 ✅ |
| **`--dry-run` 模式** | 不支持 | ToolLimit | `init` 子命令增加 `--dry-run` flag，执行编译和 AST 但不写 `rust/src/`，仅打印接口报告到 stdout | `main.rs` CLI + init 主流程 | P2 ✅ |
| **placement new 绑定** | ✅ 已实现：识别有构造函数的非抽象类，在 `free/placement_new.rs` 生成注释掉的 `@placement_new` 绑定骨架 | ToolLimit | 识别构造函数签名，在 codegen 阶段对需要 placement new 场景生成对应 Rust 接口骨架 | `ast.rs` + `codegen.rs` | P4 ✅ |
| **C++ 容器存储 Rust 数据（RustAny 模板）** | ✅ 已实现：识别 STL 容器类型，在 `types/mod.rs` 末尾和接口报告中生成 `hicc::RustAny<T>` 使用建议 | ToolLimit | 识别 STL 容器实例化类型，在 `types/` 中生成 `hicc::RustAny<T>` 类型映射建议 | `ast.rs` + `codegen.rs` | P4 ✅ |

---

**总结关键结论：**

- **hicc** 功能完整的 C++ FFI 框架，几乎覆盖所有常见 C++ 特性（含模板类、虚函数、STL 容器、RustAny 等），核心不支持项仅有：多重继承、虚继承、运算符重、析构函数显式绑定、函数指针参数、纯 `...` variadic（含类类型时）
- **cpp2rust-demo** 是 hicc 的 AST 驱动脚手架生成器，当前已覆盖最主要的使用场景（自由函数、类方法、虚函数、继承、枚举、别名解锁模板），大量"不支持"项是**工具层面未实现**（hicc 本身支持），改进空间充足且明确
- 优先级最高的改进是 **模板别名建议（§1）** 和 **链式别名传递性解析（§3）**，因为这两项直接影响模板密集型 C++ 库（如 RapidJSON）的提取覆盖率

**批次一改进状态（已完成）：**

| 改进项 | 状态 | 说明 |
|-------|:----:|------|
| P1 链式类型别名传递性解析 | ✅ 已实现 | `AliasRegistry::resolve_transitive()` + `is_alias_of_template()`；`is_supported_cpp_type()` 识别传递性别名 |
| P1 模板别名建议（`suggest-aliases` 子命令） | ✅ 已实现 | 新增 `suggest-aliases` CLI 子命令；`SkippedDecl.suggested_alias`；接口报告显示 `using` 建议代码块 |
| P3 虚继承检测与提示 | ✅ 已实现 | `BaseSpecifier.is_virtual`；虚基类被跳过；接口报告显示 `⚠️ Virtual bases (skipped)` 警告 |

**批次二改进状态（P2，已完成）：**

| 改进项 | 状态 | 说明 |
|-------|:----:|------|
| P2 类实例字段提取（`FieldDecl`） | ✅ 已实现 | 新增 `FieldIR` 结构体；`extract_field()` 从 `FieldDecl` AST 节点提取；`render_import_class()` 生成 `#[cpp(field = "...")]` 读写访问器；接口报告显示 `Instance Fields` 表格 |
| P2 `std::string` shim 建议 | ✅ 已实现 | 新增 `SkippedDecl.suggested_shim`；`generate_unsupported_type_shim()` 对 `std::string` 参数/返回生成 `const char*` shim 原型；接口报告显示 `Shim Suggestions` 章节；同步写入 `operator_shims.hpp` |
| P2 `std::function` 接口建议 | ✅ 已实现 | 同 `suggested_shim` 机制；`is_std_function_type()` 检测；接口报告生成虚函数接口骨架 + `@make_proxy` 使用提示 |
| P2 `--dry-run` 模式 | ✅ 已实现 | `InitArgs` 新增 `--dry-run` 标志；启用时跳过所有 `rust/src/` 写入，接口报告打印到 stdout；AST JSON 仍保存供调试 |

**批次三改进状态（P3，已完成）：**

| 改进项 | 状态 | 说明 |
|-------|:----:|------|
| P3 多重继承（全部 public 基类） | ✅ 已实现 | `ClassIR.bases: Vec<String>` 存储所有 public 基类；`render_import_class()` 以 `, ` 分隔列出（hicc 不支持多重继承，骨架仅作参考） |
| P3 函数指针参数（接口建议） | ✅ 已实现 | `is_function_pointer_type()` 检测含 `(*)` 类型；`categorize_unsupported_type()` 分类为 `ToolConservative`；`generate_unsupported_type_shim()` 生成虚函数接口骨架（`FooHandler`）+ `@make_proxy` 使用提示；接口报告显示 `Shim Suggestions` |
| P3 `@dynamic_cast` 绑定骨架 | ✅ 已实现 | `render_dynamic_casts_module()` 遍历有基类的类，在 `free/dynamic_casts.rs` 输出注释掉的 `@dynamic_cast<Derived>(Base *)` 绑定供用户按需解注释；`free/mod.rs` 自动注册 `dynamic_casts` 子模块 |
| P3 `va_list` / variadic 函数 | ✅ 已实现 | `is_va_list_type()` 检测 `va_list` / `__va_list_tag *` 等变体；`FunctionIR.is_variadic: bool` 标记；`extract_function()` 检测最后参数为 `va_list` 时跳过该参数并置 `is_variadic = true`；`render_free_function_with_name()` / `render_method()` 生成 `unsafe fn foo(fixed_params, ...) -> T` 绑定；接口报告增加 `Variadic Functions` 和 `@dynamic_cast Skeletons` 章节 |

**批次四改进状态（P4，已完成）：**

| 改进项 | 状态 | 说明 |
|-------|:----:|------|
| P4.1 placement new 绑定骨架 | ✅ 已实现 | 新增 `render_placement_new_module()`；对每个有提取到构造函数（`CtorIR`）的非抽象类，在 `free/placement_new.rs` 输出注释掉的 `@placement_new<ClassName>(args...)` 绑定骨架，包含 `hicc::AlignedStorage<T>` 内存参数和生命周期关联返回值（`-> &'a mut T`）；`free/mod.rs` 自动注册 `placement_new` 子模块；`build.rs` 源列表同步更新；接口报告增加 `Placement-New Skeletons (P4)` 章节列出所有可 placement new 的类及构造函数签名 |
| P4.2 STL 容器 `hicc::RustAny` 类型映射建议 | ✅ 已实现 | 新增 `SkippedDecl.stl_container_type: Option<String>` 字段；`is_stl_container_type()` 检测 15 种标准容器（`std::vector` / `std::map` / `std::set` 等）；`find_stl_container_type()` 在 `extract_function()` 跳过函数时提取首个 STL 容器类型；新增 `render_rust_any_suggestions()` 去重后输出 `hicc_std::Vector<T>` / `hicc_std::Map<T>` 等建议注释；`render_types_module()` 末尾追加建议块；接口报告增加 `hicc::RustAny Suggestions for STL Containers (P4)` 章节 |
