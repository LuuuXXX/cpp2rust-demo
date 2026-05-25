# 示例 19：内存管理

## 特性概述

本示例展示 C++ 的**内存管理**技术，包括 `new`/`delete`、placement new（原地构造）、RAII 惯用法以及自定义内存分配。placement new 允许在预分配的内存上构造对象，hicc 通过 placement new 辅助模块提供对应支持。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| `new`/`delete` | 堆内存分配与释放 |
| `new[]`/`delete[]` | 数组的堆内存分配 |
| Placement new | 在指定内存地址构造对象 |
| RAII | 构造时获取资源，析构时释放资源 |
| 重载 `operator new` | 自定义内存分配策略 |
| 对齐分配 | `std::aligned_alloc`、`alignas` |

### 代码结构

```cpp
class TrackedObject {
    static int next_id;
public:
    int get_id() const;
    static int get_created_count();
};

// 重载全局 operator new/delete
void* operator new(size_t size);
void operator delete(void* ptr) noexcept;

// Placement new 容器
class PlacementBuffer {
    char data[1024];
    void* allocate(size_t size);

    // 在 data 中构造对象
    template<typename T>
    T* construct(Args&&... args) {
        return new (allocate(sizeof(T))) T(std::forward<Args>(args)...);
    }
};
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `CXXNewExpr` | `new` 表达式（含 placement new） |
| `CXXNewExpr.isPlacementNew: true` | Placement new |
| `CXXDeleteExpr` | `delete` 表达式 |
| `FunctionDecl` `operator new` | 重载的 new 运算符 |
| `FunctionDecl` `operator delete` | 重载的 delete 运算符 |

## hicc 处理方式

### Placement New 辅助模块

cpp2rust-demo 在 `rust/src/free/placement_new.rs` 中生成 placement new 辅助绑定：

```rust
// auto-generated placement_new.rs
hicc::import_lib! {
    #![link_name = "example"]

    class TrackedObject;

    // placement new 辅助：在指定地址构造 TrackedObject
    #[cpp(func = "TrackedObject* @placement_new<TrackedObject>(void*)")]
    fn placement_new_tracked(mem: *mut std::ffi::c_void) -> ClassMutPtr<'_, TrackedObject>;

    // 显式调用析构函数
    #[cpp(func = "void @destroy<TrackedObject>(TrackedObject*)")]
    fn destroy_tracked(obj: &mut TrackedObject);
}
```

### Rust 侧 Placement New 使用

```rust
use std::alloc::{alloc, Layout};

fn main() {
    let layout = Layout::new::<TrackedObject>();
    let mem = unsafe { alloc(layout) as *mut std::ffi::c_void };

    // 在 Rust 分配的内存上构造 C++ 对象
    let obj = placement_new_tracked(mem);

    // 使用对象...
    println!("id = {}", obj.get_id());

    // 必须手动调用析构（不会自动 drop）
    unsafe { destroy_tracked(obj.as_mut().unwrap()); }

    // 释放内存
    unsafe { std::alloc::dealloc(mem as *mut u8, layout); }
}
```

### RAII 模式

RAII（Resource Acquisition Is Initialization）是 C++ 资源管理的核心惯用法，在 Rust 中通过 `Drop` trait 自然对应：

```rust
hicc::import_class! {
    #[cpp(class = "TrackedObject")]
    class TrackedObject {
        #[cpp(method = "int get_id() const")]
        fn get_id(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class TrackedObject;

    // 工厂函数（RAII 对象通过 unique_ptr 管理）
    #[cpp(func = "std::unique_ptr<TrackedObject> std::make_unique<TrackedObject>()")]
    fn tracked_new() -> TrackedObject;
}

fn main() {
    let obj = tracked_new();  // RAII：构造时分配
    println!("id = {}", obj.get_id());
    // RAII：obj 离开作用域时自动调用 ~TrackedObject()
}
```

### 自定义内存分配器

对于使用自定义分配器的类，需要确保 Rust 侧的内存操作与 C++ 侧兼容：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class PlacementBuffer;

    // PlacementBuffer 的内存分配方法
    #[cpp(func = "void* PlacementBuffer::allocate(size_t)")]
    fn placement_buffer_allocate(buf: &mut PlacementBuffer, size: usize) -> *mut std::ffi::c_void;
}
```

### `TrackedObject` 静态计数器

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 静态成员函数
    #[cpp(func = "int TrackedObject::get_created_count()")]
    fn tracked_get_count() -> i32;
}
```

## 注意事项

1. **Placement new 的责任**：使用 placement new 构造的对象，**必须**手动调用析构函数，然后再释放内存；hicc 不会自动处理这种情况
2. **对齐要求**：hicc 要求所有 C++ 对象至少按 `size_t` 字节对齐，未对齐地址视为非法指针
3. **自定义 `operator new` 的影响**：全局 `operator new` 被重载后，所有 `new` 表达式都会使用自定义版本，影响范围广泛
4. **`delete[]` 与 `delete` 配对**：数组分配必须用 `delete[]` 释放，Rust 侧应避免混用
5. **Rust 分配器**：Rust 有自己的全局分配器（`GlobalAlloc`），与 C++ 的 `operator new` 不对应，混用时需格外小心
