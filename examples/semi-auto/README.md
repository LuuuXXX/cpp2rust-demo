# ✅ 全自动示例（原半自动）

**汇总统计类别：✅ 完全自动**（工具直接生成活跃绑定，无需用户干预）

本目录包含 cpp2rust-demo 中运算符 / 继承相关的两个特性场景。
工具已经生成完整的活跃绑定，直接可编译使用，无需手工解注释。

---

## 示例列表

| 示例 | C++ 特性 | 自动化方式 |
|------|---------|---------|
| [`01-dynamic-cast/`](01-dynamic-cast/README.md) | `dynamic_cast` 向下转型 | `@dynamic_cast` 绑定自动生成并激活到 `<stem>.rs` |
| [`02-placement-new/`](02-placement-new/README.md) | Placement New（Rust 内存中构造 C++ 对象） | `@placement_new` 绑定自动生成并激活到 `<stem>.rs` |

---

## 共同特点

- 工具默认**不生成**这两类绑定，避免引入非必要的 RTTI 开销或特殊内存管理需求。
- 生成的注释骨架**语法完整**，解注释后直接可编译。
