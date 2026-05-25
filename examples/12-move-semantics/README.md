# 示例 12：移动语义

## 特性概述

本示例展示 C++ 的**移动语义**，包括移动构造函数、移动赋值运算符、右值引用（`&&`）以及 `std::move`。移动语义通过转移资源所有权避免不必要的深拷贝，在 Rust 中对应原生的所有权转移（move）。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 右值引用 `T&&` | 绑定临时对象（rvalue） |
| 移动构造 `T(T&&)` | 从临时对象"窃取"资源 |
| 移动赋值 `T& operator=(T&&)` | 赋值时转移资源 |
| `std::move()` | 将左值转换为右值引用 |
| `noexcept` | 移动操作通常应标记为 `noexcept` |

### 代码结构

```cpp
class Buffer {
    char* data;
    size_t size;

    Buffer(Buffer&& other) noexcept;           // 移动构造
    Buffer& operator=(Buffer&& other) noexcept; // 移动赋值
    Buffer(const Buffer& other);                // 拷贝构造（对比）
    Buffer& operator=(const Buffer& other);     // 拷贝赋值（对比）

    char* get_data();
    size_t get_size() const;
};

Buffer combine_buffers(Buffer a, Buffer b);     // 移动参数
Buffer create_buffer(size_t size);              // 返回时移动
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点与属性：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `CXXConstructorDecl.isMoveConstructor: true` | 移动构造函数 |
| `CXXMethodDecl` `operator=` 参数含 `&&` | 移动赋值运算符 |
| `qualType: "Buffer &&"` | 右值引用参数类型 |
| `CXXMethodDecl.isConst: false` + `&&` 限定符 | 右值引用成员函数 |

AST 中右值引用的识别：

```json
{
  "kind": "ParmVarDecl",
  "name": "other",
  "type": { "qualType": "Buffer &&" }
}
```

## hicc 处理方式

### 右值引用参数 → 值传递

C++ 中 `T&&` 参数的语义是"接收并消耗"，在 Rust 中对应值传递（所有权转移）：

| C++ 参数类型 | Rust 参数类型 | 语义 |
|-------------|--------------|------|
| `T&&` | `T`（值） | 所有权转移给被调用方 |
| `const T&` | `&T` | 只读借用 |
| `T&` | `&mut T` | 可变借用 |

```rust
hicc::import_class! {
    #[cpp(class = "Buffer")]
    class Buffer {
        #[cpp(method = "char* get_data()")]
        fn get_data(&mut self) -> *mut i8;

        #[cpp(method = "size_t get_size() const")]
        fn get_size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Buffer;

    // T&& 参数 → 值传递（消耗所有权）
    #[cpp(func = "Buffer combine_buffers(Buffer, Buffer)")]
    fn combine_buffers(a: Buffer, b: Buffer) -> Buffer;

    #[cpp(func = "Buffer create_buffer(size_t)")]
    fn create_buffer(size: usize) -> Buffer;
}
```

### 右值引用成员函数 → `self`（值接收者）

C++ 中以 `&&` 限定的成员函数（rvalue-qualified method）在 Rust 中映射为 `self`（按值接收）：

```cpp
// C++ 右值限定方法
class Buffer {
    Buffer extract() &&;  // 只能在右值上调用
};
```

```rust
hicc::import_class! {
    #[cpp(class = "Buffer")]
    class Buffer {
        // && 方法 → fn xxx(self) ...
        #[cpp(method = "Buffer Buffer::extract() &&")]
        fn extract(self) -> Buffer;
    }
}
```

cpp2rust-demo 通过检测 `qualType` 中末尾的 `&&` 来识别右值限定方法，并生成 `self` 而非 `&self`/`&mut self`。

### 移动语义与 Rust 所有权的对应

Rust 中所有值类型默认就是"移动语义"：

```rust
let buf1 = create_buffer(1024);
let buf2 = buf1;  // Rust: buf1 被 move，与 C++ std::move(buf1) 等价
// buf1 不再有效
println!("{}", buf2.get_size());
```

因此，C++ 的移动语义在 Rust FFI 中是自然支持的，不需要额外处理。

### 移动工厂模式

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Buffer;

    // 返回值 Buffer：RVO/NRVO 或移动语义，Rust 侧直接拥有
    #[cpp(func = "Buffer create_large_buffer(size_t, const char*)")]
    fn create_large_buffer(size: usize, fill: *const i8) -> Buffer;
}

fn main() {
    let buf = create_large_buffer(4096, b"A\0".as_ptr() as _);
    println!("size = {}", buf.get_size());
    // buf 在此自动析构
}
```

## 注意事项

1. **`T&&` 在 Rust 侧等同于 `T`**：C++ 中 `T&&` 和 `T` 的 ABI 处理方式相同，Rust 侧均映射为值类型
2. **`noexcept` 移动**：C++ 标准库要求移动操作为 `noexcept`，否则 `std::vector` 等在重新分配时会使用拷贝；hicc 不强制此约束，但建议遵循
3. **移动后状态**：C++ 移动后对象处于"有效但不确定"状态，Rust 侧通过所有权系统保证不会再次访问已 move 的对象
4. **右值限定方法识别**：cpp2rust-demo 通过 `qualType.trim_end().ends_with(") &&")` 检测右值限定方法
