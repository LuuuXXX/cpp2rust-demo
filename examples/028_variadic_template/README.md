# 028_variadic_template - 可变参数模板

## C++ 特性

本示例展示 C++ 可变参数模板（variadic templates）的 FFI 挑战。

## C++ 代码

### variadic_template.cpp

```cpp
// 可变参数模板
template<typename... Args>
int sum(Args... args) {
    return (args + ...);  // C++17 折叠表达式
}

// 内部实现
template<typename... Args>
int do_sum(int first, Args... rest) {
    return first + do_sum(rest...);
}
```

## FFI 挑战

### 问题

```
template<typename... Args>
int sum(Args... args);  // 无法直接 FFI 导出
```

- C 可变参数（`va_list`）需要类型信息
- 可变参数模板在编译时展开，无法动态处理
- 运行时无法知道参数数量和类型

## FFI 解决方案

### main.rs

```rust
// 策略：为每个参数数量导出独立函数
fn sum_1(a: i32) -> i32;
fn sum_2(a: i32, b: i32) -> i32;
fn sum_3(a: i32, b: i32, c: i32) -> i32;
fn sum_4(a: i32, b: i32, c: i32, d: i32) -> i32;
fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
```

## 对比分析

| 方面 | C++ 可变参数模板 | Rust FFI |
|------|------------------|----------|
| 参数数量 | 动态（编译时展开） | 固定数量 |
| 类型安全 | 编译器保证 | 手动保证 |
| 代码生成 | 自动 | 手动编写每种情况 |
| 灵活性 | 高 | 低（需预定义最大参数数） |

## 总结

- 可变参数模板是最难 FFI 的 C++ 特性之一
- 标准解决方案：导出固定参数版本的函数
- 需要预定义最大参数数量
- 高级方案：使用 type-erased wrapper（如 `std::function`）