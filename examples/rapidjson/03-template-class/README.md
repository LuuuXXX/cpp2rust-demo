# 场景 03：模板特化类提取（ClassTemplateSpecializationDecl → ClassIR）

本示例演示 cpp2rust-demo 如何将 `typedef` 别名解锁的模板特化类
提取为完整的 `import_class!` 绑定，包括：
- `canonical_name`（Rust struct 使用别名名）
- `ctor`（构造函数参数）
- 实例方法列表

**依赖**：本示例需要 Phase 1 的 `bare_template_name()` 修复，才能正确提取命名空间限定的模板类型。

---

## 背景

RapidJSON 的 `GenericDocument<UTF8<char>>` 通过 `typedef Document = GenericDocument<UTF8<char>>` 暴露为稳定的类型名。
本示例用自包含的等价实现演示同一提取流程，展示：

1. 模板特化如何通过 `ClassTemplateSpecializationDecl` 进入 AST
2. `canonical_name` 机制如何将 Rust struct 命名为 `Document`（而非冗长的模板完整名）
3. 继承自模板基类（`GenericValue`）的解析

---

## C++ 源码结构（`entry.cpp`）

```cpp
namespace rjdoc {
    // 支撑类型（policy）
    template <typename C = char> struct UTF8 { typedef C Ch; };
    struct CrtAllocator {};
    template <typename B = CrtAllocator> struct MemoryPoolAllocator {};

    // 核心模板类
    template <typename Enc, typename Alloc>
    class GenericValue {
    public:
        bool isNull() const; bool isInt() const;
        int getInt() const;  const char* getString() const;
        void setBool(bool); void setInt(int);
        // ...
    };
    typedef GenericValue<UTF8<char>> Value;   // ← 别名

    template <typename Enc, typename Alloc, typename SAlloc>
    class GenericDocument : public GenericValue<Enc, Alloc> {
    public:
        GenericDocument();
        bool parse(const char* json);
        bool hasParseError() const;
        bool hasMember(const char* name) const;
        Value& getMember(const char* name);
        int size() const;
        // ...
    };
    typedef GenericDocument<UTF8<char>> Document; // ← 别名
}
```

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature rj03 --link rapidjson --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/03-template-class/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj03

# 第 3 步：查看产物
cat .cpp2rust/rj03/rust/src/lib.rs
cat .cpp2rust/rj03/meta/init-interface-report.md
```

---

## 预期生成产物

### `entry.rs`（节选）

```rust
// Value class (extracted via GenericValue<UTF8<char>> alias)
hicc::import_class! {
    #[cpp(class = "rjdoc::GenericValue<rjdoc::UTF8<char>, rjdoc::MemoryPoolAllocator<rjdoc::CrtAllocator>>",
          ctor = "Value()")]
    class Value {
        #[cpp(method = "bool isNull() const")]
        fn is_null(&self) -> bool;

        #[cpp(method = "bool isBool() const")]
        fn is_bool(&self) -> bool;

        #[cpp(method = "bool isInt() const")]
        fn is_int(&self) -> bool;

        #[cpp(method = "int getInt() const")]
        fn get_int(&self) -> i32;

        #[cpp(method = "const char * getString() const")]
        fn get_string(&self) -> *const i8;

        #[cpp(method = "void setBool(bool)")]
        fn set_bool(&mut self, b: bool);

        #[cpp(method = "void setInt(int)")]
        fn set_int(&mut self, n: i32);
    }
}

// Document class (extends Value via GenericDocument<UTF8<char>> alias)
hicc::import_class! {
    #[cpp(class = "rjdoc::GenericDocument<rjdoc::UTF8<char>, rjdoc::MemoryPoolAllocator<rjdoc::CrtAllocator>, rjdoc::CrtAllocator>",
          ctor = "Document()")]
    class Document: Value {
        #[cpp(method = "bool parse(const char *)")]
        fn parse(&mut self, json: *const i8) -> bool;

        #[cpp(method = "bool hasParseError() const")]
        fn has_parse_error(&self) -> bool;

        #[cpp(method = "bool hasMember(const char *) const")]
        fn has_member(&self, name: *const i8) -> bool;

        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;
    }
}
```

### 接口报告中的 `canonical_name` 节

```markdown
## Class `Document` (template specialisation of `rjdoc::GenericDocument<...>`)

_Template specialisation of `rjdoc::GenericDocument<rjdoc::UTF8<char>, ...>`._

### Constructors

| C++ signature | hicc role |
|...| Document() | primary (`ctor = "..."`) |
```

---

## 场景解析

### 1. `canonical_name` 机制

当 cpp2rust-demo 提取 `GenericDocument<UTF8<char>, ...>` 的 `ClassTemplateSpecializationDecl` 时：

1. 查询 `AliasRegistry.alias_for_template("GenericDocument")` → `"Document"`
2. 设置 `ClassIR.canonical_name = Some("Document")`
3. `render_import_class()` 使用 `canonical_name` 作为 Rust struct 名（`class Document {`）
4. `#[cpp(class = "...")]` 属性保留完整的 C++ 限定名，用于 hicc 内部查找

```
C++ 完整名: rjdoc::GenericDocument<rjdoc::UTF8<char>, rjdoc::MemoryPoolAllocator<...>, rjdoc::CrtAllocator>
Rust 生成:  class Document: Value { ... }   ← 使用别名
#[cpp(class=...)] 属性: 完整限定名           ← hicc 用于 C++ 侧查找
```

### 2. 继承链的解析

`GenericDocument` 继承自 `GenericValue<Encoding, Allocator>`（基类 `bases` 数组）：

1. `extract_class_body()` 读取 `node.bases`
2. 基类类型为 `rjdoc::GenericValue<rjdoc::UTF8<char>, rjdoc::MemoryPoolAllocator<...>>`
3. 调用 `bare_template_name()` → `"GenericValue"`
4. 查询 `alias_for_template("GenericValue")` → `"Value"`
5. `class Document: Value` ✅

若没有 Phase 1 的修复，步骤 3 会错误地返回 `"MemoryPoolAllocator>"` → 步骤 4 查询失败 → 继承关系丢失。

### 3. 方法参数类型门

方法 `Value& getMember(const char*)` 的参数返回类型 `rjdoc::GenericValue<...>&`：

1. 类型含 `<` → 进入模板路径
2. `bare_template_name("rjdoc::GenericValue<rjdoc::UTF8<char>, ...>")` → `"GenericValue"`
3. `has_template_alias("GenericValue")` → `true`（因为 `typedef Value = GenericValue<...>`）
4. 类型门**放行** → 方法被提取 ✅

### 4. `setString(const char*, unsigned)` 被跳过

`setString` 的第二个参数类型为 `unsigned`（即 `unsigned int`）。
当前类型映射中 `unsigned` 对应 Rust 的 `u32`（通过 `is_primitive_cpp_type`）。
如果 clang 展开为 `unsigned int`，也能通过类型门。如展开为其他形式，可能出现 `unsupported_type` 跳过。

---

## 限制说明

| 限制 | 说明 |
|------|------|
| `operator[]` 跳过 | 运算符重载不提取，见 `rapidjson/07-operator-shim/` |
| 继承的方法不重复提取 | 若父类 `Value` 已提取，`Document` 中的同名 override 会以 `Document` 自身方法列表为准 |
| 多个模板参数的特化 | 只有 typedef 覆盖的特化被提取；其他参数组合的特化仍跳过 |
| `friend` 函数 | AST 中为 `FriendDecl`，当前跳过 |
