# 🔧 引导支持示例

**汇总统计类别：🔧 引导支持**（工具生成签名正确的 C++ shim 原型/接口骨架，用户必须填写函数体或实现 trait）

本目录包含 cpp2rust-demo 中需要用户手写业务逻辑的三个特性场景。
工具已经提供完整的签名骨架和详细的操作指引，用户只需填充业务语义部分。

---

## 示例列表

| 示例 | C++ 特性 | 工具生成内容 | 用户需做什么 |
|------|---------|------------|------------|
| [`01-std-string/`](01-std-string/README.md) | `std::string` 参数/返回 | `operator_shims.hpp` C-string shim 签名 + Rust 骨架 | 填写 shim 函数体（缓冲区/生命周期管理） |
| [`02-std-function/`](02-std-function/README.md) | `std::function`/lambda 参数 | 接口报告中的纯虚接口类代码建议 | 手写接口类 + `impl XxxInterface for MyStruct` |
| [`03-function-pointer/`](03-function-pointer/README.md) | 函数指针参数 | 接口报告中的纯虚接口类代码建议 | 手写接口类 + `impl XxxInterface for MyStruct` |

---

## 共同特点

- 工具跳过含上述参数类型的方法，标记为 `hicc_limitation`。
- 工具**生成的骨架签名是正确的**，用户无需猜测类型或调用约定。
- **真正的人工介入边界**是业务语义：缓冲区大小策略、回调逻辑等工具永远无法自动推断。
- 完成引导步骤后，Rust 侧接口与 ✅ 完全自动的绑定使用方式完全一致。
