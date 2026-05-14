# ⚠️ 条件支持示例 03：链式类型别名（AliasRegistry 传递性解析）

**汇总统计类别：✅ 完全自动**（提供任意深度的别名链后，工具全自动提取）

---

## 背景

C++ 代码中常出现多级别名：

```cpp
using IntStore = Store<int>;   // 直接别名：Store<int> 的具体化
using MyStore  = IntStore;     // 链式别名：MyStore → IntStore → Store<int>
```

早期版本的 AliasRegistry 只做**单层**映射：`IntStore` 能解锁 `Store<int>` 的提取，但 `MyStore` 无法被追踪回 `Store<int>`，导致模板提取失败。

**当前行为（已解决）**：`AliasRegistry::resolve_transitive()` 在收集完所有别名后执行传递性闭合（transitive closure），使 `MyStore` 也能正确解锁 `Store<int>` 的提取，两种别名层级均 ✅ 完全自动处理。

---

## C++ 源码（`store.hpp` / `entry.cpp`）

```cpp
// store.hpp — 无别名时默认被跳过
template<typename T>
class Store {
public:
    Store();
    void        put(const char* key, T value);
    T           get(const char* key) const;
    bool        has(const char* key) const;
    int         count() const;
    void        clear();
};
```

```cpp
// entry.cpp — 三种状态对比
// STEP A: 无别名（Store<T> 被跳过）
// #include "store.hpp"

// STEP B1: 直接别名（单层，一直可用）
// using IntStore = Store<int>;

// STEP B2: 链式别名（传递性解析，AliasRegistry 已支持）
// using IntStore = Store<int>;
// using MyStore  = IntStore;    ← 链式，MyStore 也会被解锁
```

---

## 运行步骤

### STEP A：无别名（观察跳过行为）

```bash
cpp2rust-demo init --feature cond03 --link store \
    -- clang -x c++ -fsyntax-only examples/conditional/03-chained-alias/entry.cpp

cat .cpp2rust/cond03/meta/init-interface-report.md
```

接口报告将包含：

```
## Skipped Declarations

| Name  | Category         | Reason                                         |
|-------|------------------|------------------------------------------------|
| Store | ToolConservative | 模板类无 typedef/using 别名，无法确定具体类型   |
```

同时可通过 `suggest-aliases` 查看别名建议：

```bash
cpp2rust-demo suggest-aliases --feature cond03
# 示例输出：
# 建议在 entry.cpp 中添加：
#   using IntStore = Store<int>;
```

### STEP B1：直接别名（解锁 IntStore）

编辑 `entry.cpp`，解注释 `using IntStore = Store<int>;`，然后重跑：

```bash
cpp2rust-demo init --feature cond03 --link store \
    -- clang -x c++ -fsyntax-only examples/conditional/03-chained-alias/entry.cpp

cpp2rust-demo merge --feature cond03

cat .cpp2rust/cond03/rust/src/merged_ffi.rs
```

### STEP B2：链式别名（解锁 IntStore + MyStore）

编辑 `entry.cpp`，同时解注释两行 `using`，再重跑（同上命令）。

---

## 预期生成产物（STEP B2 后）

### `types/mod.rs`（别名映射）

```rust
// AliasRegistry 传递性解析：MyStore → IntStore → Store<int>
pub type IntStore = Store_i32;
pub type MyStore  = Store_i32;  // 链式别名，映射到同一底层类型
```

### `method/mtd_entry.rs`（具体特化类绑定）

```rust
// Store<int> 由 IntStore / MyStore 任一别名解锁
hicc::import_class! {
    #[cpp(class = "Store<int>", ctor = "Store<int>()")]
    class Store_i32 {
        #[cpp(method = "void put(const char *, int)")]
        fn put(&mut self, key: *const i8, value: i32);

        #[cpp(method = "int get(const char *) const")]
        fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "bool has(const char *) const")]
        fn has(&self, key: *const i8) -> bool;

        #[cpp(method = "int count() const")]
        fn count(&self) -> i32;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}
```

---

## 转换流程手册

```
C++ 链式别名
    │  clang AST 解析
    ▼
TypeAliasDecl: IntStore = Store<int>
TypeAliasDecl: MyStore  = IntStore
    │  AliasRegistry::collect_from_ast()
    ▼
alias_to_type: { IntStore → Store<int>, MyStore → IntStore }
    │  AliasRegistry::resolve_transitive()（传递性闭合）
    ▼
alias_to_type: { IntStore → Store<int>, MyStore → Store<int> }  ← 传递性补全
template_to_alias: { Store → IntStore (or MyStore) }
    │  is_supported_cpp_type() / extract_class_body()
    ▼
ClassIR { name: "Store_i32", canonical_name: Some("IntStore"), ... }
    │  codegen
    ▼
types/mod.rs    ─── pub type IntStore = Store_i32;
                    pub type MyStore  = Store_i32;
method/mtd_*.rs ─── import_class! { class Store_i32 { ... } }
```

---

## 场景解析

| 状态 | entry.cpp 内容 | AliasRegistry 状态 | 结果 |
|------|--------------|-------------------|------|
| STEP A | 无别名 | 空 | Store 跳过，tool_conservative |
| STEP B1 | `using IntStore = Store<int>` | IntStore → Store<int> | Store_i32 提取，IntStore 别名生成 |
| STEP B2 | + `using MyStore = IntStore` | 传递性闭合后 MyStore → Store<int> | Store_i32 提取，IntStore + MyStore 别名均生成 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 三级及以上链 | `using C = B; using B = A; using A = T<...>` 均支持（算法迭代至稳定点） |
| 跨文件别名 | 别名必须在同一 translation unit（entry.cpp 包含的头文件范围内）可见 |
| 非类型模板参数 | `template<int N>` 类型不支持，只处理类型参数特化 |
