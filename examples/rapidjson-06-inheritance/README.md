# 场景 06：public 继承链 → `class Derived: Base` 语法

本示例演示 cpp2rust-demo 如何处理**公有继承**关系：
从基类抽取 `#[interface]` trait，
从派生类生成 `class Derived: Base` 的 `import_class!` 绑定。

---

## 背景

RapidJSON 的 Writer 体系（`GenericWriter`、`PrettyWriter`）采用继承设计：
`PrettyWriter<StringBuffer>` 继承 `GenericWriter<StringBuffer>`，
扩展了漂亮打印功能。

本示例用自包含的 `WriterBase` / `PrettyWriterImpl` 演示同一模式，
无需安装 RapidJSON。

---

## C++ 源码结构（`writer_base.hpp`）

```cpp
// 全纯虚基类 → #[interface]
class WriterBase {
public:
    explicit WriterBase(int indent = 0);
    virtual ~WriterBase() {}

    virtual void WriteString(const char* str, size_t len) = 0;  // 纯虚
    virtual void WriteInt(int value) = 0;                        // 纯虚
    virtual void WriteBool(bool value) = 0;                      // 纯虚
    virtual void Flush() = 0;                                    // 纯虚

    int GetIndent() const;  // 非虚访问器
};

// 具体派生类，继承 WriterBase
class PrettyWriterImpl : public WriterBase {
public:
    explicit PrettyWriterImpl(int indent = 4);

    // 覆写所有纯虚方法
    void WriteString(const char* str, size_t len) override;
    void WriteInt(int value) override;
    void WriteBool(bool value) override;
    void Flush() override;

    // 额外方法
    void SetMaxDepth(int depth);
    int  GetMaxDepth() const;
    bool IsAtLineStart() const;
};
```

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature rj06 --link writer \
    -- clang -x c++ -fsyntax-only examples/rapidjson-06-inheritance/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj06

# 第 3 步：查看产物
cat .cpp2rust/rj06/rust/src/merged_ffi.rs
```

---

## 预期生成产物

### `method/mtd_entry.rs`（两个类的绑定）

```rust
// 全纯虚基类 → #[interface] trait
hicc::import_class! {
    #[interface]
    class WriterBase {
        #[cpp(method = "void WriteString(const char *, size_t)")]
        fn write_string(&mut self, str_: *const i8, len: usize);

        #[cpp(method = "void WriteInt(int)")]
        fn write_int(&mut self, value: i32);

        #[cpp(method = "void WriteBool(bool)")]
        fn write_bool(&mut self, value: bool);

        #[cpp(method = "void Flush()")]
        fn flush(&mut self);

        // GetIndent 是非虚方法，也被提取
        #[cpp(method = "int GetIndent() const")]
        fn get_indent(&self) -> i32;
    }
}

// 具体派生类，继承 WriterBase
hicc::import_class! {
    #[cpp(class = "PrettyWriterImpl", ctor = "PrettyWriterImpl(int)")]
    class PrettyWriterImpl: WriterBase {
        #[cpp(method = "void WriteString(const char *, size_t)")]
        fn write_string(&mut self, str_: *const i8, len: usize);

        #[cpp(method = "void WriteInt(int)")]
        fn write_int(&mut self, value: i32);

        #[cpp(method = "void WriteBool(bool)")]
        fn write_bool(&mut self, value: bool);

        #[cpp(method = "void Flush()")]
        fn flush(&mut self);

        #[cpp(method = "void SetMaxDepth(int)")]
        fn set_max_depth(&mut self, depth: i32);

        #[cpp(method = "int GetMaxDepth() const")]
        fn get_max_depth(&self) -> i32;

        #[cpp(method = "bool IsAtLineStart() const")]
        fn is_at_line_start(&self) -> bool;
    }
}
```

### `free/fn_entry.rs`（`@make_proxy` + 静态方法入口）

```rust
hicc::import_lib! {
    #![link_name = "writer"]

    class WriterBase;
    class PrettyWriterImpl;

    // @make_proxy for the abstract WriterBase interface
    #[cpp(func = "WriterBase @make_proxy<WriterBase>()")]
    #[interface(name = "WriterBase")]
    fn new_writer_base_proxy(intf: hicc::Interface<WriterBase>) -> WriterBase;
}
```

---

## 场景解析

### 1. `bases` 数组的提取

clang AST JSON 中，基类信息出现在 `CXXRecordDecl` 节点的顶层 `"bases"` 数组，
**不在** `"inner"` 子节点中。`AstNode` 结构体有专门的 `bases: Vec<BaseSpecifier>` 字段
来解析此数组（不能遗漏，否则继承关系丢失）。

提取流程（`extract_class_body()`）：
```
for base in node.bases:
    if base.access == "public":
        bare = strip "class "/"struct " prefix from base.type_info.qual_type
        template_bare = bare_template_name(bare)
        resolved = alias_for_template(template_bare) ?? class_map.get(bare) ?? bare
        class_ir.bases.push(resolved)
```

### 2. 基类类型为全纯虚类时的特殊情况

`WriterBase` 被判定为 `is_abstract = true`，生成 `#[interface]` trait。
`PrettyWriterImpl` 继承自 `WriterBase`，因此：
- `PrettyWriterImpl.bases = ["WriterBase"]`
- 生成 `class PrettyWriterImpl: WriterBase`

hicc 中 `class Derived: Base` 语法的含义：
- 在 C++ 侧，`Derived*` 可 upcast 到 `Base*`
- Rust 端可以将 `PrettyWriterImpl` 实例传给期望 `WriterBase*` 的 C++ 函数

### 3. 派生类方法的提取策略

`PrettyWriterImpl` 覆写了 `WriterBase` 的所有纯虚方法（`override`）。
这些方法在 `PrettyWriterImpl` 的 `CXXMethodDecl` 中以非纯虚形式出现
（`is_pure = false`，`is_virtual = true`），因此走正常提取路径。

此外，`SetMaxDepth`、`GetMaxDepth`、`IsAtLineStart` 是 `PrettyWriterImpl` 独有的方法，
也被正常提取。

### 4. `class WriterBase` 与 `class PrettyWriterImpl` 的分工

| 能力 | 在哪个类上操作 |
|------|--------------|
| 创建实例 | `PrettyWriterImpl::new(indent)` 或 `@make_proxy` via Rust |
| 基类接口调用 | `WriterBase` trait 方法（upcast 后调用）|
| 扩展功能调用 | 仅在 `PrettyWriterImpl` 上可用 |
| 传给 C++ 函数 | 以 `PrettyWriterImpl`（或 `WriterBase` upcast）传递 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 仅首个 public 基类 | `class C: public A, public B` 只处理 `A` |
| 虚析构函数 | 跳过（`HiccLimitation`）|
| `size_t` 参数 | 映射为 `usize` |
| `protected` 继承 | 跳过（仅处理 `public` 基类）|
| 多层继承链 | `A <- B <- C` 时，`C` 的 `class C: B` 出现，`B` 的 `class B: A` 出现，链式 upcast 需用户手工处理 |
