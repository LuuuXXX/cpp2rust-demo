# 场景 04：全纯虚抽象类 → `#[interface]` + `@make_proxy`

本示例演示 cpp2rust-demo 处理**全纯虚抽象类**的完整流水线：
从 C++ 纯虚接口生成 hicc `#[interface]` trait 和 `@make_proxy` 反向绑定，
使 Rust 代码可以实现并传递 C++ 抽象类接口。

---

## 背景

RapidJSON 支持自定义 allocator（通过 `GenericDocument<Encoding, Allocator>`）。
其 `MemoryPoolAllocator` 要求 allocator 类型提供 `Malloc` / `Realloc` / `Free` 方法。

本示例将这一模式抽象为纯虚接口 `IAllocator`，展示：
- cpp2rust-demo 如何将全纯虚类提取为 `#[interface]`
- 如何通过 `@make_proxy` 让 Rust struct 实现 C++ 抽象类
- hicc `Interface<T>` 包装器的用法

---

## C++ 源码（`allocator_interface.hpp`）

```cpp
#pragma once
#include <cstddef>  // size_t

class IAllocator {
public:
    virtual ~IAllocator() {}

    virtual void* Malloc(size_t size) = 0;
    virtual void* Realloc(void* original_ptr,
                          size_t original_size,
                          size_t new_size) = 0;
    virtual void Free(void* ptr) = 0;
    virtual bool CanFree() const = 0;
};
```

所有 public 方法均为 `= 0`（纯虚）→ cpp2rust-demo 判定为 **全纯虚类**（`is_abstract = true`）。

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature rj04 --link rapidjson --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/04-abstract-interface/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj04

# 第 3 步：查看产物
cat .cpp2rust/rj04/rust/src/lib.rs
```

---

## 预期生成产物

### `method/mtd_entry.rs`（`#[interface]` trait）

```rust
hicc::import_class! {
    #[interface]
    class IAllocator {
        #[cpp(method = "void * Malloc(size_t)")]
        fn malloc(&mut self, size: usize) -> *mut core::ffi::c_void;

        #[cpp(method = "void * Realloc(void *, size_t, size_t)")]
        fn realloc(
            &mut self,
            original_ptr: *mut core::ffi::c_void,
            original_size: usize,
            new_size: usize,
        ) -> *mut core::ffi::c_void;

        #[cpp(method = "void Free(void *)")]
        fn free(&mut self, ptr: *mut core::ffi::c_void);

        #[cpp(method = "bool CanFree() const")]
        fn can_free(&self) -> bool;
    }
}
```

### `free/fn_entry.rs`（`@make_proxy` 绑定）

```rust
hicc::import_lib! {
    #![link_name = "rapidjson"]

    class IAllocator;

    // @make_proxy support — wrap a Rust struct as a C++ IAllocator.
    #[cpp(func = "IAllocator @make_proxy<IAllocator>()")]
    #[interface(name = "IAllocator")]
    fn new_i_allocator_proxy(intf: hicc::Interface<IAllocator>) -> IAllocator;
}
```

---

## 场景解析

### 1. 全纯虚类的判定条件

`extract_class_body()` 在遍历成员方法时，将所有 `is_pure = true` 的方法收集到
`pure_virtual_nodes`。遍历结束后：

```
if pure_virtual_nodes.len() == class 所有公有方法数:
    is_abstract = true   → 生成 #[interface] trait
else if pure_virtual_nodes 非空:
    has_pure_virtual = true  → 生成 companion interface（见场景 06）
```

`IAllocator` 的全部 4 个公有方法均为 `= 0`，因此 `is_abstract = true`。

### 2. `#[interface]` trait 的 hicc 语义

hicc 中 `#[interface]` 的含义：
- 该 Rust trait 对应一个 C++ 纯虚类
- Rust struct 可以通过 `impl IAllocator for MyStruct` 提供实现
- 通过 `@make_proxy` 生成的 C++ 对象会将调用转发到 Rust impl

```rust
struct MyAllocator { data: Vec<u8> }

impl IAllocator for MyAllocator {
    fn malloc(&mut self, size: usize) -> *mut core::ffi::c_void {
        // ... 分配逻辑
    }
    fn realloc(&mut self, ptr: *mut _, old: usize, new: usize) -> *mut _ {
        // ... 重分配逻辑
    }
    fn free(&mut self, ptr: *mut core::ffi::c_void) {
        // ... 释放逻辑
    }
    fn can_free(&self) -> bool { true }
}

// 将 MyAllocator 注册为 C++ IAllocator 子类
let proxy: IAllocator = new_i_allocator_proxy(MyAllocator { data: vec![] });
// 现在可以将 proxy 传给任何期望 IAllocator* 的 C++ 函数
```

### 3. `@make_proxy` 的工作原理

`@make_proxy<T>()` 是 hicc 提供的特殊语法，用于：
1. 接受一个 `hicc::Interface<T>`（Rust impl 的包装器）
2. 在 C++ 侧创建一个 vtable-backed 代理对象
3. 返回一个 `T`（C++ 对象），其虚函数调用会转发到 Rust 侧实现

cpp2rust-demo 自动为每个 `is_abstract = true` 的类生成此绑定。

### 4. `size_t` → `usize` 的类型映射

`size_t` 在 cpp2rust-demo 的类型映射表中直接对应 Rust 的 `usize`
（通过 `cpp_to_rust_type` 函数的映射表）。
`void*` 对应 `*mut core::ffi::c_void`。

### 5. 析构函数的处理

`virtual ~IAllocator() {}` 是虚析构函数。
析构函数被归类为 `HiccLimitation` skip，
因为 hicc 不支持在 `import_class!` 中显式绑定析构函数
（对象生命周期管理由 C++ 侧负责）。
这在接口报告的 `Skipped declarations` 节会可见。

---

## Rust 使用示例

```rust
// 1. 定义 Rust allocator struct
struct SystemAllocator;

// 2. 实现 IAllocator interface
impl IAllocator for SystemAllocator {
    fn malloc(&mut self, size: usize) -> *mut core::ffi::c_void {
        unsafe { libc::malloc(size) }
    }
    fn realloc(&mut self, ptr: *mut core::ffi::c_void, _old: usize, new_size: usize)
        -> *mut core::ffi::c_void {
        unsafe { libc::realloc(ptr, new_size) }
    }
    fn free(&mut self, ptr: *mut core::ffi::c_void) {
        unsafe { libc::free(ptr) }
    }
    fn can_free(&self) -> bool { true }
}

// 3. 创建 C++ 代理对象
let alloc_proxy = new_i_allocator_proxy(SystemAllocator);

// 4. 传递给期望 IAllocator 的 C++ 函数
// some_cpp_function_needing_allocator(alloc_proxy);
```

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 虚析构函数跳过 | hicc 不暴露析构函数绑定；C++ 侧管理生命周期 |
| `void*` 参数 | 映射为 `*mut core::ffi::c_void`，Rust 侧需 `unsafe` 解引用 |
| `size_t` 平台差异 | `usize` 在 32/64 位平台大小不同，与 C++ `size_t` 保持一致 |
| `@make_proxy` 需要 hicc 运行时 | 需要在 `build.rs` 的 include path 中包含 hicc 头文件目录 |
