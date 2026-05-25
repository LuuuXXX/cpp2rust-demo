# 示例 18：const 正确性

## 特性概述

本示例展示 C++ 的 **`const` 正确性（const correctness）**，包括 `const` 成员方法、`const` 引用参数、`const` 指针、`mutable` 成员变量以及 `const` 重载解析。`const` 正确性是 C++ 接口设计的重要原则，hicc 通过 `&self` vs `&mut self` 精确映射。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| `const` 方法 | 承诺不修改对象状态 |
| `const` 重载 | 同名方法的 const/非const 版本 |
| `mutable` 成员 | 即使在 `const` 方法中也可修改 |
| `const T&` 参数 | 只读引用，不拷贝 |
| `const T*` 指针 | 指向只读数据 |
| `const` 返回值 | 限制返回值的使用方式 |

### 代码结构

```cpp
class Buffer {
    char* data;
    mutable size_t access_count;  // 可在 const 方法中修改

    size_t length() const;          // const 方法
    const char* c_str() const;      // const 方法，返回 const 指针
    void clear();                   // 非 const 方法

    // const 重载
    const char& front() const;      // const 版本 → 只读引用
    char& front();                  // 非 const 版本 → 可写引用
};

// const 参数函数
bool equal(const Buffer& a, const Buffer& b);
int compare(const Buffer& a, const Buffer& b);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 属性：

| AST 属性 | 含义 |
|----------|------|
| `CXXMethodDecl.isConst: true` | `const` 成员方法 |
| `FieldDecl.isMutable: true` | `mutable` 成员变量 |
| `qualType: "const char *"` | 返回 const 指针 |
| `qualType: "const char &"` | 返回 const 引用 |
| `qualType: "const Buffer &"` | const 引用参数 |

## hicc 处理方式

### `const` 方法 → `&self`，非 `const` 方法 → `&mut self`

这是 hicc 最核心的 const 映射规则：

```rust
hicc::import_class! {
    #[cpp(class = "Buffer")]
    class Buffer {
        // const 方法 → &self（只读借用）
        #[cpp(method = "size_t length() const")]
        fn length(&self) -> usize;

        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;

        // 非 const 方法 → &mut self（可变借用）
        #[cpp(method = "void clear()")]
        fn clear(&mut self);

        // const char& front() const → 返回只读引用
        #[cpp(method = "const char& front() const")]
        fn front_const(&self) -> &i8;

        // char& front() → 返回可变引用
        #[cpp(method = "char& front()")]
        fn front_mut(&mut self) -> &mut i8;
    }
}
```

### `const` 重载处理

C++ 允许同名方法的 const/非const 重载，Rust 中需要用不同的函数名区分：

```rust
// C++ 有两个 front() 方法：
// const char& front() const;
// char& front();

// Rust 侧用不同名称区分
fn front_const(&self) -> &i8;    // const 版本
fn front_mut(&mut self) -> &mut i8;  // 非 const 版本
```

### `const` 引用参数

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Buffer;

    // const Buffer& → &Buffer（只读引用）
    #[cpp(func = "bool equal(const Buffer&, const Buffer&)")]
    fn buffer_equal(a: &Buffer, b: &Buffer) -> bool;

    #[cpp(func = "int compare(const Buffer&, const Buffer&)")]
    fn buffer_compare(a: &Buffer, b: &Buffer) -> i32;
}
```

### `mutable` 成员的影响

`mutable` 成员允许在 `const` 方法中被修改（常用于缓存、计数器等）。在 Rust 侧，`mutable` 的存在不影响方法映射——方法仍然映射为 `&self`，只是底层 C++ 实现可以修改该字段。

```rust
// C++ 中 length() 修改了 mutable access_count，但仍是 const 方法
// Rust 侧仍然映射为 &self
#[cpp(method = "size_t length() const")]
fn length(&self) -> usize;  // 即使底层修改了 access_count
```

### `const` 返回值处理

| C++ 返回类型 | Rust 返回类型 |
|-------------|--------------|
| `const char*` | `*const i8` |
| `const T&`（C++类） | `ClassRef<'_, T>` |
| `const char&` | `&i8` 或 `*const i8` |
| `T` const 值 | `T`（const 不影响值类型） |

### 使用示例

```rust
fn main() {
    let mut buf = buffer_new(b"hello\0".as_ptr() as _);

    // const 方法可在不可变借用上调用
    let len = buf.length();
    let s = buf.c_str();

    // 非 const 方法需要可变借用
    buf.clear();

    // const 重载：自动根据借用类型选择
    let immutable_ref: &Buffer = &buf;
    // 编译器会使用 const 版本（因为 immutable_ref 不可变）
}
```

## 注意事项

1. **Rust 的 const 语义更强**：Rust 的借用规则比 C++ 的 `const` 更严格——`&T` 真的不能修改任何字段（`mutable` 除外），而 C++ 的 `const` 可以通过 `const_cast` 绕过
2. **`mutable` 映射**：`mutable` 成员在 Rust 侧不直接体现，相关的 `const` 方法仍映射为 `&self`
3. **`const` 重载命名**：Rust 中相同名称的 const/非const 方法必须重命名，通常加 `_const` 或 `_mut` 后缀
4. **`const` 成员函数检测**：cpp2rust-demo 通过 AST 中 `CXXMethodDecl.isConst: true` 自动生成 `&self`
