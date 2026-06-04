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
void swap_char(char* a, char* b) { do_swap<char>(a, b); }
void swap_int_array(int* arr, int i, int j) { do_swap<int>(&arr[i], &arr[j]); }
int get_int_array(int* arr, int idx) { return arr[idx]; }
void set_int_array(int* arr, int idx, int value) { arr[idx] = value; }
```

## Rust FFI 代码

### main.rs

工具自动生成的 `import_lib!` 脚手架包含全部 6 个导出函数：

```rust
hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "void swap_int(int*, int*)")]
    unsafe fn swap_int(a: *mut i32, b: *mut i32);

    #[cpp(func = "void swap_double(double*, double*)")]
    unsafe fn swap_double(a: *mut f64, b: *mut f64);

    #[cpp(func = "void swap_char(char*, char*)")]
    unsafe fn swap_char(a: *mut i8, b: *mut i8);

    #[cpp(func = "void swap_int_array(int*, int, int)")]
    unsafe fn swap_int_array(arr: *mut i32, i: i32, j: i32);

    // ...
}
```

> **Windows MSVC ABI 说明**：`char*` 参数在 MSVC 上经过 hicc ExportFunction 包装后存在 ABI 兼容问题，实际调用时需绕过 hicc wrapper，改用裸 `extern "C"` 绑定：
>
> ```rust
> extern "C" {
>     // Windows MSVC: hicc wrapper for char* has ABI issues; use direct C binding.
>     #[link_name = "swap_char"]
>     fn swap_char_raw(a: *mut i8, b: *mut i8);
> }
> ```
>
> `import_lib!` 中保留 `swap_char` 声明（与工具生成的脚手架保持一致），运行时通过 `swap_char_raw` 调用。

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<T> void swap(T*, T*)` | 不可直接导出 |
| 模板实例化 | 编译时自动生成 | 必须手动实例化 |
| 符号 | `swap<int>` / `swap<double>` | `swap_int` / `swap_double` |
| 类型安全 | 编译器保证 | 手动保证 |

## 运行结果

```
=== 024_template_function - 函数模板 ===

Before swap: a = 10, b = 20
swap_int called
After swap: a = 20, b = 10

Before swap: x = 3.14, y = 2.71
swap_double called
After swap: x = 2.71, y = 3.14

Before swap: c1 = A, c2 = B
swap_char called
After swap: c1 = B, c2 = A

Array before swap(0,4): [1, 2, 3, 4, 5]
swap_int_array: arr[0] <-> arr[4]
Array after swap(0,4): [5, 2, 3, 4, 1]

Rust FFI: 模板必须在 C++ 侧实例化
每个模板实例 = 一个独立的 C 函数
swap_int, swap_double, swap_char 是三个不同的函数
```

## 总结

- C++ 模板无法直接 FFI 导出
- 策略：为每种需要的类型创建显式实例化函数
- Rust 端调用时需要知道具体类型
- 这是 FFI 处理模板的标准方法