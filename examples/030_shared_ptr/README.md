# 030_shared_ptr - std::shared_ptr + weak_ptr

## C++ 特性

本示例展示 C++ `std::shared_ptr`（共享所有权的智能指针）和 `std::weak_ptr`（非拥有性引用）的 FFI 处理方式。

## C++ 代码

### shared_ptr.cpp

```cpp
// 模拟 std::shared_ptr
template<typename T>
class SharedPtr {
    T* ptr_;
    int* ref_count_;
public:
    SharedPtr(T* p) : ptr_(p), ref_count_(new int(1)) {}

    // 拷贝构造 - 增加引用计数
    SharedPtr(const SharedPtr& other)
        : ptr_(other.ptr_), ref_count_(other.ref_count_) {
        ++(*ref_count_);
    }

    int use_count() const { return *ref_count_; }
    T* get() const { return ptr_; }
};
```

## shared_ptr vs unique_ptr

| 特性 | shared_ptr | unique_ptr |
|------|------------|------------|
| 所有权 | 共享（引用计数） | 独占 |
| 拷贝 | 支持 | 不支持 |
| 移动 | 支持 | 支持 |
| use_count | 可获取 | 始终 1 |
| 线程安全 | 可配置 | 是 |

## weak_ptr 作用

```cpp
// 解决循环引用问题
class B;
class A {
    std::shared_ptr<B> b;
};
class B {
    std::weak_ptr<A> a;  // 不增加引用计数
};
```

## Rust FFI 方案

### main.rs

```rust
// 策略: C++ 侧管理引用计数
#[cpp(func = "int shareddata_use_count(struct SharedData*)")]
unsafe fn shareddata_use_count(self_: *mut SharedData) -> i32;

#[cpp(func = "struct SharedData* shareddata_clone(struct SharedData*)")]
unsafe fn shareddata_clone(self_: *mut SharedData) -> *mut SharedData;

#[cpp(func = "int shareddata_expired(struct SharedData*)")]
unsafe fn shareddata_expired(self_: *mut SharedData) -> i32;
```

## FFI 对比分析

| 方面 | C++ shared_ptr | Rust FFI |
|------|----------------|----------|
| 引用计数 | 编译器管理 | 需导出函数查询 |
| 拷贝 | 自动增加计数 | `clone()` 函数 |
| 释放 | 计数为 0 时 | Rust 仍需显式 delete |
| weak_ptr | `lock()` 获取 shared_ptr | `expired()` 检查 + 创建新 |

## 运行结果

```
=== 030_shared_ptr - std::shared_ptr + weak_ptr ===

Created SharedData: TestData
Use count: 1

Cloned SharedData: TestData
Use count (shared): 1

After reset, data1 is cleared


Cache demo:
cached1a and cached1b point to same cache entry

Rust FFI: shared_ptr 的处理方式
1. C++ 侧管理引用计数
2. Rust 侧通过 FFI 函数操作
3. 相当于 Rust 的 Arc<T>

weak_ptr 用于缓存，避免循环引用
相当于 Rust 的 Weak<T>
```

## 总结

- `shared_ptr` 的 FFI 需要导出引用计数操作函数
- `weak_ptr` 用于缓存和循环引用解决
- Rust 等价物：`Arc<T>` 和 `Weak<T>`
- 跨平台推荐使用 C++ 侧自定义包装类；在 Linux/Windows 上也可使用 hicc-std 提供的安全包装
  （hicc-std 0.2 在 macOS Apple Clang 下存在编译问题，暂不支持 macOS）