# C++ 特性支持矩阵

本文档给出 `cpp2rust-demo`（配合 `hicc`）对 C++ 语言特性的完整支持状态。

## 标记说明

| 标记 | 含义 |
|------|------|
| ✅ **已支持** | 工具自动提取，无需用户干预 |
| ⚠️ **条件支持（ToolConservative）** | 满足特定条件时自动支持；不满足时跳过并在报告中标记 `tool_conservative`，可通过用户操作解锁 |
| 🔧 **半自动（需用户补全）** | 工具生成 starter 骨架，用户确认/调整后可激活绑定 |
| ❌ **不支持—hicc 限制（HiccLimitation）** | hicc 本身不支持，cpp2rust-demo 跳过并在报告中标记 `hicc_limitation` |
| ⛔ **不支持—工具限制（ToolLimit）** | 当前 cpp2rust-demo 实现层面的技术债或架构限制，与 hicc 无关；原则上可在工具侧解决（见 `future-plan.md`） |

---

## 一、自由函数

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 普通自由函数（非模板） | ✅ | `free/fn_*.rs` | `import_lib!` + `#[cpp(func = "...")]` |
| 命名空间限定函数 | ✅ | `free/fn_*.rs` | 限定名嵌入 `#[cpp(func = "ns::foo(...)")]` |
| 函数重载 | ✅ | `free/fn_*.rs` | 自动追加 `_2`, `_3`, … 后缀 |
| `static` 成员函数 | ✅ | `free/fn_*.rs` | `#[cpp(func = "Class::method(...)")]` |
| 函数模板（无显式特化） | ⚠️ | — | 需 AST 中有 concrete specialization 可见；否则跳过标记 `tool_conservative` |
| Variadic 函数（`...`） | ❌ | — | `hicc_limitation`；跳过，建议手写固定参数 C++ 包装 |
| `auto`/`decltype` 返回类型 | ❌ | — | `hicc_limitation`；跳过 |
| 函数指针参数 | ❌ | — | `hicc_limitation`；含 `(*)` 的参数类型跳过 |
| `std::function` / lambda 参数 | ❌ | — | `hicc_limitation`；建议封装为虚函数接口 + `@make_proxy` |

---

## 二、类与方法

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 普通实例方法（`public`，非模板） | ✅ | `method/mtd_*.rs` | `import_class!` + `#[cpp(method = "...")]` |
| `const` 方法 | ✅ | `method/mtd_*.rs` | 映射为 `fn foo(&self)` |
| 非 `const` 方法 | ✅ | `method/mtd_*.rs` | 映射为 `fn foo(&mut self)` |
| 非纯 `virtual` 方法 | ✅ | `method/mtd_*.rs` | hicc 通过 vtable 透明调用，Rust 侧无感知 |
| 全纯虚类（所有公有方法均 `= 0`） | ✅ | `method/mtd_*.rs` | 生成 `#[interface]` trait + `@make_proxy` 反向绑定 |
| 混合类（部分方法为纯虚） | ✅ | `method/mtd_*.rs` | 普通方法正常提取；纯虚方法生成 companion `#[interface]` |
| 构造函数（主构造函数） | ✅ | `method/mtd_*.rs` | 参数最少的 public ctor → `ctor = "..."` 属性 |
| 额外构造函数（重载） | ✅ | `free/fn_*.rs` | 作为工厂函数进入 `import_lib!` |
| Copy / Move 构造函数 | ✅（自动跳过） | — | 自动识别 `const T&` / `T&&` 签名，跳过 |
| 析构函数 | ❌ | — | `hicc_limitation`；hicc 不支持显式析构绑定；对象生命周期由 C++ 侧管理 |
| 运算符重载 | 🔧 | `free/shim_ops.rs` | 工具自动生成 `operator_shims.hpp` starter 和 Rust 骨架；用户确认实现后激活 |
| `private` / `protected` 成员 | ✅（自动跳过） | — | 设计上自动排除，不进入输出 |
| 友元函数（`friend`） | ❌ | — | AST 提取不可靠（`FriendDecl` 解析受限）；跳过 |
| 方法模板（类内函数模板） | ❌ | — | `hicc_limitation`；无法生成通用 Rust 泛型；跳过 |

---

## 三、继承

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 单层 public 继承 | ✅ | `method/mtd_*.rs` | `class Derived: Base` 语法；支持 upcast |
| public 继承链（多层） | ✅ | `method/mtd_*.rs` | 每一层单独生成 `class X: Y`；链式 upcast 需用户手工处理 |
| 多重继承 | ✅（骨架生成） | `method/mtd_*.rs` | 所有 public 基类均提取，生成 `class C: A, B` 骨架；但 hicc 本身不支持多重继承运行时语义，骨架无法编译，需手写 C++ 委托包装后单继承绑定 |
| `protected` 继承 | ✅（自动跳过） | — | 仅处理 `public` 基类，`protected`/`private` 继承忽略 |
| `virtual` 继承（菱形继承） | ⚠️（跳过并报告） | 接口报告 | 工具自动检测虚基类，跳过并在接口报告中列出 `⚠️ Virtual bases (skipped)` 警告；hicc 不支持虚继承 |

---

## 四、模板

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 模板类特化（有 `typedef`/`using` 别名） | ⚠️ | `method/mtd_*.rs` | AliasRegistry 解锁：裸模板名 → 别名列表（1:N）→ 完整限定类型 |
| 同一模板的多个不同特化（各有独立别名） | ✅ | `method/mtd_*.rs` | `alias_for_type()` 精确匹配完整特化类型，每个特化生成独立 Rust struct（如 `using IntBox = Box<int>; using StrBox = Box<string>;` → `IntBox` 与 `StrBox` 各自独立提取） |
| 模板类（无任何别名） | ⚠️ | — | 跳过，标记 `tool_conservative`；在 entry.cpp 添加别名后可解锁 |
| 多参数模板特化 | ⚠️ | `method/mtd_*.rs` | 仅 typedef 覆盖的特化被提取；其他参数组合的特化仍跳过 |
| 链式别名（`using A = B<T>; using C = A;`） | ✅ | `types/mod.rs` | AliasRegistry 传递性闭合解析已实现；`C` 正确映射回原始模板并解锁模板提取 |
| 函数模板（自由函数/方法级）| ⚠️ | — | 需 AST 中有 concrete specialization 可见（如显式特化 `template<> void foo<int>()`）；否则跳过 |
| 类模板部分特化（`template<typename T> class Foo<T, int>`） | ⚠️ | — | 需 typedef 配合完整特化提取；无别名则跳过；纯部分特化无法独立生成绑定 |
| `std::` 容器参数（无别名） | ⚠️ | — | 无别名时跳过；为容器类型添加 `using` 别名可解锁 |
| 模板运算符 | ❌ | — | 模板类的 operator 仍需 typedef + shim 双重解锁 |

---

## 五、类型系统

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| `enum`（C++03 非 scoped） | ✅ | `types/mod.rs` | `#[repr(C)] pub enum` |
| `enum class`（C++11 scoped） | ✅ | `types/mod.rs` | 同上；Rust 本身已是 scoped |
| `typedef` 别名 | ✅ | `types/mod.rs` | 注册到 AliasRegistry，同时生成类型映射条目 |
| `using` 别名（C++11） | ✅ | `types/mod.rs` | 与 `typedef` 同等对待 |
| 全局变量（命名空间级） | ✅ | `free/fn_*.rs` | `#[cpp(data = "ns::var")]` 绑定 |
| 类静态数据成员 | ✅ | `free/fn_*.rs` | `#[cpp(data = "Class::member")]` 绑定 |
| `const` 变量 | ✅ | `free/fn_*.rs` | 返回 `&'static T`（只读引用） |
| 非 `const` 变量 | ✅ | `free/fn_*.rs` | 返回 `&'static mut T`（可写引用） |
| 基本标量类型（`int`, `bool`, `double` 等） | ✅ | — | 内置映射表覆盖所有 C++ 基本类型 |
| `size_t` / `ptrdiff_t` | ✅ | — | 分别映射为 `usize` / `isize` |
| `void*` / `const void*` | ✅ | — | 映射为 `*mut core::ffi::c_void` / `*const core::ffi::c_void` |
| `const char*` | ✅ | — | 映射为 `*const i8` |
| `std::string` 参数/返回 | ❌ | — | `hicc_limitation`；需 C++ shim 将结果转为 `const char*` 或输出参数 |
| 引用类型（`T&`, `const T&`） | ✅ | — | 映射为 Rust 引用（`&T` / `&mut T`） |
| 右值引用（`T&&`） | ❌ | — | `hicc_limitation`；仅在 move ctor 上自动识别为跳过，其他场景跳过 |
| 匿名 `enum` / 匿名 `struct` | ❌ | — | 无名称，无法生成有意义的 Rust 类型；跳过 |

---

## 六、`@make_proxy` 与接口实现

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 全纯虚类的 `@make_proxy` 绑定 | ✅ | `free/fn_*.rs` | 自动为每个 `#[interface]` 生成对应 `fn new_xxx_proxy(intf: hicc::Interface<Xxx>) -> Xxx` |
| Rust struct 实现 C++ 接口 | ✅ | — | 通过 `impl XxxInterface for MyStruct` + `new_xxx_proxy()` 使用 |
| 混合类的 companion interface | ✅ | `method/mtd_*.rs` | 纯虚方法生成 companion `#[interface]` trait；混合类继承该 trait |

---

## 七、多翻译单元与 merge

| C++ 特性 | 状态 | 输出位置 | 说明 |
|----------|------|---------|------|
| 多编译单元（多次 `init`） | ✅ | `mod_<group>/` | 每次 `init` 累积到同一 feature 目录的不同 `mod_*` 子目录 |
| `--no-link`（header-only 库） | ✅ | `build.rs` | 不向 `build.rs` 注入 `cargo::rustc-link-lib=<name>` |
| `merge` 合并多个分组 | ✅ | `src.2/merged_ffi.rs` | 合并所有 `mod_*/include/free/class/method/types`；生成全局视图 |
| `src → src.2` 符号链接切换 | ✅ | `rust/src` | `build.rs` 引用 `src/...`，merge 后自动指向 `src.2` 产物 |
| 跨 group 重复类型去重 | ✅ | `merged_ffi.rs` | merge 阶段合并相同 include/type 定义，避免重复 |

---

## 八、不支持特性汇总与原因

### 8.1 hicc 限制（HiccLimitation）

以下特性需手写 C++ shim 才能绑定，原因是 hicc 绑定层本身不支持：

| 特性 | 跳过原因 | 建议方案 |
|------|---------|---------|
| 析构函数 | hicc 无析构绑定语法 | 由 C++ 侧/RAII 管理生命周期；若需要通知 Rust，使用普通方法 |
| `std::string` 参数/返回 | hicc 无 `std::string` ABI 支持 | C++ shim：返回 `const char*` 或接受 `const char*` 输入 |
| `std::function` / lambda 参数 | 无法映射到 Rust 闭包 | 封装为虚函数接口，再用 `@make_proxy` 反向绑定 |
| Variadic 函数 (`...`) | hicc 不支持可变参数 | 手写固定参数 C++ 包装函数 |
| `auto`/`decltype` 返回类型 | 无法在 hicc 签名中表达 | 手写包装函数，显式写出返回类型 |
| 函数指针参数 | Rust 函数指针 ABI 与 C++ 不兼容 | 封装为接口 + `@make_proxy` |
| 右值引用（`T&&`，非 move ctor） | hicc 不支持 `&&` 语义 | 手写接受 `const T&` 或按值传递的 shim |
| 方法模板 | 无法在 hicc 中表达泛型方法 | 针对具体实例化写 shim 函数 |
| 友元函数 | AST `FriendDecl` 提取受限 | 将友元函数以普通自由函数形式重写为 shim |

### 8.2 工具层面已解决的历史限制

以下特性曾是 cpp2rust-demo 的工具实现限制，现已全部解决：

| 特性 | 解决方案 |
|------|---------|
| 多重继承 | `ClassIR.bases: Vec<String>` 存储所有 public 基类；`render_import_class()` 生成 `class C: A, B`（hicc 不支持多重继承运行时语义，骨架仅作参考） |
| 链式类型别名 | `AliasRegistry::resolve_transitive()` 实现传递性闭合解析，`using B = A; using A = T<...>` 可正确解锁模板提取 |
| Virtual 继承（菱形继承）检测 | `BaseSpecifier.is_virtual` 自动检测虚基类，跳过后在接口报告中列出 `⚠️ Virtual bases (skipped)` 警告 |

---

## 示例与参考

示例目录按汇总统计类别组织，详见 [`examples/README.md`](../examples/README.md)。

| 特性分类 | 汇总类别 | 对应示例 |
|----------|---------|---------|
| 自由函数 / 重载 | ✅ | `examples/simple/` |
| 类全特性（ctor、virtual、继承、static） | ✅ | `examples/class/` |
| enum / enum class | ✅ | `examples/rapidjson/01-enum/` |
| typedef / using 别名 | ✅ | `examples/rapidjson/02-typedef-alias/` |
| 模板类特化（有别名） | ✅ | `examples/rapidjson/03-template-class/` |
| 全纯虚接口 + @make_proxy | ✅ | `examples/rapidjson/04-abstract-interface/` |
| 非纯虚方法 | ✅ | `examples/rapidjson/05-virtual-methods/` |
| public 继承 | ✅ | `examples/rapidjson/06-inheritance/` |
| 运算符重载 shim | 🔧 | `examples/rapidjson/07-operator-shim/` |
| 多翻译单元 + merge | ✅ | `examples/rapidjson/08-multi-tu/` |
| dynamic_cast 向下转型 | ⚙️ | `examples/semi-auto/01-dynamic-cast/` |
| Placement New | ⚙️ | `examples/semi-auto/02-placement-new/` |
| 模板类（无别名） | ⚠️ | `examples/conditional/01-template-no-alias/` |
| 函数模板（无显式特化） | ⚠️ | `examples/conditional/02-function-template/` |
| std::string 参数/返回 | 🔧 | `examples/guided/01-std-string/` |
| std::function / lambda 参数 | 🔧 | `examples/guided/02-std-function/` |
| 函数指针参数 | 🔧 | `examples/guided/03-function-pointer/` |
