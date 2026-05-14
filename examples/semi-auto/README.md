# ⚙️ 半自动示例

**汇总统计类别：⚙️ 半自动**（工具生成注释骨架，用户解注释后即可完全自动化）

本目录包含 cpp2rust-demo 中需要少量人工干预的两个特性场景。
工具已经生成完整的绑定骨架，用户只需解注释目标绑定，无需手写任何绑定代码。

---

## 示例列表

| 示例 | C++ 特性 | 解锁方式 |
|------|---------|---------|
| [`01-dynamic-cast/`](01-dynamic-cast/README.md) | `dynamic_cast` 向下转型 | 解注释 `free/dynamic_casts.rs` |
| [`02-placement-new/`](02-placement-new/README.md) | Placement New（Rust 内存中构造 C++ 对象） | 解注释 `free/placement_new.rs` |

---

## 共同特点

- 工具默认**不生成**这两类绑定，避免引入非必要的 RTTI 开销或特殊内存管理需求。
- 生成的注释骨架**语法完整**，解注释后直接可编译。
