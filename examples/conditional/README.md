# ⚠️ 条件支持示例

**汇总统计类别：⚠️ 条件支持**（满足特定前置条件后重跑工具可完全自动化）

本目录包含 cpp2rust-demo 中需要用户先修改 C++ 源码、再重跑工具的两个特性场景。
工具本身没有技术限制——只要提供正确的前置条件（别名/显式实例化），工具即可全自动完成提取。

---

## 示例列表

| 示例 | C++ 特性 | 解锁前置条件 |
|------|---------|------------|
| [`01-template-no-alias/`](01-template-no-alias/README.md) | 模板类（无 `typedef`/`using` 别名） | 在 `entry.cpp` 中添加 `using Alias = Template<Args>;` |
| [`02-function-template/`](02-function-template/README.md) | 函数模板（无显式特化） | 在 `entry.cpp` 中添加 `template int foo<int>(...);` 显式实例化 |

---

## 共同特点

- 工具默认**跳过**这两类声明，在接口报告中记录 `tool_conservative`。
- `suggest-aliases` 子命令可帮助发现需要添加的别名。
- 添加前置条件后**无需修改工具调用方式**，直接重跑 `init` 即可自动提取。
