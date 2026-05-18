# ✅ RapidJSON 场景示例

本目录下的 8 个示例基于 [RapidJSON](https://github.com/Tencent/rapidjson)（MIT 协议，header-only C++ JSON 库），
系统覆盖 cpp2rust-demo 中 **✅ 完全自动**（零人工介入）特性的所有主要场景。

每个示例都使用自包含的 C++ 源文件，**无需安装 RapidJSON**（`08-multi-tu/` 除外）。

---

## 示例列表

| 示例 | 核心 C++ 特性 | 对应 hicc 能力 | 关键产物 |
|------|-------------|---------------|---------|
| [`01-enum/`](01-enum/README.md) | `enum` / `enum class` | `EnumIR` → `#[repr(C)] enum` | `types/mod.rs` |
| [`02-typedef-alias/`](02-typedef-alias/README.md) | `typedef` / `using` 别名 | AliasRegistry 两张映射表 | `types/mod.rs` + 接口报告 |
| [`03-template-class/`](03-template-class/README.md) | 模板特化 + typedef 解锁 | `ClassTemplateSpecializationDecl` → `ClassIR` | `method/mtd_*.rs` |
| [`04-abstract-interface/`](04-abstract-interface/README.md) | 全纯虚类 | `#[interface]` + `@make_proxy` | `method/mtd_*.rs` + `free/fn_*.rs` |
| [`05-virtual-methods/`](05-virtual-methods/README.md) | 非纯虚方法 | vtable 透明调用 | `method/mtd_*.rs` |
| [`06-inheritance/`](06-inheritance/README.md) | public 继承链 | `class Derived: Base` 语法 | `method/mtd_*.rs` |
| [`07-operator-shim/`](07-operator-shim/README.md) | 运算符重载 | `operator_shims.hpp` 三步工作流 | `free/shim_ops.rs` + `meta/operator_shims.hpp` |
| [`08-multi-tu/`](08-multi-tu/README.md) | 多编译单元 + header-only | `--no-link` + `merge` 全流程 | `lib.rs` |

---

## 快速阅读路径

**了解枚举/别名/模板类**：
> `01-enum/` → `02-typedef-alias/` → `03-template-class/`

**了解虚函数/接口/继承**：
> `05-virtual-methods/` → `04-abstract-interface/` → `06-inheritance/`

**了解运算符重载 shim（🔧 引导支持）**：
> `07-operator-shim/`

**完整多 TU 流程**：
> `08-multi-tu/`（依赖 01~07 的知识背景）

---

## 注意事项

- `07-operator-shim/` 属于 **🔧 引导支持**（用户需填写 shim 函数体），其余均为 **✅ 完全自动**。
- `08-multi-tu/` 需要本地安装 RapidJSON 头文件（`/tmp/rapidjson`），其余示例完全自包含。
