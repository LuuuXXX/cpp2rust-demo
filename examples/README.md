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

## 相关文档

- [docs/design.md](../docs/design.md) — 架构设计、v1 能力边界、完整能力矩阵、AliasRegistry 指南
- [docs/hicc-usage.md](../docs/hicc-usage.md) — hicc 语法参考、模板密集型库配置建议
- [docs/clang-ast.md](../docs/clang-ast.md) — Clang AST JSON 格式参考
