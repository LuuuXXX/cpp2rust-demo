# cpp2rust-demo

C++ 到 Rust FFI 示例集合，通过 48 个循序渐进的示例，演示 C++ 各种特性如何通过 [hicc](https://crates.io/crates/hicc) 映射到 Rust FFI 接口。

## 目录结构

```
cpp2rust-demo/
├── examples/          # 48 个示例，每个含 cpp/ 和 rust_hicc/ 子目录
├── docs/
│   ├── plans/         # 自动化工具方案文档
│   └── references/    # hicc、c2rust-demo 等参考文档
└── references/
    └── c2rust-demo/   # c2rust-demo 参考实现（LD_PRELOAD 拦截机制）
```

## 示例列表

| 编号 | 名称 | C++ 特性 | FFI 模式 |
|------|------|----------|----------|
| 001 | hello_world | 函数导出 | extern "C" 函数 |
| 002 | function_overload | 函数重载 | 重载解析模拟 |
| 003 | default_args | 默认参数 | 默认值模拟 |
| 004 | inline_functions | 内联函数 | 内联提示处理 |
| 005 | variadic_functions | 可变参数 | 固定参数 wrapper |
| 006 | class_basic | 类定义 | opaque pointer |
| 007 | class_constructor | 构造函数 | 工厂函数 |
| 008 | class_copy | 拷贝构造 | 拷贝语义模拟 |
| 009 | class_move | 移动构造 | 移动语义模拟 |
| 010 | class_static | 静态成员 | 静态函数包装 |
| 011 | class_const | const 成员 | const 正确性 |
| 012 | class_volatile | volatile | 内存顺序语义 |
| 013 | inheritance_single | 单继承 | 基类指针模拟 |
| 014 | inheritance_multiple | 多继承 | 多接口模拟 |
| 015 | virtual_basic | 虚函数 | 虚表模拟 |
| 016 | virtual_pure | 纯虚函数 | 抽象接口 |
| 017 | virtual_override | 函数覆盖 | 动态分发 |
| 018 | virtual_diamond | 菱形继承 | 虚继承处理 |
| 019 | operator_overload | 运算符重载 | 命名函数模拟 |
| 020 | friend_function | 友元函数 | 友元访问模拟 |
| 021 | explicit_ctor | explicit | 隐式转换阻止 |
| 022 | mutable_member | mutable | 状态修改语义 |
| 023 | typeid_rtti | typeid/RTTI | 类型信息传递 |
| 024 | template_function | 函数模板 | 实例化模拟 |
| 025 | template_class | 类模板 | 特化处理 |
| 026 | template_specialization | 模板特化 | 特化版本选择 |
| 027 | template_instantiation | 显式实例化 | 链接处理 |
| 028 | variadic_template | 可变参数模板 | 参数包展开 |
| 029 | unique_ptr | unique_ptr | 独占所有权 |
| 030 | shared_ptr | shared_ptr | 引用计数 |
| 031 | custom_deleter | 自定义删除器 | 资源释放策略 |
| 032 | placement_new | placement new | 指定地址构造 |
| 033 | raii_pattern | RAII 模式 | 资源获取释放 |
| 034 | vector_basic | vector | 动态数组模拟 |
| 035 | map_basic | map | 红黑树映射 |
| 036 | string_basic | string | 字符串处理 |
| 037 | array_basic | array | 固定数组 |
| 038 | tuple_basic | tuple | 异构集合 |
| 039 | lambda_basic | lambda | 闭包模拟 |
| 040 | std_function | std::function | 函数包装器 |
| 041 | functional_bind | std::bind | 部分应用模拟 |
| 042 | exception_basic | 异常处理 | 错误码模式 |
| 043 | namespace_nested | 嵌套命名空间 | 命名空间映射 |
| 044 | enum_class | 强类型枚举 | 类型安全枚举 |
| 045 | union_basic | 共用体 | 内存 overlay |
| 046 | constexpr_basic | constexpr | 编译期计算 |
| 047 | noexcept_basic | noexcept | 异常规格 |
| 048 | summary | 综合 FFI 模式 | 所有策略组合 |

## 学习路径

1. **入门**：`001_hello_world` → `006_class_basic`
2. **进阶**：`013_inheritance_single` → `023_typeid_rtti`
3. **模板**：`024_template_function` → `028_variadic_template`
4. **内存**：`029_unique_ptr` → `033_raii_pattern`
5. **STL**：`034_vector_basic` → `040_std_function`
6. **高级**：`041_functional_bind` → `047_noexcept_basic`

## 构建单个示例

```bash
cd examples/001_hello_world

# 编译 C++ 共享库
cd cpp && g++ -shared -fPIC hello_world.cpp -o libhello_world.so && cd ..

# 编译并运行 Rust FFI
cd rust_hicc && cargo run
```

## 依赖

- C++11 或更高版本的编译器（g++ / clang++）
- Rust 工具链（rustc / cargo）
- [`hicc`](https://crates.io/crates/hicc) `0.2` 和 [`hicc-build`](https://crates.io/crates/hicc-build) `0.2`

## 许可

本项目仅供学习参考。
