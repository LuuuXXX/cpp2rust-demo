# 特性粒度示例（features/）

本目录包含针对 **单一 C++ 特性** 的最小化示例，每个示例仅展示一个特性，
方便用户快速查看：工具如何处理该特性，以及生成的 Rust FFI 代码形式。

所有示例均为 **✅ 完全自动** 特性，无需用户额外操作。

---

## 示例一览

| 目录 | C++ 特性 | 关键生成内容 |
|------|---------|------------|
| [`01-inline-functions/`](01-inline-functions/README.md) | `inline` 函数 | `import_lib!` 中普通绑定（inline 透明） |
| [`02-default-params/`](02-default-params/README.md) | 默认参数 | 完整参数列表提取，默认值忽略 |
| [`03-rvalue-ref/`](03-rvalue-ref/README.md) | `&&` 右值引用方法 | `fn build(self)` 消耗语义 |
| [`04-va-list/`](04-va-list/README.md) | `va_list` 最后参数 | `unsafe fn foo(args, ...)` |
| [`05-global-vars/`](05-global-vars/README.md) | 全局变量 | `#[cpp(data = "...")]` + `&'static` 引用 |
| [`06-static-members/`](06-static-members/README.md) | 静态类数据成员 | `#[cpp(data = "Class::member")]` |
| [`07-instance-fields/`](07-instance-fields/README.md) | 实例字段（FieldDecl） | `#[cpp(field = "Class::field")]` 读写访问器 |

---

## 特性映射速查

| C++ 关键字/语法 | 对应示例 | 生成的 Rust 语法 |
|--------------|---------|--------------|
| `inline` | `01-inline-functions/` | 普通 `fn`（与非 inline 相同） |
| `int foo(int a, bool b = false)` 默认参数 | `02-default-params/` | `fn foo(a: i32, b: bool)`（默认值丢弃） |
| `T method() &&` | `03-rvalue-ref/` | `fn method(self) -> T` |
| `void fn(int, va_list)` | `04-va-list/` | `unsafe fn fn(i32, ...)` |
| `extern int g_var` | `05-global-vars/` | `fn g_var() -> &'static mut i32` |
| `static int Class::count` | `06-static-members/` | `fn count() -> &'static mut i32` |
| `double x` (公有字段) | `07-instance-fields/` | `fn get_x(&self) -> &f64` |

---

## 与其他示例目录的关系

```
examples/
├── simple/         ← 核心：自由函数、命名空间、重载
├── class/          ← 核心：类方法、构造函数、virtual、继承
├── features/       ← 本目录：单特性最小化示例
├── rapidjson/      ← 真实场景：RapidJSON 完整系列
├── semi-auto/      ← 半自动：dynamic_cast、placement new
├── conditional/    ← 条件支持：模板类、函数模板、链式别名
└── guided/         ← 引导支持：std::string、std::function、函数指针
```

如需了解工具全貌，从 `simple/` 和 `class/` 开始，再查看 `features/` 补充细节。
