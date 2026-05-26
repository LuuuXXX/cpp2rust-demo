# 033_raii_pattern - RAII 模式

## C++ 特性

本示例展示 C++ 中 RAII（Resource Acquisition Is Initialization）模式的 FFI 处理方式。

## C++ 代码

### raii_pattern.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct Mutex;
struct ScopedLock;

struct Mutex* mutex_new(void);
void mutex_delete(struct Mutex* self);
void mutex_lock(struct Mutex* self);
void mutex_unlock(struct Mutex* self);

// 作用域锁：构造时加锁，析构时解锁
struct ScopedLock* scoped_lock_new(struct Mutex* mutex);
void scoped_lock_delete(struct ScopedLock* self);

#ifdef __cplusplus
}
#endif
```

### raii_pattern.cpp

```cpp
#include "raii_pattern.h"
#include <mutex>

struct Mutex {
    std::mutex mtx;
};

struct ScopedLock {
    Mutex* mutex;
    explicit ScopedLock(Mutex* m) : mutex(m) {
        mutex->mtx.lock();
    }
    ~ScopedLock() {
        mutex->mtx.unlock();
    }
};

Mutex* mutex_new() { return new Mutex(); }
void mutex_delete(Mutex* self) { delete self; }
void mutex_lock(Mutex* self) { self->mtx.lock(); }
void mutex_unlock(Mutex* self) { self->mtx.unlock(); }

ScopedLock* scoped_lock_new(Mutex* mutex) {
    return new ScopedLock(mutex);
}
void scoped_lock_delete(ScopedLock* self) { delete self; }
```

## RAII 模式原理

### 核心思想

```cpp
class ResourceGuard {
    Resource* resource;
public:
    ResourceGuard() {
        resource = acquire_resource();  // 获取资源
    }
    ~ResourceGuard() {
        release_resource(resource);     // 释放资源
    }
};
```

### RAII 的优势

1. **异常安全**：即使抛出异常，析构函数也会执行
2. **作用域绑定**：资源生命周期与对象绑定
3. **不可重入**：编译期保证

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "raii_pattern"]

    struct Mutex;
    struct ScopedLock;

    #[cpp(func = "struct Mutex* mutex_new(void)")]
    fn mutex_new() -> *mut Mutex;

    #[cpp(func = "struct ScopedLock* scoped_lock_new(struct Mutex*)")]
    fn scoped_lock_new(mtx: *mut Mutex) -> *mut ScopedLock;

    #[cpp(func = "void scoped_lock_delete(struct ScopedLock*)")]
    unsafe fn scoped_lock_delete(lock: *mut ScopedLock);
}

// Rust RAII Guard
struct MutexGuard<'a> {
    mutex: &'a Mutex,
}

impl Drop for MutexGuard<'_> {
    fn drop(&mut self) {
        unsafe { mutex_unlock(self.mutex) }
    }
}
```

## FFI 对比分析

| 方面 | C++ RAII | Rust FFI |
|------|----------|----------|
| 构造 | 构造函数 | Rust 创建对象 |
| 析构 | 析构函数 | Drop trait |
| 异常安全 | 是 | 是 |
| 资源释放 | 自动 | Rust Drop 调用 |

## 关键点

1. **ScopedLock 对象**：Rust 侧创建后在合适的时机销毁
2. **生命周期管理**：确保析构函数被调用
3. **Rust Drop trait**：提供类似 RAII 的自动资源管理
4. **FFI 边界**：C++ 构造函数/析构函数不直接暴露

## 总结

- RAII 是 C++ 最核心的资源管理模式
- FFI 边界通过"创建/销毁函数对"模拟
- Rust 可以通过 Drop trait 实现等效行为
- 作用域锁（ScopedLock）是最常见的 RAII 应用
## 运行结果

```
=== 033_raii_pattern - RAII 模式 ===

--- Manual Lock/Unlock ---
Critical section started
Critical section ended
Mutex 'unnamed' deleted

--- ScopedLock Demo ---
Inside scoped lock region
ScopedLock will auto-unlock on delete
Mutex 'unnamed' deleted

--- FileLock Demo ---
File is locked, performing I/O...

Rust FFI: RAII 模式映射
1. C++ RAII: 构造函数加锁，析构函数解锁
2. Rust 等效: Drop trait 自动调用
3. FFI 边界: ScopedLock 对象在 Rust 析构时自动释放
4. 推荐模式: Rust 封装 RAII guard 类型
```
