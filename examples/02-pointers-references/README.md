# 示例 02：指针与引用

## 特性概述

本示例展示 C++ 的**指针与引用**体系，包括原始指针（`*`）、引用（`&`）、多重指针（`**`）、函数指针，以及通过指针操作对象生命周期。这些是 C++ 内存管理的核心，在 Rust FFI 中需要特别处理。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 原始指针 `T*` | 可能为空，需手动管理生命周期 |
| 常量指针 `const T*` | 指向常量的指针，不可修改目标值 |
| 引用 `T&` | 非空别名，始终有效 |
| 多重指针 `T**` | 指向指针的指针 |
| 对象指针 | 指向类实例的指针 |

### 代码结构

```cpp
class Counter { ... };

// 指针参数
void increment(int* p);
void ptr_to_ptr(int** pp);

// 引用参数
void increment_ref(int& r);

// 堆内存管理
int* create_int(int v);  // new int(v)
void destroy_int(int* p); // delete p

// 对象生命周期
Counter* create_counter(int v);
void destroy_counter(Counter* c);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `qualType: "int *"` | 原始指针参数 |
| `qualType: "const int *"` | 常量指针参数 |
| `qualType: "int &"` | 引用参数 |
| `qualType: "int **"` | 双重指针 |
| `CXXRecordDecl` | `Counter` 类声明 |
| `CXXConstructorDecl` | 构造函数 |
| `CXXDestructorDecl` | 析构函数 |

## hicc 处理方式

### 指针类型映射

| C++ 类型 | Rust 类型 | 说明 |
|----------|-----------|------|
| `T*` | `*mut T` | 可变裸指针（可为空） |
| `const T*` | `*const T` | 只读裸指针 |
| `T**` | `*mut *mut T` | 双重指针 |
| `T*` (C++ 对象) | `ClassMutPtr<'_, T>` | 推荐用法，携带生命周期 |
| `const T*` (C++ 对象) | `ClassPtr<'_, T>` | 只读对象指针 |

### 引用类型映射

| C++ 类型 | Rust 类型 | 说明 |
|----------|-----------|------|
| `T&` 基础类型 | `&mut T` | 可变引用 |
| `const T&` 基础类型 | `&T` | 只读引用 |
| `const T&` C++ 对象 | `ClassRef<'_, T>` | 对象只读引用 |
| `T&` C++ 对象 | `ClassRefMut<'_, T>` | 对象可变引用 |

### 全局函数映射

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 指针参数
    #[cpp(func = "void increment(int*)")]
    fn increment(p: *mut i32);

    // 引用参数（基础类型）
    #[cpp(func = "void increment_ref(int&)")]
    fn increment_ref(r: &mut i32);

    // 堆内存管理
    #[cpp(func = "int* create_int(int)")]
    fn create_int(v: i32) -> *mut i32;

    #[cpp(func = "void destroy_int(int*)")]
    fn destroy_int(p: *mut i32);
}
```

### 类对象指针（推荐写法）

对于指向 C++ 类的指针，推荐使用 `ClassMutPtr` / `ClassPtr` 而非裸指针：

```rust
hicc::import_class! {
    #[cpp(class = "Counter")]
    class Counter {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Counter;

    #[cpp(func = "Counter* create_counter(int)")]
    fn create_counter(v: i32) -> Counter;

    #[cpp(func = "void destroy_counter(Counter*)")]
    fn destroy_counter(c: Counter);
}
```

### 多重指针（高级用法）

多重指针通过 `ClassPtr<'a, T, N>` 泛型表示：

```rust
// C++: std::string** string_array_clone(const std::string** in, size_t len)
fn string_array_clone(
    in: ClassPtr<'_, MyString, 2>,
    len: usize
) -> ClassMutPtr<'static, MyString, 2>;
```

## 注意事项

1. **空指针安全**：C++ 指针可能为空，调用 Rust 侧代码前需做空指针检查（`is_null()`）
2. **生命周期管理**：通过裸指针获得的对象不受 Rust 所有权管理，需手动调用对应的 `destroy_*` 函数
3. **引用非空保证**：C++ 引用对应 Rust 的 `&T` / `&mut T`，语义上保证非空
4. **`ClassRef` vs `&T`**：返回 C++ 类引用时，必须用 `ClassRef<'_, T>` 而非 `&T`，以正确传递指针语义
