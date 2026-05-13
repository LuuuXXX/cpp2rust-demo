# ⚠️ 条件支持示例 02：函数模板（无显式特化）

**汇总统计类别：⚠️ 条件支持**（满足前置条件后重跑工具可完全自动化）

---

## 背景

C++ 函数模板在 AST 中以 `FunctionTemplateDecl` 表示，没有具体的参数类型。
cpp2rust-demo 只提取 **显式特化版本**（`FunctionDecl` with template specialization），
需要在编译单元中添加显式实例化声明来让 clang 生成 specialization 节点。

**不满足条件时**：工具跳过 `clamp<T>` 等，在接口报告中记录 `tool_conservative`。  
**满足条件后**：添加 `template int clamp<int>(int, int, int);`，重跑 `init` 即可自动提取。

---

## C++ 源码（`algorithms.hpp` / `entry.cpp`）

```cpp
// algorithms.hpp — 无显式特化，默认被跳过
template<typename T>
T clamp(T val, T lo, T hi);

template<typename T>
void swap_values(T& a, T& b);

template<typename T>
T max_of(T a, T b);

template<typename T>
T lerp(T a, T b, double t);
```

```cpp
// entry.cpp — STEP A：无显式实例化（函数模板被跳过）
#include "algorithms.hpp"

// ── [UNLOCK] 解注释以下显式实例化声明后重跑 init ──────────────────────
// template int    clamp<int>(int, int, int);
// template void   swap_values<int>(int&, int&);
// template int    max_of<int>(int, int);
// template double lerp<double>(double, double, double);
// ─────────────────────────────────────────────────────────────────────
```

---

## 运行步骤

### STEP A：无显式特化（观察跳过行为）

```bash
cpp2rust-demo init --feature cond02 --link algorithms \
    -- clang -x c++ -fsyntax-only examples/conditional/02-function-template/entry.cpp

# 查看接口报告
cat .cpp2rust/cond02/meta/init-interface-report.md
```

接口报告示例：

```
## Skipped Declarations

| Name         | Category        | Reason                                            |
|--------------|-----------------|---------------------------------------------------|
| clamp        | ToolConservative | 函数模板无显式特化，无法确定参数类型               |
| swap_values  | ToolConservative | 函数模板无显式特化                                |
| max_of       | ToolConservative | 函数模板无显式特化                                |
| lerp         | ToolConservative | 函数模板无显式特化                                |
```

### STEP B：添加显式实例化（解锁自动提取）

编辑 `entry.cpp`，解注释显式实例化声明：

```cpp
template int    clamp<int>(int, int, int);
template void   swap_values<int>(int&, int&);
template int    max_of<int>(int, int);
template double lerp<double>(double, double, double);
```

然后重新运行：

```bash
cpp2rust-demo init --feature cond02 --link algorithms \
    -- clang -x c++ -fsyntax-only examples/conditional/02-function-template/entry.cpp

cpp2rust-demo merge --feature cond02
cat .cpp2rust/cond02/rust/src/merged_ffi.rs
```

---

## 预期生成产物（STEP B 后）

### `free/fn_entry.rs`（具体特化函数绑定）

```rust
hicc::import_lib! {
    #![link_name = "algorithms"]

    // clamp<int>
    #[cpp(func = "int clamp(int, int, int)")]
    fn clamp(val: i32, lo: i32, hi: i32) -> i32;

    // swap_values<int>
    #[cpp(func = "void swap_values(int &, int &)")]
    fn swap_values(a: &mut i32, b: &mut i32);

    // max_of<int>
    #[cpp(func = "int max_of(int, int)")]
    fn max_of(a: i32, b: i32) -> i32;

    // lerp<double>
    #[cpp(func = "double lerp(double, double, double)")]
    fn lerp(a: f64, b: f64, t: f64) -> f64;
}
```

---

## 转换流程手册

```
C++ 函数模板（无显式特化）
    │  clang AST 解析
    ▼
FunctionTemplateDecl（仅模板定义，无 specialization 子节点）
    │  cpp2rust-demo 类型门检查
    ▼
无 specialization → SkipCategory::ToolConservative
    │  接口报告
    ▼
meta/init-interface-report.md（记录跳过原因）

────── 用户添加 template int clamp<int>(int, int, int); 后重跑 ──────

    │  clang AST 解析（含显式实例化）
    ▼
FunctionDecl（clamp, template_specialization = true）
    │  is_supported_cpp_type 放行（参数类型均为 POD）
    ▼
FunctionIR { name: "clamp", params: [(i32, i32, i32)], ret: i32 }
    │  codegen
    ▼
free/fn_entry.rs ─── import_lib! { fn clamp(...) -> i32; }
```

---

## 场景解析

| 状态 | AST 节点 | 结果 |
|------|---------|------|
| 无显式实例化 | FunctionTemplateDecl only | 跳过，ToolConservative |
| 有 `template int clamp<int>(...)` | FunctionDecl (specialization) | 自动提取为 `fn clamp` |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 每个参数类型组合需单独实例化 | `clamp<int>` 和 `clamp<double>` 需分别添加显式实例化 |
| 非 POD 模板参数 | 若特化参数类型不被 hicc 支持（如 `std::string`），仍会跳过 |
| 可变参数模板 | `template<typename... Args>` 不支持，需手写固定参数包装 |
