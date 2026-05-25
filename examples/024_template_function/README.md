# 024_template_function - 函数模板

## C++ 特性

本示例展示 C++ 函数模板的 FFI 处理方式。模板是编译时多态，必须为每种类型显式实例化。

## C++ 代码

### template_function.cpp

```cpp
// 模板函数 swap - 内部实现
template<typename T>
void do_swap(T* a, T* b) {
    T temp = *a;
    *a = *b;
    *b = temp;
}

// 必须显式实例化要导出的类型
void swap_int(int* a, int* b) { do_swap<int>(a, b); }
void swap_double(double* a, double* b) { do_swap<double>(a, b); }
```

## Rust FFI 代码

### main.rs

```rust
// 每个模板实例化导出为一个函数
unsafe fn swap_int(a: *mut i32, b: *mut i32);
unsafe fn swap_double(a: *mut f64, b: *mut f64);
unsafe fn swap_char(a: *mut i8, b: *mut i8);
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<T> void swap(T*, T*)` | 不可直接导出 |
| 模板实例化 | 编译时自动生成 | 必须手动实例化 |
| 符号 | `swap<int>` / `swap<double>` | `swap_int` / `swap_double` |
| 类型安全 | 编译器保证 | 手动保证 |

## 总结

- C++ 模板无法直接 FFI 导出
- 策略：为每种需要的类型创建显式实例化函数
- Rust 端调用时需要知道具体类型
- 这是 FFI 处理模板的标准方法