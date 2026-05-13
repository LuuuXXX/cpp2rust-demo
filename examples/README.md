# cpp2rust-demo 示例目录

本目录包含基于真实 C++ 场景的 `cpp2rust-demo` 用例，按照**汇总统计类别**（来自 `docs/特性支持全景图.md`）组织，
覆盖从最简单的自由函数绑定到复杂的引导支持场景的完整使用链路。

每个示例均包含：
- 完整的 C++ 源文件（`.hpp` / `.cpp`）
- `README.md`（背景说明 → C++ 源码 → init/merge 命令 → 预期生成产物 → 转换流程手册 → 场景解析 → 限制说明）

---

## 目录结构（按汇总统计类别）

```
examples/
├── simple/           ✅ 基础：自由函数、命名空间、函数重载
├── class/            ✅ 基础：类与方法、构造函数、virtual/纯虚、继承
├── rapidjson/        ✅ RapidJSON 场景：枚举、别名、模板类、接口、虚方法、继承、运算符 shim、多 TU
├── semi-auto/        ⚙️ 半自动：dynamic_cast、placement new
├── conditional/      ⚠️ 条件支持：模板类无别名、函数模板无显式特化
└── guided/           🔧 引导支持：std::string、std::function、函数指针
```

---

## ✅ 基础示例（完全自动）

这两个示例覆盖 cpp2rust-demo 的核心用法，是理解其他示例的前提。

| 示例 | C++ 特性 | 文档 |
|------|---------|------|
| [`simple/`](simple/README.md) | 自由函数、命名空间、函数重载 | [simple/README.md](simple/README.md) |
| [`class/`](class/README.md) | 类与方法、构造函数、virtual/纯虚、public 继承 | [class/README.md](class/README.md) |

---

## ✅ RapidJSON 场景示例（完全自动）

8 个基于 RapidJSON 的系统性场景，涵盖 hicc 绑定体系所有主要能力。
详见 [`rapidjson/README.md`](rapidjson/README.md)。

| 示例 | 核心 C++ 特性 | 关键产物 |
|------|-------------|---------|
| [`rapidjson/01-enum/`](rapidjson/01-enum/README.md) | `enum` / `enum class` | `types/mod.rs` |
| [`rapidjson/02-typedef-alias/`](rapidjson/02-typedef-alias/README.md) | `typedef` / `using` 别名 | `types/mod.rs` |
| [`rapidjson/03-template-class/`](rapidjson/03-template-class/README.md) | 模板特化 + typedef 解锁 | `method/mtd_*.rs` |
| [`rapidjson/04-abstract-interface/`](rapidjson/04-abstract-interface/README.md) | 全纯虚类 + `@make_proxy` | `method/mtd_*.rs` + `free/fn_*.rs` |
| [`rapidjson/05-virtual-methods/`](rapidjson/05-virtual-methods/README.md) | 非纯虚方法 | `method/mtd_*.rs` |
| [`rapidjson/06-inheritance/`](rapidjson/06-inheritance/README.md) | public 继承链 | `method/mtd_*.rs` |
| [`rapidjson/07-operator-shim/`](rapidjson/07-operator-shim/README.md) | 运算符重载 shim（🔧 引导） | `free/shim_ops.rs` + `meta/operator_shims.hpp` |
| [`rapidjson/08-multi-tu/`](rapidjson/08-multi-tu/README.md) | 多编译单元 + `--no-link` | `merged_ffi.rs` |

---

## ⚙️ 半自动示例

工具生成注释骨架，用户解注释或加 flag 后即可完全自动化。
详见 [`semi-auto/README.md`](semi-auto/README.md)。

| 示例 | C++ 特性 | 解锁方式 |
|------|---------|---------|
| [`semi-auto/01-dynamic-cast/`](semi-auto/01-dynamic-cast/README.md) | `dynamic_cast` 向下转型 | 解注释或加 `--enable-dynamic-cast` |
| [`semi-auto/02-placement-new/`](semi-auto/02-placement-new/README.md) | Placement New | 解注释或加 `--enable-placement-new` |

---

## ⚠️ 条件支持示例

满足前置条件（添加别名/显式实例化）后重跑工具即可完全自动化。
详见 [`conditional/README.md`](conditional/README.md)。

| 示例 | C++ 特性 | 解锁前置条件 |
|------|---------|------------|
| [`conditional/01-template-no-alias/`](conditional/01-template-no-alias/README.md) | 模板类（无别名） | 添加 `using Alias = Template<Args>;` |
| [`conditional/02-function-template/`](conditional/02-function-template/README.md) | 函数模板（无显式特化） | 添加 `template int foo<int>(...);` |

---

## 🔧 引导支持示例

工具生成签名正确的 C++ shim 原型/接口骨架，用户必须填写函数体或实现 trait。
详见 [`guided/README.md`](guided/README.md)。

| 示例 | C++ 特性 | 用户需做什么 |
|------|---------|------------|
| [`guided/01-std-string/`](guided/01-std-string/README.md) | `std::string` 参数/返回 | 填写 `operator_shims.hpp` 中的 C-string shim 函数体 |
| [`guided/02-std-function/`](guided/02-std-function/README.md) | `std::function`/lambda 参数 | 手写纯虚接口类 + `impl XxxInterface for MyStruct` |
| [`guided/03-function-pointer/`](guided/03-function-pointer/README.md) | 函数指针参数 | 手写纯虚接口类 + `impl XxxInterface for MyStruct` |

---

## 快速阅读路径

**只想了解基本用法**（自由函数/类方法绑定）：
> `simple/` → `class/`

**只想了解模板类绑定**：
> `rapidjson/02-typedef-alias/` → `rapidjson/03-template-class/` → `conditional/01-template-no-alias/`

**只想了解虚函数/接口/继承**：
> `rapidjson/05-virtual-methods/` → `rapidjson/04-abstract-interface/` → `rapidjson/06-inheritance/`

**只想了解 `std::function` / 函数指针回调**：
> `guided/02-std-function/` → `guided/03-function-pointer/`

**只想了解半自动特性**（dynamic_cast / placement new）：
> `semi-auto/01-dynamic-cast/` → `semi-auto/02-placement-new/`

**完整 RapidJSON 多 TU 流程**：
> `rapidjson/08-multi-tu/`（依赖 01~07 的知识背景）

---

## hicc 语法速查

| hicc 语法 | 对应示例 |
|-----------|---------|
| `import_lib!` + 自由函数 | `simple/`, `rapidjson/01-enum/` |
| `import_class!` + 实例方法 | `class/`, `rapidjson/05-virtual-methods/` |
| `#[interface]` 纯虚接口 | `class/`, `rapidjson/04-abstract-interface/` |
| `ctor = "..."` 构造函数 | `class/`, `rapidjson/03-template-class/` |
| `class Foo: Base` 继承 | `class/`, `rapidjson/06-inheritance/` |
| `@make_proxy` 反向绑定 | `rapidjson/04-abstract-interface/`, `guided/02-std-function/` |
| `#[repr(C)] enum` 枚举 | `rapidjson/01-enum/` |
| AliasRegistry 模板别名 | `rapidjson/02-typedef-alias/`, `rapidjson/03-template-class/` |
| `operator_shims.hpp` shim | `rapidjson/07-operator-shim/`, `guided/01-std-string/` |
| `--no-link` header-only | `rapidjson/08-multi-tu/` |
| `merge` 多 TU 合并 | `rapidjson/08-multi-tu/` |
| `dynamic_cast!` 向下转型 | `semi-auto/01-dynamic-cast/` |
| `@placement_new` | `semi-auto/02-placement-new/` |

---

## 不支持特性说明

以下特性当前不被提取，工具会在 `meta/init-interface-report.md` 中记录跳过原因。

### ❌ hicc 限制（HiccLimitation）

以下特性是 **hicc 本身的能力边界**，无法通过调整 cpp2rust-demo 来绕过；需要手写 C++ shim：

| C++ 特性 | 跳过原因 | 建议方案 |
|---------|---------|---------|
| 析构函数 | hicc 不支持显式析构绑定语法 | 由 C++ RAII / 对象生命周期管理；无需 Rust 侧显式析构 |
| 运算符重载 | hicc 不支持运算符名称作为绑定符号；工具自动生成 `operator_shims.hpp` starter | 补全 `operator_shims.hpp` + 在 `hicc::cpp!` 中引入（见 `rapidjson/07-operator-shim/`） |
| `std::string` 参数/返回 | hicc 无 `std::string` ABI 支持 | 见 `guided/01-std-string/` |
| `std::function` / lambda 参数 | 无法映射到 Rust 闭包 | 见 `guided/02-std-function/` |
| Variadic 函数（`...`） | hicc 不支持可变参数 | 手写固定参数 C++ 包装函数 |
| `auto`/`decltype` 返回类型 | 无法在 hicc 签名中表达 | 手写包装函数，显式写出返回类型 |
| 函数指针参数 | Rust 函数指针 ABI 与 C++ 不兼容 | 见 `guided/03-function-pointer/` |
| 方法模板（类内函数模板） | 无法生成通用 Rust 泛型方法 | 针对具体实例化写独立 shim 函数 |
| 友元函数 | `FriendDecl` AST 提取不可靠 | 以普通自由函数形式重写 shim |

### ⚠️ 工具条件限制（ToolConservative）

满足条件后可自动解锁，见 `conditional/` 目录：

| C++ 特性 | 默认状态 | 解锁方式 |
|---------|---------|---------|
| 模板类（无 typedef/using 别名） | 跳过 | 见 `conditional/01-template-no-alias/` |
| `std::` 容器参数（无别名） | 跳过 | 为容器类型添加 `using` 别名 |
| 函数模板（无显式特化） | 跳过 | 见 `conditional/02-function-template/` |

### ⛔ 工具层面限制（ToolLimit，可改进）

| C++ 特性 | 当前行为 | 计划改进 |
|---------|---------|---------|
| 多重继承 | 只处理首个 public 基类 | `docs/future-plan.md §2` |
| 链式类型别名 | AliasRegistry 不追踪两层别名 | `docs/future-plan.md §3` |
| 虚继承（菱形继承） | Virtual 基类被跳过 | `docs/future-plan.md §4` |

---

## 相关文档

- [docs/特性支持全景图.md](../docs/特性支持全景图.md) — 完整特性支持全景表与汇总统计
- [docs/design.md](../docs/design.md) — 架构设计、v1 能力边界、AliasRegistry 指南
- [docs/cpp-features.md](../docs/cpp-features.md) — C++ 特性支持状态表（含示例链接）
- [docs/future-plan.md](../docs/future-plan.md) — 可落地的工具改进计划
- [docs/rapidjson-support.md](../docs/rapidjson-support.md) — RapidJSON 完整验证文档
- [docs/hicc-usage.md](../docs/hicc-usage.md) — hicc 语法参考
- [docs/clang-ast.md](../docs/clang-ast.md) — Clang AST JSON 格式参考
