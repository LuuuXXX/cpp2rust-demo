# 示例 11：智能指针

## 特性概述

本示例展示 C++ 的**智能指针**，包括 `std::unique_ptr`（独占所有权）、`std::shared_ptr`（共享所有权）、`std::weak_ptr`（弱引用）以及自定义删除器。智能指针是现代 C++ 内存安全的核心工具，与 Rust 的所有权模型有深度对应关系。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| `unique_ptr<T>` | 独占所有权，移动语义，不可拷贝 |
| `shared_ptr<T>` | 共享所有权，引用计数 |
| `weak_ptr<T>` | 不持有所有权的观察者指针 |
| 自定义删除器 | `unique_ptr<T, Deleter>` |
| `make_unique<T>` | 工厂函数，推荐构建方式 |
| `make_shared<T>` | 工厂函数，避免二次内存分配 |

### 代码结构

```cpp
class Resource { int value; };

// 工厂函数
std::unique_ptr<Resource> create_unique(int value);
std::shared_ptr<Resource> create_shared(int value);

// 所有权转移
void consume_unique(std::unique_ptr<Resource> p);

// 共享所有权
void use_shared(std::shared_ptr<Resource>& p);

// 返回引用
std::shared_ptr<Resource>& get_static_shared();
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 信息：

| AST 表现 | 含义 |
|----------|------|
| `qualType: "std::unique_ptr<Resource>"` | `unique_ptr` 返回/参数类型 |
| `qualType: "std::shared_ptr<Resource>"` | `shared_ptr` 类型 |
| `qualType: "std::shared_ptr<Resource> &"` | `shared_ptr` 引用 |

## hicc 处理方式

### `unique_ptr` 映射

`std::unique_ptr<T>` 映射为 Rust 中的拥有对象（值语义），离开作用域时自动销毁：

```rust
hicc::import_class! {
    #[cpp(class = "Resource")]
    class Resource {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void set(int)")]
        fn set(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Resource;

    // unique_ptr 返回值 → 直接返回 Resource（所有权转移给 Rust）
    #[cpp(func = "std::unique_ptr<Resource> create_unique(int)")]
    fn create_unique(v: i32) -> Resource;

    // 消费 unique_ptr（所有权从 Rust 转移给 C++）
    #[cpp(func = "void consume_unique(std::unique_ptr<Resource>)")]
    fn consume_unique(p: Resource);
}
```

### `shared_ptr` 映射

`std::shared_ptr<T>` 同样映射为值语义，但底层通过引用计数共享：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Resource;

    #[cpp(func = "std::shared_ptr<Resource> create_shared(int)")]
    fn create_shared(v: i32) -> Resource;

    // 接受 shared_ptr 引用
    #[cpp(func = "void use_shared(std::shared_ptr<Resource>&)")]
    fn use_shared(p: &mut Resource);

    // 返回 shared_ptr 引用（生命周期绑定到静态存储）
    #[cpp(func = "std::shared_ptr<Resource>& get_static_shared()")]
    fn get_static_shared() -> ClassRef<'static, Resource>;
}
```

### `unique_ptr` 与 `make_unique` 工厂模式

这是 hicc 中最常见的对象创建模式：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Resource;

    // 使用 make_unique 创建对象
    #[cpp(func = "std::unique_ptr<Resource> std::make_unique<Resource, int>(int&&)")]
    fn resource_new(v: i32) -> Resource;
}

fn main() {
    let res = resource_new(42);  // Rust 拥有所有权
    println!("value = {}", res.get());
    // res 在此处析构，自动调用 C++ ~Resource()
}
```

### 自定义删除器

```rust
// C++: std::unique_ptr<Resource, CustomDeleter>
// hicc::unique_ptr<T> 处理非缺省删除器的情况
hicc::import_lib! {
    #![link_name = "example"]

    class Resource;

    #[cpp(func = "hicc::unique_ptr<Resource> create_with_custom_deleter()")]
    fn create_custom() -> Resource;
}
```

> **注意**：若 `unique_ptr` 的删除器模板参数非默认，对应的 Rust 类型应使用 `hicc::unique_ptr<T>` 而非 `std::unique_ptr<T>`。

### `weak_ptr` 处理

`std::weak_ptr<T>` 通常作为循环引用的打破机制，在 Rust 侧需要通过 `shared_ptr` 的 `lock()` 方法转换：

```rust
// weak_ptr 通常不直接暴露为 FFI 接口
// 通过工厂方法处理：
#[cpp(func = "std::shared_ptr<Resource> weak_lock(const std::weak_ptr<Resource>&)")]
fn weak_lock(w: &WeakResource) -> Resource;
```

## Rust 所有权与 C++ 智能指针对应关系

| C++ 智能指针 | Rust 对应 | 说明 |
|-------------|-----------|------|
| `unique_ptr<T>` | 值语义 `T`（owned） | 移动后 C++ 侧失去所有权 |
| `shared_ptr<T>` | 值语义 `T`（Arc 语义） | 拷贝时增加引用计数 |
| `weak_ptr<T>` | 需要 `lock()` 转 `shared_ptr` | 通过工厂函数处理 |
| 裸指针 `T*` | `ClassMutPtr<'_, T>` | 不管理生命周期 |

## 注意事项

1. **所有权转移**：将 `unique_ptr` 传给 Rust 后，C++ 侧的 `unique_ptr` 变为空（moved-from 状态）
2. **循环引用**：`shared_ptr` 循环引用会导致内存泄漏，Rust 侧同样无法自动检测，需设计时避免
3. **线程安全**：`shared_ptr` 的引用计数是线程安全的，但指向的对象不一定是线程安全的
4. **`shared_ptr` 的拷贝开销**：hicc 将 `shared_ptr` 映射为值类型，拷贝时会增加引用计数
