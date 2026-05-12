# cpp2rust-demo 示例目录

本目录包含基于真实 C++ 场景的 `cpp2rust-demo` 用例，覆盖从最简单的自由函数绑定到模板密集型库（RapidJSON）的完整使用链路。

每个示例均包含：
- 完整的 C++ 源文件（`.hpp` / `.cpp`）
- `README.md`（背景说明 → C++ 源码 → init/merge 命令 → 预期生成产物 → 场景解析 → 限制说明）

---

## 基础示例

这两个示例覆盖 cpp2rust-demo 的核心用法，是理解 RapidJSON 场景示例的前提。

| 示例 | C++ 特性 | 文档 |
|------|---------|------|
| [`simple/`](simple/README.md) | 自由函数、命名空间、函数重载 | [simple/README.md](simple/README.md) |
| [`class/`](class/README.md) | 类与方法、构造函数、virtual/纯虚、public 继承、运算符 shim | [class/README.md](class/README.md) |

---

## RapidJSON 场景示例

以下 8 个示例基于 [RapidJSON](https://github.com/Tencent/rapidjson)（MIT 协议，header-only C++ JSON 库），
系统覆盖 hicc 绑定体系中的所有主要场景。

每个示例都使用自包含的 C++ 源文件（不需要安装 RapidJSON），
只有 [`rapidjson-08-multi-tu/`](rapidjson-08-multi-tu/README.md) 需要真实的 RapidJSON 头文件。

| 示例 | 核心 C++ 特性 | 对应 hicc 能力 | 关键产物 |
|------|-------------|---------------|---------|
| [`rapidjson-01-enum/`](rapidjson-01-enum/README.md) | `enum` / `enum class` | `EnumIR` → `#[repr(C)] enum` | `types/mod.rs` |
| [`rapidjson-02-typedef-alias/`](rapidjson-02-typedef-alias/README.md) | `typedef` / `using` 别名 | AliasRegistry 两张映射表 | `types/mod.rs` + 接口报告 |
| [`rapidjson-03-template-class/`](rapidjson-03-template-class/README.md) | 模板特化 + typedef 解锁 | `ClassTemplateSpecializationDecl` → `ClassIR` | `method/mtd_*.rs` |
| [`rapidjson-04-abstract-interface/`](rapidjson-04-abstract-interface/README.md) | 全纯虚类 | `#[interface]` + `@make_proxy` | `method/mtd_*.rs` + `free/fn_*.rs` |
| [`rapidjson-05-virtual-methods/`](rapidjson-05-virtual-methods/README.md) | 非纯虚方法 | vtable 透明调用 | `method/mtd_*.rs` |
| [`rapidjson-06-inheritance/`](rapidjson-06-inheritance/README.md) | public 继承链 | `class Derived: Base` 语法 | `method/mtd_*.rs` |
| [`rapidjson-07-operator-shim/`](rapidjson-07-operator-shim/README.md) | 运算符重载 | `operator_shims.hpp` 三步工作流 | `free/shim_ops.rs` + `meta/operator_shims.hpp` |
| [`rapidjson-08-multi-tu/`](rapidjson-08-multi-tu/README.md) | 多编译单元 + header-only | `--no-link` + `merge` 全流程 | `merged_ffi.rs` |

---

## 快速阅读路径

**只想了解基本用法**（自由函数/类方法绑定）：
> `simple/` → `class/`

**只想了解模板类绑定（RapidJSON 式）**：
> `rapidjson-02-typedef-alias/` → `rapidjson-03-template-class/`

**只想了解虚函数/接口/继承**：
> `rapidjson-05-virtual-methods/` → `rapidjson-04-abstract-interface/` → `rapidjson-06-inheritance/`

**只想了解运算符重载 shim**：
> `rapidjson-07-operator-shim/`

**完整 RapidJSON 多 TU 流程**：
> `rapidjson-08-multi-tu/`（依赖 01~07 的知识背景）

---

## 能力速查

| hicc 语法 | 对应示例 |
|-----------|---------|
| `import_lib!` + 自由函数 | `simple/`, `rapidjson-01-enum/` |
| `import_class!` + 实例方法 | `class/`, `rapidjson-05-virtual-methods/` |
| `#[interface]` 纯虚接口 | `class/`, `rapidjson-04-abstract-interface/` |
| `ctor = "..."` 构造函数 | `class/`, `rapidjson-03-template-class/` |
| `class Foo: Base` 继承 | `class/`, `rapidjson-06-inheritance/` |
| `@make_proxy` 反向绑定 | `rapidjson-04-abstract-interface/` |
| `#[repr(C)] enum` 枚举 | `rapidjson-01-enum/` |
| AliasRegistry 模板别名 | `rapidjson-02-typedef-alias/`, `rapidjson-03-template-class/` |
| `operator_shims.hpp` shim | `rapidjson-07-operator-shim/` |
| `--no-link` header-only | `rapidjson-08-multi-tu/` |
| `merge` 多 TU 合并 | `rapidjson-08-multi-tu/` |

---

## 不支持特性说明

以下特性当前不被提取，工具会在 `meta/init-interface-report.md` 中记录跳过原因。

### ❌ hicc 限制（HiccLimitation）

以下特性是 **hicc 本身的能力边界**，无法通过调整 cpp2rust-demo 来绕过；需要手写 C++ shim：

| C++ 特性 | 跳过原因 | 建议方案 |
|---------|---------|---------|
| 析构函数 | hicc 不支持显式析构绑定语法 | 由 C++ RAII / 对象生命周期管理；无需 Rust 侧显式析构 |
| `std::string` 参数/返回 | hicc 无 `std::string` ABI 支持 | 手写 C++ shim，将结果转为 `const char*` 或通过输出参数传出 |
| `std::function` / lambda 参数 | 无法映射到 Rust 闭包 | 封装为虚函数接口 + `@make_proxy` 反向绑定（见 `rapidjson-04-abstract-interface`） |
| Variadic 函数（`...`） | hicc 不支持可变参数 | 手写固定参数 C++ 包装函数 |
| `auto`/`decltype` 返回类型 | 无法在 hicc 签名中表达 | 手写包装函数，显式写出返回类型 |
| 函数指针参数 | Rust 函数指针 ABI 与 C++ 不兼容 | 封装为接口类 + `@make_proxy` |
| 右值引用参数（`T&&`，非 move ctor） | hicc 不支持 `&&` 语义 | 手写接受 `const T&` 的 shim |
| 方法模板（类内函数模板） | 无法生成通用 Rust 泛型方法 | 针对具体实例化写独立 shim 函数 |
| 友元函数 | `FriendDecl` AST 提取不可靠 | 以普通自由函数形式重写 shim |

### ⚠️ 工具条件限制（ToolConservative）

以下特性在**满足特定条件**后可自动解锁；不满足时跳过并标记 `tool_conservative`：

| C++ 特性 | 默认状态 | 解锁方式 |
|---------|---------|---------|
| 模板类（无 typedef/using 别名） | 跳过 | 在 entry.cpp 添加 `typedef`/`using` 别名，触发 AliasRegistry 注册 |
| `std::` 容器参数（无别名） | 跳过 | 为容器类型添加 `using` 别名 |
| 函数模板（无显式特化） | 跳过 | 在 AST 中提供 concrete specialization 可见 |
| 运算符重载 | 跳过（生成 shim starter） | 补全 `operator_shims.hpp` + 在 `hicc::cpp!` 中引入（见 `rapidjson-07-operator-shim`） |

### ⛔ 工具层面限制（ToolLimit，可改进）

以下特性是 **cpp2rust-demo 当前实现的技术限制**（与 hicc 无关），原则上可以在工具侧解决，详见 `docs/future-plan.md`：

| C++ 特性 | 当前行为 | 计划改进 |
|---------|---------|---------|
| 多重继承（`class C: public A, public B`） | 只处理首个 public 基类 `A`，`B` 被忽略 | `future-plan.md §2` |
| 链式类型别名（`using B = A; using A = Foo<T>;`） | AliasRegistry 不追踪两层别名，`B` 无法解锁模板 | `future-plan.md §3` |
| Virtual 继承（菱形继承） | Virtual 基类被跳过，链路不完整 | `future-plan.md §4` |

---

## 相关文档

- [docs/design.md](../docs/design.md) — 架构设计、v1 能力边界、完整能力矩阵、AliasRegistry 指南
- [docs/cpp-features.md](../docs/cpp-features.md) — 完整 C++ 特性支持状态表
- [docs/future-plan.md](../docs/future-plan.md) — 可落地的工具改进计划
- [docs/rapidjson-support.md](../docs/rapidjson-support.md) — RapidJSON 完整验证文档
- [docs/hicc-usage.md](../docs/hicc-usage.md) — hicc 语法参考
- [docs/clang-ast.md](../docs/clang-ast.md) — Clang AST JSON 格式参考
