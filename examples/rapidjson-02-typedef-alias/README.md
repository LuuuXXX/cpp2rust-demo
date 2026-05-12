# 场景 02：typedef/using 别名解锁模板提取（AliasRegistry）

本示例演示 cpp2rust-demo 的 **AliasRegistry** 机制——
通过 `typedef`/`using` 别名声明，将命名空间限定的模板特化类型注册为可识别别名，
进而解锁模板类的 FFI 提取和方法参数类型门。

这是 RapidJSON 绑定的核心前置条件，也是 `rapidjson-03` 的基础。

---

## 背景

RapidJSON 的 `document.h` 中有大量这样的定义：

```cpp
// rapidjson/document.h（简化）
namespace rapidjson {
    template <typename Encoding, typename Allocator, typename StackAllocator>
    class GenericDocument : public GenericValue<Encoding, Allocator> { ... };

    typedef GenericDocument<UTF8<char>> Document;   // ← 别名
    typedef GenericValue<UTF8<char>>    Value;       // ← 别名
}
```

在没有别名机制之前，cpp2rust-demo 遇到 `GenericDocument<UTF8<char>, ...>` 这类类型时，
会认为它是无法处理的模板类型并跳过。
通过 AliasRegistry，只要 `typedef Document = GenericDocument<...>` 对 clang 可见，
工具就会自动：
1. 将 `GenericDocument` 映射到别名 `Document`
2. 在类型门检查时，对 `GenericDocument<...>` 参数放行
3. 提取的类/方法使用 `Document` 作为 Rust struct 名（`canonical_name`）

---

## C++ 源码结构（`entry.cpp`）

```cpp
namespace rjson {
    // 模板类原型
    template <typename Enc, typename Alloc>
    class GenericValue { ... };

    template <typename Enc, typename Alloc, typename SAlloc>
    class GenericDocument : public GenericValue<Enc, Alloc> { ... };

    // ← 关键：这两行让 AliasRegistry 建立映射
    typedef GenericValue<UTF8<char>>    Value;
    typedef GenericDocument<UTF8<char>> Document;

    // using 语法（C++11）也被收集
    using ScopedDoc = GenericDocument<ASCII<char>>;
}

// 使用别名类型的自由函数 → 类型门通过
const char* valueTypeName(const rjson::Value& val);
bool        documentOk(const rjson::Document& doc);
```

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature rj02 --link rapidjson --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson-02-typedef-alias/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj02

# 第 3 步：查看产物
cat .cpp2rust/rj02/meta/init-interface-report.md   # 接口报告（含 AliasRegistry 效果）
cat .cpp2rust/rj02/rust/src/merged_ffi.rs          # 合并后 FFI
```

---

## 预期生成产物

### 接口报告（`meta/init-interface-report.md`，节选）

```markdown
## Type Aliases

| C++ alias | Underlying C++ type | Rust type |
|-----------|---------------------|-----------|
| `Value`   | `rjson::GenericValue<rjson::UTF8<char>>` | `Value` |
| `Document`| `rjson::GenericDocument<rjson::UTF8<char>>` | `Document` |
| `ScopedDoc`| `rjson::GenericDocument<rjson::ASCII<char>>` | `ScopedDoc` |

## Free Functions

| C++ name | Rust name | Signature |
|----------|-----------|-----------|
| `valueTypeName` | `value_type_name` | `const char * valueTypeName(const rjson::Value &)` |
| `documentOk`    | `document_ok`     | `bool documentOk(const rjson::Document &)` |
```

### `types/mod.rs`（节选）

```rust
// Type alias mappings registered from the AST.
// "GenericValue"    → alias "Value"
// "GenericDocument" → alias "Document"
// "GenericDocument" → alias "ScopedDoc"

pub fn rust_type_for(cpp_type: &str) -> Option<&'static str> {
    match cpp_type {
        "rjson::GenericValue<rjson::UTF8<char>>" => Some("Value"),
        "rjson::GenericDocument<rjson::UTF8<char>>" => Some("Document"),
        "rjson::GenericDocument<rjson::ASCII<char>>" => Some("ScopedDoc"),
        _ => None,
    }
}
```

### `free/fn_entry.rs`（节选）

```rust
hicc::import_lib! {
    #![link_name = "rapidjson"]

    #[cpp(func = "const char * valueTypeName(const rjson::Value &)")]
    fn value_type_name(val: &Value) -> *const i8;

    #[cpp(func = "bool documentOk(const rjson::Document &)")]
    fn document_ok(doc: &Document) -> bool;
}
```

---

## 场景解析

### AliasRegistry 的两张映射表

cpp2rust-demo 在 AST 第一遍扫描时，调用 `AliasRegistry::collect_from_ast()` 收集所有
`TypedefDecl` 和 `TypeAliasDecl` 节点：

```
template_to_alias（裸模板名 → 别名名）:
  "GenericValue"    → "Value"
  "GenericDocument" → "Document"   （第一个注册的别名优先）
  "GenericDocument" → "ScopedDoc"  （已有 Document，忽略）

alias_to_type（别名名 → 完整限定类型）:
  "Value"     → "rjson::GenericValue<rjson::UTF8<char>>"
  "Document"  → "rjson::GenericDocument<rjson::UTF8<char>>"
  "ScopedDoc" → "rjson::GenericDocument<rjson::ASCII<char>>"
```

### `bare_template_name()` 的关键作用

当 clang 提供类型字符串 `"rjson::GenericDocument<rjson::UTF8<char>, rjson::CrtAllocator>"` 时，
AliasRegistry 需要提取裸模板名 `"GenericDocument"` 来建立映射。

**旧实现（有 Bug）**：
```
rsplit("::") → "CrtAllocator>"       ← 错误！取到最后一个 :: 分段
split('<')   → "CrtAllocator>"       ← < 不在这里
bare = "CrtAllocator>"               ← 映射建立失败
```

**新实现（`bare_template_name()`）**：
```
split('<')   → "rjson::GenericDocument"  ← 先剥离模板参数
rsplit("::") → "GenericDocument"         ← 再剥离命名空间
bare = "GenericDocument"                 ← 映射正确建立 ✅
```

这一修复（Phase 1 Bug Fix 1-3）是 RapidJSON 模板类绑定的根本前提。

### 类型门的工作流程

当工具遇到函数 `bool documentOk(const rjson::Document& doc)` 时：

1. 参数类型解析后得到 `"rjson::Document"` 或 `"rjson::GenericDocument<...>"`
2. `is_supported_cpp_type()` 检测到类型含 `<`（模板）
3. 调用 `bare_template_name()` 提取 `"GenericDocument"`
4. 查询 `alias_registry.has_template_alias("GenericDocument")` → `true`
5. 类型门**放行**，函数进入提取结果

若没有 `typedef Document = GenericDocument<...>` 的声明，步骤 4 返回 `false`，
函数会被跳过并在报告中标记为 `tool_conservative`。

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 多个别名取第一个 | 若 `GenericDocument` 有多个 typedef，只有最先出现的别名名被注册为 canonical |
| 别名仅一层 | 不支持链式别名（`using A = B<T>; using C = A;`），`C` 不会映射回模板 |
| 部分特化无法提取 | `template<> class GenericValue<MyEncoding>` 仍需要 typedef 配合 |
| 函数模板 | 函数级模板仍跳过；别名机制仅解锁类模板特化 |
