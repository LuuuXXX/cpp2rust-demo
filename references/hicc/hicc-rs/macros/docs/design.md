# `#[export_class]` & `#[export_lib]` 过程宏设计文档

## 概述

`#[export_class]` 和 `#[export_lib]` 是基于 hicc-rs 库的 Rust 属性宏(attribute macro),
将 Rust 类型的方法或全局函数自动包装为 C 兼容的 FFI 接口,减少手工编写样板代码的工作量.

## 宏总览

| 宏 | 作用 | 适用场景 |
|----|------|---------|
| `#[export_class]` | 将 Rust 类型的 `impl` 块方法包装为 C 函数 | 导出面向对象的接口 |
| `#[export_lib]` | 将 Rust 全局函数包装为 C 函数库 | 导出过程式函数库 |

## `#[export_class]` 使用场景

### 1. 基本用法

```rust
#[export_class]
impl MyType {
    fn method(&self) -> i32;
}
```

### 2. 导出到模块 (mod grouping)

```rust
#[export_class]
mod ffi {
    impl Foo { ... }
    impl Bar { ... }
}
```

### 3. 泛型参数

#### 3a. 单泛型参数

```rust
#[export_class]
impl<T> MyContainer<T> {
    fn take(self) -> T;
    fn get_ptr(&self) -> *const T;
}
```

#### 3b. 多泛型参数

```rust
#[export_class]
impl<T, U, V> Multi<T, U, V> {
    fn get_first(&self) -> *const T;
    fn get_second(&self) -> *const U;
    fn count(&self) -> i32;
}
```

#### 3c. 部分泛型参数未在方法中使用

```rust
#[export_class]
impl<T, U, V> Foo<T, U, V> {
    fn get_first(&self) -> *const T;
}
```

### 4. 深度分组 (Depth Group)

| 分组 | 深度 | 示例签名 | 约束 | 样例 |
|------|------|---------|------|------|
| A | 0 | `fn take(self) -> T` | 无 | `group_a` |
| B | 1 | `fn get_ptr(&self) -> *const T` | `T::Depth: Depth0_3` | `group_b` |
| C | 2 | `fn get_ptr_ptr(&self) -> *const *const T` | `T::Depth: Depth0_2` | `group_c` |
| D | 3 | `fn get_ptr3(&self) -> *const *const *const T` | `T::Depth: Depth0_1` | `group_d` |
| E | 4 | `fn get_ptr4(&self) -> *const *const *const *const T` | `T::Depth: Depth0_0` | `group_e` |

深度 5+ 报告编译错误。

### 5. 生命周期参数

```rust
#[export_class]
impl<'a, T> Ref<'a, T> {
    fn get_val(&self) -> i32;
}
```

### 6. 类型参数约束 (Bounds)

#### 6a. Where 子句

```rust
#[export_class]
impl<T, U, V> Foo<T, U, V>
where T: ::std::fmt::Debug, U: 'static, V: ::std::hash::Hash + 'static
{ fn get_t(&self) -> i32; }
```

#### 6b. 内联约束

```rust
#[export_class]
impl<T: ::std::fmt::Debug, U: 'static, V> Foo<T, U, V> { ... }
```

### 7. 路径类型

```rust
#[export_class]
impl foo::bar::Bar { fn method(&self) -> i32; }
```

生成 `foo_bar_BarClass`, `foo_bar_BarMethods` 等唯一标识符。

### 8. 方法体处理

- **无方法体**: 自动生成 `from_abi`/`into_abi` 包装,调用原始方法。
- **有方法体**: 将 `self` 替换为转换变量,在方法体外部包装转换。

### 9. 禁止泛型函数

```rust
#[export_class]
impl Foo { fn bar<T>(&self) -> T; }  // 编译错误
```

### 10. `in_hicc` 属性

```rust
#[export_class(in_hicc)]
impl MyType { ... }
```

将 `::hicc_rs::` 替换为 `crate::`。

## `#[export_lib]` 使用场景

### 1. 基本用法

```rust
#[export_lib(export_name = "get_ffi")]
mod ffi {
    fn my_function(val: &Option<i32>) -> bool;
}
```

### 2. 无方法体 (声明)

适配函数通过 `crate::function_name()` 调用外部实现。

### 3. 有方法体 (自定义)

适配函数直接使用方法体中的代码。

### 4. `in_hicc` 属性

```rust
#[export_lib(in_hicc, export_name = "get_ffi")]
mod ffi { ... }
```

## 生成代码结构

### `export_class` 展开

生成: ValueType impl + 适配器包装结构体 + C 适配函数 + MethodArray + Methods 结构体 + new_methods + ClassMethods 实现。
当存在深度 >0 方法时额外生成特化 trait 和多级 const METHODS。

### `export_lib` 展开

生成结构保留在原始 mod 内: 函数指针结构体 + 适配函数 + const METHODS + `#[no_mangle]` 入口函数。

## 示例项目一览

| 项目 | 测试场景 | 关键特征 |
|------|---------|---------|
| `group_a` | 深度 0 (A 组) | `fn take(self) -> T` 值类型 |
| `group_b` | 深度 1 (B 组) | `fn get_ptr(&self) -> *const T` |
| `group_c` | 深度 2 (C 组) | `fn get_ptr_ptr(&self) -> *const *const T` |
| `group_d` | 深度 3 (D 组) | `fn get_ptr3(&self) -> *const *const *const T` |
| `group_e` | 深度 4 (E 组) | `fn get_ptr4(&self) -> *const *const *const *const T` |
| `multi_param` | 多泛型参数 | `T, U, V` 全部被方法使用 |
| `multi_param_unused` | 部分泛型未使用 | `V` 未被任何方法引用 |
| `lifetime_param` | 生命周期参数 | `'a, T` 在泛型中 |
| `bounded_params` | 简单类型约束 | `T: Send` |
| `bounded_generics` | Where 子句 | where `T: Debug, U: 'static, V: Hash+'static` |
| `where_clause` | Where 子句(交替语法) | 同上,使用 where 而非内联 |
| `depth_lifetime` | 深度+生命周期 | 深度 1 + 生命周期组合 |
| `export_lib` | export_lib 全功能 | 声明、自定义体、多种参数类型 |
| `simple_demo` | 基本场景(旧式) | 单方法非泛型 |
| `option_demo` | 泛型+深度(旧式) | 深度 0/1 混合 + 泛型 |

## 注意事项

1. **禁止泛型方法**: `fn foo<T>(&self) -> T` 不允许
2. **深度限制**: 引用/指针嵌套深度超过 4 层报错
3. **生命周期**: 保留但不添加 `ValueType` 约束
4. **路径类型**: `::` 在路径中转为 `_` 用于标识符
5. **传递项**: export_lib 中非函数项保留在 mod 内
6. **外部依赖**: 通过 `hicc_rs` 重导出,外部只需依赖 `hicc-rs`

## 返回值引用类型与生命周期

当前宏生成的 Methods 结构体和 extern "C" 适配函数**不支持方法返回值中的匿名生命周期**（如 `fn get(&self) -> &T`）。
这是因为 `extern "C"` 函数指针类型不能包含泛型生命周期参数,而 Methods 结构体中的函数指针类型无法表达返回类型 `&T` 中匿名生命周期的约束。

### 推荐做法:使用裸指针替代引用

对于需要返回引用的方法,使用裸指针 `*const T` / `*mut T` 替代 `&T` / `&mut T`:

```rust
#[export_class]
impl<T> Container<T> {
    fn get_ptr(&self) -> *const T;   // ✅ 支持
    // fn get_ref(&self) -> &T;      // ❌ 不支持(匿名生命周期)
}
```

`*const T` 是深度 1 (Group B),适配函数签名中不需要生命周期参数,可正确生成。

### 实现原理

生成代码中的适配器函数采用 `unsafe extern "C"` 签名。当返回类型包含引用时:

- `&T` 中的匿名生命周期无法在 `extern "C" fn(...)` 函数签名中表达
- 在 extern "C" 函数体中使用 `fn<'a>(...) -> ...` 语法虽然允许,但函数指针类型无法包含 `'a` 参数
- 因此 Methods 结构体中的字段类型无法写入引用返回类型

### 未来扩展

如果需要在 Methods 结构体中支持引用返回类型,可考虑:

1. 使用 HRTB: `for<'a> unsafe extern "C" fn(...) -> ...`
2. 在适配函数中手动添加生命周期参数 `<'a>`,输出时通过 turbofish 或类型标注确定
3. 将引用转换为指针存储在 Methods 中

当前优先级下,推荐在公开接口中使用裸指针。
