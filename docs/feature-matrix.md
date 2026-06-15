# C++ 特性支持矩阵

> 图例：✅ 完全自动生成可编译代码　⚠️ 降级生成 + 内联 TODO（代码仍可 `cargo check`）
> 平台列：Linux = Linux（GCC/Clang）；macOS = macOS（Apple Clang）；Win = Windows（MinGW/MSVC）
> `¹` 标注：Linux/Win 自动引入 `hicc-std`；macOS 仅 wrapper 类方式

---

## 完整特性矩阵

| 示例 | 类别 | C++ 特性 | 状态 | 平台 | FFI 策略 |
|------|------|---------|------|------|---------|
| [001_hello_world](../examples/001_hello_world) | 基础函数 | extern "C" 函数 | ✅ | 全平台 | AST 直接提取 → `import_lib!` |
| [002_function_overload](../examples/002_function_overload) | 基础函数 | 函数重载 | ✅ | 全平台 | 名称加类型后缀（`_i32`/`_f64`） |
| [003_default_args](../examples/003_default_args) | 基础函数 | 默认参数 | ✅ | 全平台 | C++ 侧展开为多个固定参数重载 |
| [004_inline_functions](../examples/004_inline_functions) | 基础函数 | inline 函数 | ✅ | 全平台 | 函数体从 `.cpp2rust` 提取，写入 `hicc::cpp!` |
| [005_variadic_functions](../examples/005_variadic_functions) | 基础函数 | C 可变参数（`...`） | ⚠️ `[CV]` | 全平台 | `...` 函数整体跳过 |
| [006_class_basic](../examples/006_class_basic) | 类与对象 | 基础类 | ✅ | 全平台 | opaque pointer + `import_class!` |
| [007_class_constructor](../examples/007_class_constructor) | 类与对象 | 构造/析构 | ✅ | 全平台 | `*_new()` / `*_delete()` shim |
| [008_class_copy](../examples/008_class_copy) | 类与对象 | 拷贝构造 | ✅ | 全平台 | `*_copy()` shim |
| [009_class_move](../examples/009_class_move) | 类与对象 | 移动构造 | ✅ | 全平台 | `*_move()` shim |
| [010_class_static](../examples/010_class_static) | 类与对象 | 静态成员 | ✅ | 全平台 | getter/setter shim |
| [011_class_const](../examples/011_class_const) | 类与对象 | const 成员函数 | ✅ | 全平台 | 映射为 `fn method(&self)` |
| [012_class_volatile](../examples/012_class_volatile) | 类与对象 | volatile 成员函数 | ⚠️ `[VM]` | 全平台 | `volatile this` 方法移除 |
| [013_inheritance_single](../examples/013_inheritance_single) | 面向对象 | 单继承 | ✅ | 全平台 | 基类方法在子类中提升 |
| [014_inheritance_multiple](../examples/014_inheritance_multiple) | 面向对象 | 多继承 | ✅ | 全平台 | 多条继承链展开 |
| [015_virtual_basic](../examples/015_virtual_basic) | 面向对象 | 虚函数 | ✅ | 全平台 | hicc 处理虚表 dispatch |
| [016_virtual_pure](../examples/016_virtual_pure) | 面向对象 | 纯虚/抽象类 | ✅ | 全平台 | 只生成前向声明 |
| [017_virtual_override](../examples/017_virtual_override) | 面向对象 | override | ✅ | 全平台 | 与普通虚函数相同 |
| [018_virtual_diamond](../examples/018_virtual_diamond) | 面向对象 | 菱形继承 | ✅ | 全平台 | 命名 shim 避免指针调整 |
| [019_operator_overload](../examples/019_operator_overload) | 运算符/类型 | 运算符重载 | ⚠️ `[OP]` | 全平台 | 命名 shim + `import_lib!` |
| [020_friend_function](../examples/020_friend_function) | 运算符/类型 | 友元函数 | ✅ | 全平台 | 提取为普通函数 |
| [021_explicit_ctor](../examples/021_explicit_ctor) | 运算符/类型 | explicit 构造 | ✅ | 全平台 | 对 FFI 透明 |
| [022_mutable_member](../examples/022_mutable_member) | 运算符/类型 | mutable 成员 | ✅ | 全平台 | 对 FFI 透明 |
| [023_typeid_rtti](../examples/023_typeid_rtti) | 运算符/类型 | typeid/RTTI | ✅ | 全平台 | 注入枚举 + `getType()` |
| [024_template_function](../examples/024_template_function) | 模板实例化 | 函数模板 | ✅ | 全平台 | 为实例化版本生成 C 包装 |
| [025_template_class](../examples/025_template_class) | 模板实例化 | 类模板 | ✅ | 全平台 | 只处理实例化的具体类型 |
| [026_template_specialization](../examples/026_template_specialization) | 模板实例化 | 模板偏特化 | ✅ | 全平台 | 视为实例化路径之一 |
| [027_template_instantiation](../examples/027_template_instantiation) | 模板实例化 | 显式实例化 | ✅ | 全平台 | 按普通类处理 |
| [028_variadic_template](../examples/028_variadic_template) | 模板实例化 | 可变参数模板 | ⚠️ `[VA]` | 全平台 | wrapper 类 + 按数量展开 |
| [029_unique_ptr](../examples/029_unique_ptr) | 智能指针 | std::unique_ptr | ✅ | 全平台 | opaque pointer |
| [030_shared_ptr](../examples/030_shared_ptr) | 智能指针 | std::shared_ptr | ✅ | 全平台 | `*_clone()`/`*_delete()` shim |
| [031_custom_deleter](../examples/031_custom_deleter) | 智能指针 | 自定义删除器 | ✅ | 全平台 | 删除器注入 `hicc::cpp!` |
| [032_placement_new](../examples/032_placement_new) | 智能指针 | placement new | ✅ | 全平台 | `*_placement_new()` shim |
| [033_raii_pattern](../examples/033_raii_pattern) | 智能指针 | RAII | ✅ | 全平台 | 析构 → `*_delete()` shim |
| [034_vector_basic](../examples/034_vector_basic) | STL 容器 | std::vector | ✅ | 全平台¹ | wrapper 类 |
| [035_map_basic](../examples/035_map_basic) | STL 容器 | std::map | ✅ | 全平台¹ | wrapper 类 |
| [036_string_basic](../examples/036_string_basic) | STL 容器 | std::string | ✅ | 全平台¹ | wrapper 类 |
| [037_array_basic](../examples/037_array_basic) | STL 容器 | std::array | ✅ | 全平台¹ | wrapper 类 |
| [038_tuple_basic](../examples/038_tuple_basic) | STL 容器 | std::tuple | ✅ | 全平台¹ | wrapper 类 |
| [039_lambda_basic](../examples/039_lambda_basic) | 函数对象 | Lambda | ⚠️ `[LM]` | 全平台 | 函数指针或 class wrapper |
| [040_std_function](../examples/040_std_function) | 函数对象 | std::function | ⚠️ `[LM]` | 全平台 | class wrapper + opaque pointer |
| [041_functional_bind](../examples/041_functional_bind) | 函数对象 | std::bind | ✅ | 全平台 | 同有状态 lambda 策略 |
| [042_exception_basic](../examples/042_exception_basic) | 函数对象 | C++ 异常 | ✅ | 全平台 | try/catch → 错误码 |
| [043_namespace_nested](../examples/043_namespace_nested) | 高级特性 | 嵌套命名空间 | ✅ | 全平台 | 函数名前缀扁平化 |
| [044_enum_class](../examples/044_enum_class) | 高级特性 | enum class | ✅ | 全平台 | 枚举值导出为 Rust `const` |
| [045_union_basic](../examples/045_union_basic) | 高级特性 | union | ✅ | 全平台 | opaque pointer + getter/setter |
| [046_constexpr_basic](../examples/046_constexpr_basic) | 高级特性 | constexpr | ✅ | 全平台 | 常量读取 → Rust `const` |
| [047_noexcept_basic](../examples/047_noexcept_basic) | 高级特性 | noexcept | ✅ | 全平台 | 对 FFI 透明 |
| [048_summary](../examples/048_summary) | 高级特性 | 综合 FFI | ✅ | 全平台 | 所有策略组合 |
| [049_direct_class_basic](../examples/049_direct_class_basic) | Direct Binding | 直接绑定类 | ✅ | 全平台 | `import_class!` + `import_lib!` |

---

## 降级特性详解

| TAG | 示例 | C++ 特性 | 根本原因 | 自动降级策略 | 用户剩余工作 |
|-----|------|---------|---------|------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号 | 生成命名 shim（`{class}_add` 等）写入 `hicc::cpp!` + `import_lib!` | 手动实现 `impl std::ops::Add<T> for T` 等 |
| `[VA]` | 028 | 可变参数模板 | `...Args` 编译期展开，FFI 无法表达任意参数数 | 生成 wrapper 类 + 按参数数量组合静态方法 | 按需添加新参数组合 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态→函数指针；有状态→class wrapper + opaque pointer | 编写 trampoline |
| `[LM]` | 040 | std::function | 类型擦除容器，捕获状态不透明 | class wrapper + opaque pointer | 编写 Rust 闭包适配层 |
| `[CV]` | 005 | C 可变参数函数 | `...` 参数在运行时按 `va_list` 访问 | 含 `...` 函数整体跳过 | 补充固定参数 wrapper |
| `[FP]` | 039, 040 | 函数指针参数 | C++ 成员函数指针无法映射为 Rust FFI 类型 | C 函数指针 → `unsafe extern "C" fn(...)` + `[FP]` 注释 | 确认 `extern "C"` 调用约定 |
| `[VM]` | 012 | volatile 成员函数 | `volatile this` 在 Rust 无对应语义 | volatile 方法从 `import_class!` 移除 | 检查 `import_lib!` 中 volatile shim |
| `[LONG_DOUBLE]` | — | `long double` | x86-64 80 位扩展浮点，Rust 无原生对应 | 降级映射为 `f64`（精度损失）+ `[LONG_DOUBLE]` 注释 | 引入 `f128`/`rug` crate 或手动桥接 |

---

## 类型映射注意事项

| C++ 类型 | Rust 映射 | 注意事项 |
|---------|----------|---------|
| `long double` | `f64` | 精度损失（80→64 位），标注 `[LONG_DOUBLE]` |
| `T&`（左值引用） | `&mut T` | 生命周期由调用方管理 |
| `const T&` | `&T` | 同上 |
| `void*` | `*mut u8` | 建议通过 hicc `import_class!` 封装 |
| `T[N]` | `*mut T` | C 数组退化为指针，元素数量信息丢失 |

---

## 学习路径

```
入门：001 → 002 → 003 → 004 → 005 → 006
类与对象：007 → 008 → 009 → 010 → 011 → 012
面向对象：013 → 014 → 015 → 016 → 017 → 018
运算符：019 → 020 → 021 → 022 → 023
模板：024 → 025 → 026 → 027 → 028
内存：029 → 030 → 031 → 032 → 033
STL：034 → 035 → 036 → 037 → 038
函数对象：039 → 040 → 041 → 042
高级：043 → 044 → 045 → 046 → 047 → 048
```
