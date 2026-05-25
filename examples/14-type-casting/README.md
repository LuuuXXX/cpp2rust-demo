# 示例 14：类型转换

## 特性概述

本示例展示 C++ 的四种**类型转换**运算符：`static_cast`、`dynamic_cast`、`const_cast` 和 `reinterpret_cast`。每种转换在 Rust FFI 中有不同的处理策略，其中 `dynamic_cast` 需要特殊的运行时辅助支持。

## C++ 特性说明

| 转换类型 | 说明 | 安全性 |
|----------|------|--------|
| `static_cast<T>` | 编译期类型转换，含隐式转换 | 编译期检查 |
| `dynamic_cast<T*>` | 运行时多态类型检查 | 运行时（可返回 null） |
| `const_cast<T>` | 移除或添加 `const` 限定符 | 编译期（可能引发 UB） |
| `reinterpret_cast<T>` | 位级重新解释 | 极度不安全 |

### 代码结构

```cpp
class Base { virtual int get_value() const; };
class Derived : public Base { int get_derived_value() const; };

// 各类型转换
int static_cast_example(int x);
double static_cast_double(int x);
Derived* to_derived(Base* p);      // dynamic_cast
int* const_cast_example(const int* p);  // const_cast
intptr_t reinterpret_intptr(int* p);    // reinterpret_cast
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `CStyleCastExpr` / `ImplicitCastExpr` | 隐式转换 |
| `CXXStaticCastExpr` | `static_cast` |
| `CXXDynamicCastExpr` | `dynamic_cast` |
| `CXXConstCastExpr` | `const_cast` |
| `CXXReinterpretCastExpr` | `reinterpret_cast` |

## hicc 处理方式

### `static_cast` → Rust `as` 运算符

基础类型的 `static_cast` 对应 Rust 的 `as` 运算符：

```rust
// C++: static_cast<int>(x * 1.5)
let result = (x as f64 * 1.5) as i32;

// C++: static_cast<double>(x)
let result = x as f64;
```

对于 C++ 类型转换，通过函数接口映射：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int static_cast_example(int)")]
    fn static_cast_example(x: i32) -> i32;

    #[cpp(func = "double static_cast_double(int)")]
    fn static_cast_double(x: i32) -> f64;
}
```

### `dynamic_cast` → `@dynamic_cast` 内置函数

`dynamic_cast` 需要运行时类型信息（RTTI），hicc 通过 `@dynamic_cast` 内置函数提供支持：

```rust
hicc::import_class! {
    #[cpp(class = "Base")]
    class Base {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }

    #[cpp(class = "Derived")]
    class Derived: Base {
        #[cpp(method = "int get_derived_value() const")]
        fn get_derived_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Base;
    class Derived;

    // @dynamic_cast 内置函数：Base* → Derived*
    #[cpp(func = "Derived* @dynamic_cast<Derived*>(Base*)")]
    fn base_to_derived(p: &Base) -> ClassMutPtr<'_, Derived>;
}

fn main() {
    let base = create_base();
    let derived = base_to_derived(&base);
    if !derived.is_null() {
        println!("derived value = {}", derived.get_derived_value());
    } else {
        println!("cast failed: not a Derived");
    }
}
```

cpp2rust-demo 会在 `meta/dynamic_casts.rs` 中自动生成 `@dynamic_cast` 辅助函数的脚手架。

### `const_cast` → `unsafe` 裸指针转换

`const_cast` 移除 `const` 限定，在 Rust 中需要 `unsafe`：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int* const_cast_example(const int*)")]
    fn const_cast_example(p: *const i32) -> *mut i32;
}

// Rust 侧对应：
unsafe fn remove_const(p: *const i32) -> *mut i32 {
    p as *mut i32
}
```

### `reinterpret_cast` → `unsafe` 转换

`reinterpret_cast` 是最危险的转换，在 Rust 中需要 `unsafe` 代码块：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "intptr_t reinterpret_intptr(int*)")]
    unsafe fn reinterpret_intptr(p: *mut i32) -> isize;
}

// Rust 侧对应：
let ptr: *mut i32 = ...;
let addr = ptr as isize;  // 等价的 Rust 写法
```

### 动态转换辅助模块

cpp2rust-demo 会在 `rust/src/free/dynamic_casts.rs` 中生成动态转换辅助绑定：

```rust
// auto-generated dynamic_casts.rs
hicc::import_lib! {
    #![link_name = "example"]

    class Base;
    class Derived;

    #[cpp(func = "Derived* @dynamic_cast<Derived*>(Base*)")]
    fn dynamic_cast_base_to_derived(p: &Base) -> ClassMutPtr<'_, Derived>;
}
```

## 注意事项

1. **`dynamic_cast` RTTI 依赖**：需要启用 RTTI（`-frtti`，即默认选项），禁用 RTTI 时 `dynamic_cast` 不可用
2. **`const_cast` 安全性**：移除 `const` 后修改原本不可变的数据会导致未定义行为（UB），Rust 侧需标记 `unsafe`
3. **`reinterpret_cast` 限制**：只有在明确了解内存布局时才能安全使用，hicc 建议封装为 `unsafe fn`
4. **`static_cast` 向下转型**：`static_cast<Derived*>` 下行转换在 C++ 中不做运行时检查，使用 `@dynamic_cast` 更安全
5. **Rust 中的类型转换**：Rust 提供 `as`（数值转换）、`From`/`Into`（安全转换）、`transmute`（位级转换，等价于 `reinterpret_cast`）
