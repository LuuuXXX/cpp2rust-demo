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

```

struct MyTypeMethods {
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

#### 5a. 带生命周期的 impl

```rust
#[export_class]
impl<'a, T> Slice<'a, T> {
    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> &T;
}
```

生命周期 `'a` 会保留在生成的 Methods 结构体泛型参数中（因为字段类型引用了 `self_type`），
但不会保留在包装类（`XxxClass`）结构体中（该类字段不涉及生命周期）。

当方法返回引用类型（如 `&T`、`&mut T`）时，`fn_ptr_type_with_lt` 函数会自动将
`&T` 转换为 `&'a T`，其中 `'a` 来自 impl 的已有生命周期或由 `needs_lt` 注入。

#### 5b. 仅有生命周期参数（无类型参数）

```rust
#[export_class]
impl<'a> Any<'a> {
    fn type_id(&self) -> [u8; 16];
}
```

此时 `st_arg_idents` 为空（因为没有类型参数），`class_ty()` 辅助函数
处理为输出 `AnyClass::new_methods()` 而不是 `AnyClass<>::new_methods()`。

#### 5c. 匿名生命周期 `'_`

```rust
#[export_class]
impl AnyMut<'_> {
    fn type_id(&self) -> [u8; 16];
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

#### 6c. 关联类型约束

```rust
#[export_class]
impl<T: ValueType<Flag1 = IsClass>, const N: usize> Array<T, N> {
    fn get(&self, idx: usize) -> &T;
}
```

原始约束（如 `Flag1 = IsClass`）会被保留在生成的泛型参数中，
同时通过 `value_where` 函数额外添加 `ValueType` 约束作为 where 子句。

### 7. 路径类型

```rust
#[export_class]
impl foo::bar::Bar { fn method(&self) -> i32; }
```

生成 `foo_bar_BarClass`, `foo_bar_BarMethods` 等唯一标识符。

### 8. 常量泛型参数

```rust
#[export_class]
impl<T: ValueType<Flag1 = IsClass>, const N: usize> Array<T, N> {
    fn len(&self) -> usize { N }
    fn get(&self, idx: usize) -> &T;
    fn set(&mut self, idx: usize, val: T);
}
```

常量泛型参数（`const N: usize`）会被保留在 `all_arg_idents` 中，
参与类型参数位置的展开，确保生成正确的 `ArrayClass<T, N>` 和 `ArrayMethods<'a, T, N>`。

### 9. 方法体处理

- **无方法体**：自动生成 `from_abi`/`into_abi` 包装，调用原始方法。
- **有方法体**：将 `self` 替换为转换变量，在方法体外部包装转换。
  `self.0`（元组字段访问）也会被正确替换为 `obj`。

### 10. panic!() 方法的特殊处理

当方法体仅包含 `panic!()` 调用时（如 `{ panic!(); }`），宏检测到 panic 方法体后：

1. 在 Methods 结构体中将该字段标记为 `Option<fn ptr>`（即使深度为 0）
2. 在 specialize 方法的 default/specialized 数组中都使用 `0 as *const ()`（空指针）
3. 调用侧可以通过 `.is_none()` / `.is_some()` / `.unwrap()` 检查方法可用性

这用于处理不可变类型的 `&mut self` 方法（如 `&[T]` 的 `set` 方法）。

### 11. 禁止泛型函数

```rust
#[export_class]
impl Foo { fn bar<T>(&self) -> T; }  // 编译错误
```

### 12. `in_hicc` 属性

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

生成: ValueType impl + 包装结构体 + C 适配函数 + MethodArray + Methods 结构体 + new_methods + ClassMethods 实现。

当存在深度 >0 方法时额外生成特化 trait 和多级 const METHODS。
当方法返回引用时，Methods 结构体加入 `'a` 生命周期参数。

### `export_lib` 展开

生成结构保留在原始 mod 内: 函数指针结构体 + 适配函数 + const METHODS + `#[no_mangle]` 入口函数。

## 实现细节

### `replace_semicolons` 的 `brace_depth` 追踪

`replace_semicolons` 函数将方法声明中的 `;` 替换为 `{}`（空方法体）。
最初实现会递归替换所有 `;`，导致方法体内的表达式（如 `[u8; 128]` 数组类型中的 `;`）
也被替换为 `{}`，产生 `[u8 {} 128]` 导致解析失败。

使用 `brace_depth` 追踪大括号嵌套深度：
- `brace_depth == 1`：直接位于 impl 块内 → 替换 `;`
- `brace_depth >= 2`：在方法体内或其它嵌套块内 → 不替换
- `in_non_brace_group == true`：在 `[]` 或 `()` 或 `<>` 内 → 不替换

### 生命周期与引用返回

当方法返回引用类型（如 `fn get(&self) -> &T`）时：

1. `needs_lt` 为 `true`，Methods 结构体增加 `'a` 生命周期参数
2. `fn_ptr_type_with_lt` 被调用，其内部 `add_lt_to_type` 将 `&T` 转换为 `&'a T`
3. 适配器函数通过 `use_lt` 标志在参数类型中显式使用 `&'a self_type`，
   使返回的 `&'a T` 的生命周期正确约束
4. `T: 'a` 边界被自动添加到 Methods 结构体的 where 子句中

适配器函数参数类型使用 `&'a #self_type`（而非 `&#self_type`），
确保 `'a` 显式出现在函数签名中，使编译器能正确追踪返回引用生命周期。

### 包装类（Wrapper Class）的生命周期处理

包装类（`XxxClass`）仅包含类型字段，不包含生命周期。因此 `gen_wrapper`
会过滤掉 `GenericParam::Lifetime`，避免编译器警告 `'a` 未使用。

适配器函数的 impl 块使用 `generics.params`（包含生命周期），
但类型参数位置使用 `st_arg_idents`（不包含生命周期）。

### `class_ty` 辅助函数

```rust
fn class_ty(class_ident: &Ident, arg_idents: &[TS2]) -> TS2
```

- 当 `arg_idents` 非空时：生成 `ClassName<arg1, arg2>`
- 当 `arg_idents` 为空时：生成 `ClassName`（不带 `<>`）

用于避免在无泛型参数时生成 `ClassName<>` 这种不合法的语法。

### Turbofish 语法

在 ClassMethods 实现的 const 表达式中，使用 turbofish 语法：

```rust
// gen_class_simple:
#class_ident::<#(#arg_idents),*>::new_methods()
```

而非 `#ct::new_methods()`（其中 `#ct = ResultClass<T, E>`），
因为 `<T, E>` 在表达式上下文中会被解析为比较操作符。

### MethodArray 唯一命名

`MethodArray` 现在包含类型名前缀以避免冲突：

```rust
let array_ident: Ident = format_ident!("{}MethodArray", type_ident);
```

当同一个模块中有多个 `#[export_class]` 块时（如 `Slice` 和 `SliceMut`），
生成 `SliceMethodArray` 和 `SliceMutMethodArray` 而非冲突的 `MethodArray`。

### 多个 `#[export_class]` 块在相同模块中

多个 `#[export_class]` 块可以共存于同一模块（如 `slice.rs`），
两者互不干扰。生成的 ValueType impl、包装类、Methods 等所有项
都使用唯一的名称（基于 `type_ident`）。

## 生成字段的 `Option` 包装规则

Methods 结构体中方法字段的 `Option` 包装规则：

| 条件 | 包装为 |
|------|--------|
| 深度 > 0 | `Option<fn ptr>` |
| 深度 == 0 且方法体为 `panic!()` | `Option<fn ptr>` |
| 其他 | 普通 `fn ptr` |

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

## 内部类型映射

| 生成项 | 命名规则 | 示例 |
|--------|---------|------|
| ValueType impl | `impl<...> ValueType for MyType` | `impl<T> ValueType for Box<T>` |
| 包装类 | `{Type}Class` | `BoxClass<T>`，`SliceClass` |
| 适配函数 | 方法同名 | `len`，`get` |
| MethodArray | `{Type}MethodArray` | `BoxMethodArray`，`SliceMethodArray` |
| Methods 结构体 | `{Type}Methods` | `BoxMethods<'a, T>`，`ArrayMethods<'a, T, N>` |
| 特化 trait | `{Type}ClassMethods` | `BoxClassMethods` |
| ClassMethods impl | `impl<...> ClassMethods for MyType` | `impl<T> ClassMethods for Box<T>` |

## 注意事项

1. **禁止泛型方法**: `fn foo<T>(&self) -> T` 不允许
2. **深度限制**: 引用/指针嵌套深度超过 4 层报错
3. **生命周期**: 保留在 Methods 结构体中，但过滤出包装类
4. **路径类型**: `::` 在路径中转为 `_` 用于标识符
5. **传递项**: export_lib 中非函数项保留在 mod 内
6. **外部依赖**: 通过 `hicc_rs` 重导出,外部只需依赖 `hicc-rs`
7. **引用返回**: 支持 `&T` 和 `&mut T` 返回，通过 `'a` 生命周期处理
8. **常量泛型**: 支持 `const N: usize` 等泛型参数
9. **关联类型约束**: 保留 `ValueType<Flag1 = IsClass>` 等原始约束
10. **`panic!()` 方法体**: 自动视为不可用，在数组中设为空指针
11. **同一模块多个 export_class**: 使用唯一 MethodArray 命名避免冲突
12. **`[u8; N]` 类型**: 方法体中不能直接 `core::mem::transmute` 不同大小类型
