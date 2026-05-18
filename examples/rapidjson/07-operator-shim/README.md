# 场景 07：运算符重载 → `operator_shims.hpp` 全自动生成

本示例演示 cpp2rust-demo 处理 **C++ 运算符重载**的完整工作流：
工具全自动生成 `operator_shims.hpp` C++ shim 函数体、`entry.rs` 中的激活 `import_lib!` 绑定、
`hicc::cpp!` include 块以及 build.rs include 路径——**用户无需任何手动操作**。

---

## 背景

RapidJSON 的 `GenericValue` 大量使用运算符：
```cpp
Value doc["key"];        // operator[]
Value a = b;             // operator=
if (val) { ... }         // operator bool
if (val1 == val2) { ... } // operator==
```

hicc 本身不支持直接绑定 C++ 运算符（运算符名不是合法的 Rust 函数名）。
cpp2rust-demo 的解决方案是自动生成**具名 C++ 包装函数**（operator shims）：

```cpp
// 自动生成：
JsonValue& json_value_get_at(JsonValue& self, const char* key) { return self[key]; }
```

再通过 `#[cpp(func = "...")]` 绑定为普通 Rust 函数。

---

## C++ 源码（`value_with_ops.hpp`）

```cpp
class JsonValue {
public:
    explicit JsonValue(int type = 0);

    // 普通方法 — 直接提取
    int  GetType() const;
    bool IsNull() const;
    int  GetInt() const;
    void SetInt(int v);

    // 运算符 — 跳过提取，但生成 shim starter
    JsonValue& operator=(const JsonValue& rhs);
    JsonValue& operator[](const char* key);
    const JsonValue& operator[](const char* key) const;  // const 重载
    explicit operator bool() const;
    bool operator==(const JsonValue& rhs) const;
    bool operator!=(const JsonValue& rhs) const;
};
```

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI（工具全自动生成 operator_shims.hpp 和激活的 import_lib! 绑定）
cpp2rust-demo init --feature rj07 --link jsonvalue \
    -- clang -x c++ -fsyntax-only examples/rapidjson/07-operator-shim/entry.cpp

# 查看全自动生成的结果
cat .cpp2rust/rj07/meta/operator_shims.hpp   # C++ shim 函数体（完整实现）
cat .cpp2rust/rj07/rust/src/entry.rs         # 激活的 import_lib! 绑定 + 自动插入的 hicc::cpp! include

# 第 2 步：合并
cpp2rust-demo merge --feature rj07
```

---

## 预期生成产物

### 自动生成：`meta/operator_shims.hpp`（完整 C++ shim 函数体）

```cpp
// Auto-generated operator shims by cpp2rust-demo.
// Include this file in your hicc::cpp! block, then bind
// the functions below via #[cpp(func = "...")] in import_lib!.
#pragma once
#ifndef OPERATOR_SHIMS_HPP
#define OPERATOR_SHIMS_HPP

#include "entry.cpp"
static inline JsonValue& json_value_assign(JsonValue& self, const JsonValue& rhs) {
    return self = rhs;
}

/// Shim for `operator[]`
static inline JsonValue& json_value_get_at(JsonValue& self, const char* key) {
    return self[key];
}

/// Shim for `operator[]` (const)
static inline const JsonValue& json_value_get_at_const(const JsonValue& self, const char* key) {
    return self[key];
}

/// Shim for `operator bool`
static inline bool json_value_to_bool(const JsonValue& self) {
    return static_cast<bool>(self);
}

/// Shim for `operator==`
static inline bool json_value_eq(const JsonValue& self, const JsonValue& rhs) {
    return self == rhs;
}

/// Shim for `operator!=`
static inline bool json_value_ne(const JsonValue& self, const JsonValue& rhs) {
    return self != rhs;
}

#endif // OPERATOR_SHIMS_HPP
```

### 自动生成：`entry.rs` 中的 operator shim 绑定（节选）

绑定为**激活状态**，无需手动解注释：

```rust
// 自动插入的 hicc::cpp! include（meta/ 目录已自动添加到 build.rs include 路径）
hicc::cpp! {
    #include "operator_shims.hpp"
}

// --- operator shims ---
// Auto-generated operator shim Rust bindings by cpp2rust-demo.

hicc::import_lib! {
    #![link_name = "jsonvalue"]

    // Shim for `operator=`
    #[cpp(func = "JsonValue & json_value_assign(JsonValue &, const JsonValue &)")]
    fn json_value_assign(this_val: &mut JsonValue, rhs: &JsonValue) -> &mut JsonValue;

    // Shim for `operator[]`
    #[cpp(func = "JsonValue & json_value_get_at(JsonValue &, const char *)")]
    fn json_value_get_at(this_val: &mut JsonValue, key: *const i8) -> &mut JsonValue;

    // Shim for `operator[]` (const)
    #[cpp(func = "const JsonValue & json_value_get_at_const(const JsonValue &, const char *)")]
    fn json_value_get_at_const(this_val: &JsonValue, key: *const i8) -> &JsonValue;

    // Shim for `operator bool`
    #[cpp(func = "bool json_value_to_bool(const JsonValue &)")]
    fn json_value_to_bool(this_val: &JsonValue) -> bool;

    // Shim for `operator==`
    #[cpp(func = "bool json_value_eq(const JsonValue &, const JsonValue &)")]
    fn json_value_eq(this_val: &JsonValue, rhs: &JsonValue) -> bool;

    // Shim for `operator!=`
    #[cpp(func = "bool json_value_ne(const JsonValue &, const JsonValue &)")]
    fn json_value_ne(this_val: &JsonValue, rhs: &JsonValue) -> bool;
}
```

### 普通方法仍然提取：`entry.rs`（节选）

```rust
hicc::import_class! {
    #[cpp(class = "JsonValue", ctor = "JsonValue(int)")]
    class JsonValue {
        #[cpp(method = "int GetType() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "bool IsNull() const")]
        fn is_null(&self) -> bool;

        #[cpp(method = "int GetInt() const")]
        fn get_int(&self) -> i32;

        #[cpp(method = "void SetInt(int)")]
        fn set_int(&mut self, v: i32);
    }
}
```

### 接口报告中的 Skipped / Shim 节

```markdown
## Operator Overload Shim Hints

The following C++ operators were skipped. hicc does not support operator overloads
directly.  Auto-generated C++ shims have been written to `meta/operator_shims.hpp`.

| Skipped operator | Suggested shim name |
|-----------------|---------------------|
| `JsonValue::operator=`  | `json_value_assign` |
| `JsonValue::operator[]` | `json_value_get_at` |
| `JsonValue::operator bool` | `json_value_to_bool` |
| `JsonValue::operator==` | `json_value_eq` |
| `JsonValue::operator!=` | `json_value_ne` |
```

---

## 场景解析

### 全自动工作流详解

**工具自动完成的所有步骤（`cpp2rust-demo init` 一次完成）**

`extract_class_body()` 遇到 `is_operator_name() == true` 的方法时：
1. 跳过提取（不进入 `FunctionIR` / `import_class!`）
2. 创建 `OperatorShimIR`（记录运算符名、参数类型、返回类型、class 归属）
3. 进入 `ExtractedDecls.operator_shims`

init 结束后，工具自动完成：
- `render_operator_shims_hpp()` → `meta/operator_shims.hpp`（**完整** C++ shim 函数体，直接可用）
- `render_flat_module()` → `<stem>.rs` 中自动插入 `hicc::cpp! { #include "operator_shims.hpp" }`
- `render_operator_shims_rs()` → `<stem>.rs` 末尾**激活的** `import_lib!` 绑定（无注释）
- `build.rs` → meta/ 目录自动添加到 C++ include 路径（`operator_shims.hpp` 可被编译器找到）

**用户无需任何手动操作。**

### 运算符命名规则

工具按以下规则生成 shim 函数名：

| C++ 运算符 | 生成的 shim 名 |
|-----------|--------------|
| `operator=` | `<class>_assign` |
| `operator[]` | `<class>_get_at` |
| `operator bool` | `<class>_to_bool` |
| `operator==` | `<class>_eq` |
| `operator!=` | `<class>_ne` |
| `operator<` | `<class>_lt` |
| `operator+` | `<class>_add` |
| `operator()` | `<class>_call` |

其中 `<class>` 为类名的 snake_case 形式（`JsonValue` → `json_value`）。

### 哪些 operator 可以直接用自动 shim

| 运算符类型 | 直接可用 | 注意事项 |
|-----------|--------|---------|
| 比较（`==`, `!=`, `<`, `>`）| ✅ | 直接可用 |
| 算术（`+`, `-`, `*`, `/`）| ✅ | 直接可用 |
| 赋值（`=`, `+=`）| ✅ | 注意返回类型 `&mut` |
| 下标（`[]`）| ⚠️ | const/非 const 两个版本需分开 |
| 类型转换（`operator bool`）| ✅ | `static_cast<bool>(self)` |
| 函数调用（`()`）| ⚠️ | 可能需要手工处理参数列表 |
| 流操作（`<<`, `>>`）| ⚠️ | 参数是 `std::ostream&`，通常需手工改写 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| const/非 const 重载 | `operator[]` 的两个版本由工具分别处理，shim 名后缀 `_const` 区分 |
| 返回 `&mut Self` 类型 | hicc/Rust 中返回自引用需谨慎，可能需要调整为返回 `*mut` 或 `()` |
| `std::ostream&` 参数 | 映射为 `*mut` 指针，需在 Rust 侧管理 C++ iostream 对象生命周期 |
| 模板运算符 | 模板类的 `operator` 仍跳过，需配合 typedef 别名解锁 |
