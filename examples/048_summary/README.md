# 048_summary - 汇总

## 项目概述

本项目是一个 C++ 到 Rust 的 FFI（Foreign Function Interface）示例集合，通过 48 个循序渐进示例展示 C++ 各种特性如何映射到 Rust FFI 接口。

### 目录结构

```
examples/
├── 001_hello_world/       # 简单函数导出
├── 002_function_overload/  # 函数重载
├── 003_default_args/       # 默认参数
├── 004_inline_functions/   # 内联函数
├── 005_variadic_functions/ # 可变参数函数
├── 006_class_basic/        # 基础类
├── 007_class_constructor/   # 构造函数
├── 008_class_copy/        # 拷贝构造
├── 009_class_move/        # 移动构造
├── 010_class_static/      # 静态成员
├── 011_class_const/       # const 成员函数
├── 012_class_volatile/    # volatile 成员
├── 013_inheritance_single/ # 单继承
├── 014_inheritance_multiple/ # 多继承
├── 015_virtual_basic/     # 虚函数基础
├── 016_virtual_pure/      # 纯虚函数
├── 017_virtual_override/  # 函数覆盖
├── 018_virtual_diamond/   # 菱形继承
├── 019_operator_overload/ # 运算符重载
├── 020_friend_function/  # 友元函数
├── 021_explicit_ctor/     # explicit 构造函数
├── 022_mutable_member/    # mutable 成员
├── 023_typeid_rtti/       # typeid 和 RTTI
├── 024_template_function/ # 函数模板
├── 025_template_class/    # 类模板
├── 026_template_specialization/ # 模板特化
├── 027_template_instantiation/ # 模板实例化
├── 028_variadic_template/ # 可变参数模板
├── 029_unique_ptr/       # unique_ptr
├── 030_shared_ptr/        # shared_ptr
├── 031_custom_deleter/    # 自定义删除器
├── 032_placement_new/     # placement new
├── 033_raii_pattern/      # RAII 模式
├── 034_vector_basic/      # vector 基础
├── 035_map_basic/         # map 基础
├── 036_string_basic/      # string 基础
├── 037_array_basic/       # array 基础
├── 038_tuple_basic/       # tuple 基础
├── 039_lambda_basic/      # lambda 基础
├── 040_std_function/      # std::function
├── 041_functional_bind/   # std::bind
├── 042_exception_basic/   # 异常处理
├── 043_namespace_nested/  # 嵌套命名空间
├── 044_enum_class/        # 强类型枚举
├── 045_union_basic/      # 共用体
├── 046_constexpr_basic/   # constexpr
├── 047_noexcept_basic/    # noexcept
└── 048_summary/           # 本汇总
```

## 示例分类

### 第一部分：基础（001-005）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 001 | hello_world | 函数导出 | extern "C" 函数 |
| 002 | function_overload | 函数重载 | 重载解析模拟 |
| 003 | default_args | 默认参数 | 默认值模拟 |
| 004 | inline_functions | 内联函数 | 内联提示处理 |
| 005 | variadic_functions | 可变参数 | va_list 转换 |

### 第二部分：类基础（006-012）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 006 | class_basic | 类定义 | opaque pointer |
| 007 | class_constructor | 构造函数 | 工厂函数 |
| 008 | class_copy | 拷贝构造 | 拷贝语义模拟 |
| 009 | class_move | 移动构造 | 移动语义模拟 |
| 010 | class_static | 静态成员 | 静态函数包装 |
| 011 | class_const | const 成员 | const 正确性 |
| 012 | class_volatile | volatile | 内存顺序语义 |

### 第三部分：继承与多态（013-018）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 013 | inheritance_single | 单继承 | 基类指针模拟 |
| 014 | inheritance_multiple | 多继承 | 多接口模拟 |
| 015 | virtual_basic | 虚函数 | 虚表模拟 |
| 016 | virtual_pure | 纯虚函数 | 抽象接口 |
| 017 | virtual_override | 函数覆盖 | 动态分发 |
| 018 | virtual_diamond | 菱形继承 | 虚继承处理 |

### 第四部分：运算符与特殊成员（019-023）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 019 | operator_overload | 运算符重载 | 命名函数模拟 |
| 020 | friend_function | 友元函数 | 友元访问模拟 |
| 021 | explicit_ctor | explicit | 隐式转换阻止 |
| 022 | mutable_member | mutable | 状态修改语义 |
| 023 | typeid_rtti | typeid/RTTI | 类型信息传递 |

### 第五部分：模板（024-028）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 024 | template_function | 函数模板 | 实例化模拟 |
| 025 | template_class | 类模板 | 特化处理 |
| 026 | template_specialization | 模板特化 | 特化版本选择 |
| 027 | template_instantiation | 显式实例化 | 链接处理 |
| 028 | variadic_template | 可变参数模板 | 参数包展开 |

### 第六部分：智能指针与内存（029-033）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 029 | unique_ptr | unique_ptr | 独占所有权 |
| 030 | shared_ptr | shared_ptr | 引用计数 |
| 031 | custom_deleter | 自定义删除器 | 资源释放策略 |
| 032 | placement_new | placement new | 指定地址构造 |
| 033 | raii_pattern | RAII 模式 | 资源获取释放 |

### 第七部分：STL 容器（034-038）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 034 | vector_basic | vector | 动态数组模拟 |
| 035 | map_basic | map | 红黑树映射 |
| 036 | string_basic | string | 字符串处理 |
| 037 | array_basic | array | 固定数组 |
| 038 | tuple_basic | tuple | 异构集合 |

### 第八部分：函数对象（039-042）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 039 | lambda_basic | lambda | 闭包模拟 |
| 040 | std_function | std::function | 函数包装器 |
| 041 | functional_bind | std::bind | 部分应用模拟 |
| 042 | exception_basic | 异常处理 | 错误码模式 |

### 第九部分：其他高级特性（043-048）

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 043 | namespace_nested | 嵌套命名空间 | 命名空间映射 |
| 044 | enum_class | 强类型枚举 | 类型安全枚举 |
| 045 | union_basic | 共用体 | 内存overlay |
| 046 | constexpr_basic | constexpr | 编译期计算 |
| 047 | noexcept_basic | noexcept | 异常规格 |
| 048 | summary | 综合 FFI 模式 | 所有策略组合 |

## 构建所有示例

### 前提条件

- C++ 编译器（g++ 或 clang++）
- Rust 编译器（rustc）和 Cargo
- hicc 和 hicc-build crate

### 构建单个示例

```bash
# 进入示例目录
cd 001_hello_world

# 编译 C++ 共享库
cd cpp
g++ -shared -fPIC hello_world.cpp -o libhello_world.so
cd ..

# 编译 Rust FFI
cd rust_hicc
cargo build
cd ..
```

### 批量构建脚本

```bash
#!/bin/bash

for dir in 0*/; do
    if [ -d "$dir/cpp" ] && [ -d "$dir/rust_hicc" ]; then
        echo "Building $dir..."
        (cd "$dir/cpp" && g++ -shared -fPIC *.cpp -o lib"${dir%/}"".so" 2>/dev/null)
        (cd "$dir/rust_hicc" && cargo build 2>/dev/null)
        echo "Done: $dir"
    fi
done
```

## FFI 模式总结

### Opaque Pointer（不透明指针）

最常用的 FFI 模式，用于处理 C++ 类：

```cpp
// C++
struct MyClass;
MyClass* my_class_new();
void my_class_delete(MyClass*);
```

```rust
// Rust
struct MyClass;
#[cpp(func = "struct MyClass* my_class_new()")]
fn my_class_new() -> *mut MyClass;
```

### 函数指针回调

处理 C++ 回调到 Rust：

```cpp
// C++
typedef int (*Callback)(int);
void set_callback(Callback cb);
```

```rust
// Rust
type Callback = extern "C" fn(i32) -> i32;
#[cpp(func = "void set_callback(int(*)(int))")]
fn set_callback(cb: Option<Callback>);
```

### 错误处理模式

跨 FFI 边界的异常传播不可行，需要特殊模式：

1. **错误码返回**：`int error = do_something();`
2. **全局状态**：`get_last_error()` 获取错误信息
3. **hicc::Exception<T>**：类型安全的异常封装

### 类型映射表

| C++ 类型 | Rust 类型 | 说明 |
|----------|-----------|------|
| `int` | `i32` | |
| `unsigned` | `u32` | |
| `long` | `i64` / `isize` | 平台相关 |
| `float` | `f32` | |
| `double` | `f64` | |
| `char*` | `*const i8` | C 字符串 |
| `void*` | `*mut std::ffi::c_void` | |
| `bool` | `bool` | |

## 项目依赖

### C++ 端

- C++11 或更高版本
- 标准库

### Rust 端

```toml
[dependencies]
hicc = "0.2"

[build-dependencies]
hicc-build = "0.2"
```

## 学习路径

1. **入门**：`001_hello_world` -> `006_class_basic`
2. **进阶**：`013_inheritance_single` -> `023_typeid_rtti`
3. **模板**：`024_template_function` -> `028_variadic_template`
4. **内存**：`029_unique_ptr` -> `033_raii_pattern`
5. **STL**：`034_vector_basic` -> `040_std_function`
6. **高级**：`041_functional_bind` -> `047_noexcept_basic`

## 许可

本项目仅供学习参考。