# ⚠️ 条件支持示例 01：模板类（无 typedef/using 别名）

**汇总统计类别：⚠️ 条件支持**（满足前置条件后重跑工具可完全自动化）

---

## 背景

C++ 模板类在 clang AST 中以 `ClassTemplateDecl` 表示，没有具体的类型信息。
cpp2rust-demo 只提取**具体特化版本**（`ClassTemplateSpecializationDecl`），
而具体特化需通过 `typedef`/`using` 别名来触发实例化并在 AST 中可见。

**不满足条件时**：工具跳过 `Stack<T>`，在接口报告中记录 `tool_conservative`。  
**满足条件后**：在 `entry.cpp` 中添加 `using IntStack = Stack<int>;`，重跑 `init` 即可自动提取。

---

## C++ 源码（`stack.hpp` / `entry.cpp`）

```cpp
// stack.hpp — 无别名，默认被跳过
template<typename T>
class Stack {
public:
    Stack();
    void push(T value);
    T    pop();
    T    top() const;
    bool empty() const;
    int  size() const;
    void clear();
};
```

```cpp
// entry.cpp — STEP A：无别名（Stack<T> 被跳过）
#include "stack.hpp"

// ── [UNLOCK] 解注释以下别名后重跑 init ────────────────────────────────
// using IntStack    = Stack<int>;
// using DoubleStack = Stack<double>;
// ─────────────────────────────────────────────────────────────────────
```

---

## 运行步骤

### STEP A：无别名（观察跳过行为）

```bash
# 生成 FFI
cpp2rust-demo init --feature cond01 --link stack \
    -- clang -x c++ -fsyntax-only examples/conditional/01-template-no-alias/entry.cpp

# 查看接口报告（Stack<T> 被标记为 tool_conservative）
cat .cpp2rust/cond01/meta/init-interface-report.md
```

接口报告将包含如下条目：

```
## Skipped Declarations

| Name    | Category        | Reason                                      |
|---------|-----------------|---------------------------------------------|
| Stack   | ToolConservative | 模板类无 typedef/using 别名，无法确定具体类型 |
```

同时，工具会输出 `suggest-aliases` 建议：

```bash
cpp2rust-demo suggest-aliases --feature cond01
# 输出示例：
# 建议在 entry.cpp 中添加：
#   using IntStack = Stack<int>;    // 若已有 Stack<int> 使用场景
```

### STEP B：添加别名（解锁自动提取）

编辑 `entry.cpp`，解注释 `using` 别名行：

```cpp
using IntStack    = Stack<int>;
using DoubleStack = Stack<double>;
```

然后重新运行：

```bash
cpp2rust-demo init --feature cond01 --link stack \
    -- clang -x c++ -fsyntax-only examples/conditional/01-template-no-alias/entry.cpp

cpp2rust-demo merge --feature cond01

cat .cpp2rust/cond01/rust/src/merged_ffi.rs
```

---

## 预期生成产物（STEP B 后）

### `types/mod.rs`（别名映射）

```rust
// Stack<int> → IntStack 别名注册
pub type IntStack    = Stack_i32;
pub type DoubleStack = Stack_f64;
```

### `method/mtd_entry.rs`（具体特化类绑定）

```rust
// IntStack = Stack<int>
hicc::import_class! {
    class Stack_i32 {
        ctor = "Stack<int>()"

        #[cpp(method = "void push(int)")]
        fn push(&mut self, value: i32);

        #[cpp(method = "int pop()")]
        fn pop(&mut self) -> i32;

        #[cpp(method = "int top() const")]
        fn top(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

// DoubleStack = Stack<double> — 类似结构，类型替换为 f64
```

---

## 转换流程手册

```
C++ 模板类（无别名）
    │  clang AST 解析
    ▼
ClassTemplateDecl（仅模板定义，无特化节点）
    │  cpp2rust-demo AliasRegistry 检查
    ▼
无匹配别名 → SkipCategory::ToolConservative
    │  接口报告
    ▼
meta/init-interface-report.md（记录跳过原因）

────── 用户添加 using IntStack = Stack<int>; 后重跑 ──────

    │  clang AST 解析（含别名）
    ▼
ClassTemplateSpecializationDecl（Stack<int>）
    + TypedefDecl / TypeAliasDecl（IntStack = Stack<int>）
    │  AliasRegistry 注册
    ▼
bare_template_name("Stack") → alias "IntStack"
    │  ClassIR 提取
    ▼
ClassIR { name: "Stack_i32", canonical_name: Some("IntStack"), ... }
    │  codegen
    ▼
types/mod.rs       ─── pub type IntStack = Stack_i32;
method/mtd_*.rs    ─── import_class! { class Stack_i32 { ... } }
```

---

## 场景解析

| 状态 | AliasRegistry | AST 节点 | 结果 |
|------|--------------|---------|------|
| 无别名 | 空 | ClassTemplateDecl only | 跳过，ToolConservative |
| 有别名 | `Stack → IntStack` | ClassTemplateSpecializationDecl | 自动提取 Stack_i32 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 嵌套模板参数 | `Stack<std::vector<int>>` 仍需 `using` 别名且内层类型也要已知 |
| 非类型模板参数 | `template<int N> class Buf` 不支持（工具跳过） |
| 部分特化 | 部分特化版本不提取，只提取完全特化 |
