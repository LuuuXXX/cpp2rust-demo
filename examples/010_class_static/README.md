# 010_class_static - 静态成员

## C++ 特性

本示例展示 C++ 类的静态成员变量和静态成员函数的 FFI 映射。

## C++ 代码

### class_static.h

```cpp
// 静态方法
int counter_getInstanceCount(void);
void counter_resetInstanceCount(void);
```

### class_static.cpp

```cpp
// 静态成员：所有实例共享
static int instance_count = 0;

struct Counter* counter_new(void) {
    ++instance_count;
    return new Counter{0};
}

int counter_getInstanceCount(void) {
    return instance_count;
}

void counter_resetInstanceCount(void) {
    instance_count = 0;
}
```

## 静态成员与 FFI

### 什么是静态成员

| 类型 | 特点 |
|------|------|
| 静态成员变量 | 所有实例共享一份 |
| 静态成员函数 | 不依赖实例即可调用 |

### FFI 映射策略

| C++ 概念 | FFI 映射 |
|----------|----------|
| 静态成员变量 | 全局变量（内部管理） |
| 静态成员函数 | 普通函数（无 this 参数） |

## Rust FFI 代码

```rust
// 静态方法：不需要实例调用
#[cpp(func = "int counter_getInstanceCount(void)")]
fn counter_getInstanceCount() -> i32;

#[cpp(func = "void counter_resetInstanceCount(void)")]
fn counter_resetInstanceCount();
```

### 调用方式对比

```rust
// 实例方法：需要实例
let c1 = counter_new();
counter_increment(c1);  // 隐式传递 this

// 静态方法：不需要实例
counter_getInstanceCount();  // 直接调用
counter_resetInstanceCount();
```

## 关键点

### 静态成员变量的封装

本例中静态成员 `instance_count` 是文件作用域静态变量，对外不可见：

```cpp
static int instance_count = 0;  // 仅本文件可见
```

外部只能通过静态方法访问。

### Rust 端调用

Rust 调用静态方法不需要实例：

```rust
let count = counter_getInstanceCount();  // 直接调用
```

## 运行结果

```
Initial instance count: 0
Instance count after creating 3: 3
c1 value: 2
c2 value: 1
c3 value: 0
Instance count after deleting c1: 2
Instance count after deleting all: 0
Instance count after reset: 0

Rust FFI: Static members work!
```

## 总结

1. **静态成员变量**：所有实例共享，通过静态方法访问
2. **静态成员函数**：无 this 参数，直接调用
3. **FFI 映射**：静态方法映射为普通函数
4. **封装**：静态成员变量通常设为 private，通过方法访问
