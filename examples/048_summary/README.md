# 048_summary - 汇总（hicc 直出，去 shim）

## 项目概述

本项目是一个 C++ 到 Rust 的 FFI（Foreign Function Interface）示例集合，通过 48 个循序渐进示例展示 C++ 各种特性如何映射到 Rust FFI 接口。本收尾示例保留“总结全系列”的定位，但代码风格迁移为 hicc shimless 直出：直接绑定 C++ 命名空间类与自由函数，不再编写 extern-C shim。

### 目录结构

```
examples/
├── 001_hello_world/       # 简单函数导出
├── 002_function_overload/  # 函数重载
├── 003_default_args/       # 默认参数
├── 004_inline_functions/   # 内联函数
├── 005_variadic_functions/ # 可变参数函数
├── 006_class_basic/        # 基础类
├── 007_class_constructor/  # 构造函数
├── 008_class_copy/         # 拷贝构造
├── 009_class_move/         # 移动构造
├── 010_class_static/       # 静态成员
├── 011_class_const/        # const 成员函数
├── 012_class_volatile/     # volatile 成员
├── 013_inheritance_single/ # 单继承
├── 014_inheritance_multiple/ # 多继承
├── 015_virtual_basic/      # 虚函数基础
├── 016_virtual_pure/       # 纯虚函数
├── 017_virtual_override/   # 函数覆盖
├── 018_virtual_diamond/    # 菱形继承
├── 019_operator_overload/  # 运算符重载
├── 020_friend_function/    # 友元函数
├── 021_explicit_ctor/      # explicit 构造函数
├── 022_mutable_member/     # mutable 成员
├── 023_typeid_rtti/        # typeid 和 RTTI
├── 024_template_function/  # 函数模板
├── 025_template_class/     # 类模板
├── 026_template_specialization/ # 模板特化
├── 027_template_instantiation/  # 模板实例化
├── 028_variadic_template/  # 可变参数模板
├── 029_unique_ptr/         # unique_ptr
├── 030_shared_ptr/         # shared_ptr
├── 031_custom_deleter/     # 自定义删除器
├── 032_placement_new/      # placement new
├── 033_raii_pattern/       # RAII 模式
├── 034_vector_basic/       # vector 基础
├── 035_map_basic/          # map 基础
├── 036_string_basic/       # string 基础
├── 037_array_basic/        # array 基础
├── 038_tuple_basic/        # tuple 基础
├── 039_lambda_basic/       # lambda 基础
├── 040_std_function/       # std::function
├── 041_functional_bind/    # std::bind
├── 042_exception_basic/    # 异常处理
├── 043_namespace_nested/   # 嵌套命名空间
├── 044_enum_class/         # 强类型枚举
├── 045_union_basic/        # 共用体
├── 046_constexpr_basic/    # constexpr
├── 047_noexcept_basic/     # noexcept
└── 048_summary/            # 本汇总
```

## 示例分类

| 范围 | 主题 | 说明 |
|------|------|------|
| 001-005 | 基础函数 | 函数、重载、默认参数、inline、可变参数 |
| 006-012 | 类基础 | 构造、拷贝、移动、静态/const/volatile 成员 |
| 013-018 | 继承与多态 | 单/多继承、虚函数、纯虚函数、菱形继承 |
| 019-023 | 运算符与特殊成员 | 运算符、友元、explicit、mutable、RTTI |
| 024-028 | 模板 | 函数模板、类模板、特化、实例化、参数包 |
| 029-033 | 智能指针与内存 | unique_ptr、shared_ptr、自定义删除器、RAII |
| 034-038 | STL 容器 | vector、map、string、array、tuple |
| 039-042 | 函数对象与异常 | lambda、std::function、bind、异常处理 |
| 043-048 | 其他高级特性 | namespace、enum class、union、constexpr、noexcept、汇总 |

## C++ 特性

本示例展示一个最小但完整的收尾绑定：`summary_ns::Counter` 保存对象内状态，`safe_add` 与 `max_size` 是命名空间自由函数。跨 FFI 只交换 `int` 标量；对象通过 `std::unique_ptr<summary_ns::Counter> hicc::make_unique<...>()` 构造，析构由 Rust `Drop` 自动完成。

## C++ 代码

### summary.h

```cpp
namespace summary_ns {

class Counter {
    int count_;
public:
    Counter() : count_(0) {}
    void increment() { ++count_; }
    void decrement() { --count_; }
    int get() const { return count_; }
    void reset() { count_ = 0; }
};

int safe_add(int a, int b);
int max_size();
int summary_anchor();

} // namespace summary_ns
```

### summary.cpp

```cpp
namespace summary_ns {

int safe_add(int a, int b) { return a + b; }
int max_size() { return 1024; }
int summary_anchor() { return 0; }

} // namespace summary_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定命名空间类、`make_unique` 工厂与自由函数：

```rust
hicc::cpp! {
    #include "summary.h"
}

hicc::import_class! {
    #[cpp(class = "summary_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);
        #[cpp(method = "void decrement()")]
        pub fn decrement(&mut self);
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;
        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "summary"]

    #[cpp(func = "std::unique_ptr<summary_ns::Counter> hicc::make_unique<summary_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "int summary_ns::safe_add(int, int)")]
    pub fn safe_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int summary_ns::max_size()")]
    pub fn max_size() -> i32;
}
```

## FFI 对比分析

| 方面 | 旧 extern-C shim | hicc 直出 |
|------|------------------|-----------|
| 类绑定 | `struct Counter*` + `counter_new/delete` | `summary_ns::Counter` 直接导入 |
| 构造 | 手写 C shim 工厂 | `hicc::make_unique` 工厂 |
| 析构 | 手写 `*_delete` | Rust `Drop` 自动触发 |
| 命名空间 | shim 中展平 | Rust 注解写完整 C++ 名称 |
| 参数/返回 | 标量或指针 | 本例仅交换 `int` 标量 |

## 运行结果

```
=== 048_summary - 示例系列汇总（hicc 直出）===

initial=0
after increment x3=3
after decrement=2
after reset=0
safe_add(2,3)=5
max_size()=1024

Rust FFI: hicc 直接绑定命名空间类与自由函数，无需 extern-C shim
```

## FFI 模式总结

1. C++ 异常不能跨 FFI 边界传播，应在 C++ 侧转换为错误码或受控结果。
2. 类对象可通过 hicc `import_class!` 直接绑定，减少不透明指针 shim。
3. 命名空间保留在 C++ 类型/函数签名中，Rust 注解写全限定名。
4. enum class、union、constexpr、noexcept 等高级特性应在 C++ 侧保持语义，跨边界优先交换标量或 `const char*`。
5. 资源所有权优先交给 `std::unique_ptr` 与 Rust `Drop` 管理。

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_counter_state_is_per_object` | `Counter::new`、increment x3、decrement、reset |
| `smoke_free_functions` | `safe_add(2,3)==5`、`max_size()==1024` |

### 运行方式

```bash
bash examples/048_summary/cpp/standalone.sh
cd examples/048_summary/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 学习路径

1. **入门**：`001_hello_world` -> `006_class_basic`
2. **进阶**：`013_inheritance_single` -> `023_typeid_rtti`
3. **模板**：`024_template_function` -> `028_variadic_template`
4. **内存**：`029_unique_ptr` -> `033_raii_pattern`
5. **STL**：`034_vector_basic` -> `040_std_function`
6. **高级**：`041_functional_bind` -> `047_noexcept_basic`
7. **收尾**：`048_summary` 回顾全系列并展示去 shim 直出写法

## 总结

- 本示例保留汇总全系列的 README 精神，同时将代码迁移为 hicc shimless direct binding
- `Counter` 作为命名空间类直接导入，构造经 `make_unique`，析构由 Rust `Drop` 自动完成
- `safe_add` / `max_size` 作为命名空间自由函数直出绑定，无需 extern-C 包装层

## 许可

本项目仅供学习参考。
