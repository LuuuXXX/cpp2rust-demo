# 示例 15：异常处理

## 特性概述

本示例展示 C++ 的**异常处理**机制，包括 `try/catch` 语句、`throw` 表达式、自定义异常类、`noexcept` 规范以及异常安全保证。hicc 通过 `hicc::Exception<T>` 类型在 Rust 侧捕获 C++ 异常。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| `throw` 表达式 | 抛出任意类型的异常 |
| `try/catch` | 捕获并处理异常 |
| 自定义异常类 | 继承 `std::exception` 层次 |
| `noexcept` | 声明函数不抛出异常 |
| 异常规范 | `noexcept(false)` 显式允许异常 |
| 异常安全 | 基本保证/强保证/不抛出保证 |

### 代码结构

```cpp
// 自定义异常
class MyException : public std::runtime_error {
    int error_code;
    int get_code() const;
};

// 可能抛出的函数
int divide_throw(int a, int b);           // throw std::runtime_error
int get_element(const int* arr, int size, int index); // throw std::out_of_range
void throw_custom();                      // throw MyException

// 捕获异常的函数
int safe_divide(int a, int b, int default_val);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `CXXThrowExpr` | `throw` 表达式 |
| `CXXTryStmt` | `try` 块 |
| `CXXCatchStmt` | `catch` 块 |
| `FunctionDecl.noexcept: true` | `noexcept` 函数 |
| `FunctionDecl` 函数体含 `CXXThrowExpr` | 可能抛出异常的函数 |

## hicc 处理方式

### `hicc::Exception<T>` 包装器

当 C++ 函数可能抛出异常时，将返回类型包装在 `hicc::Exception<T>` 中：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 普通返回值 → Exception<T> 捕获异常
    #[cpp(func = "int divide_throw(int, int)")]
    fn divide_throw(a: i32, b: i32) -> hicc::Exception<i32>;

    // 返回 void 的函数
    #[cpp(func = "void throw_custom()")]
    fn throw_custom() -> hicc::Exception<()>;

    // 不抛出异常的函数（不需要包装）
    #[cpp(func = "int safe_divide(int, int, int)")]
    fn safe_divide(a: i32, b: i32, default_val: i32) -> i32;
}

fn main() {
    // 处理可能的异常
    match divide_throw(10, 0).ok() {
        Ok(result) => println!("result = {}", result),
        Err(e) => println!("caught exception: {}", e),
    }

    // 忽略返回值，只检查是否有异常
    if let Err(e) = throw_custom().ok() {
        println!("custom exception: {}", e);
    }
}
```

### `hicc::Exception<T>` 接口

```rust
impl<T> hicc::Exception<T> {
    // 转换为 Result<T, String>
    fn ok(self) -> Result<T, String>;

    // 检查是否有异常
    fn has_exception(&self) -> bool;

    // 获取异常信息字符串
    fn exception_message(&self) -> Option<&str>;
}
```

hicc 尽力将 C++ 异常转换为可读的 `String` 错误信息：
- `std::exception` 子类：调用 `.what()` 方法
- `const char*` 异常：直接作为字符串
- 其他类型：以类型名表示

### 缺省参数与异常

`hicc::Exception<T>` 也可以与省略缺省参数一起使用：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // C++: int foo(int v1, int v2 = 0) { throw 3; return v1 + v2; }
    #[cpp(func = "int foo(int, int)")]
    fn foo(v: i32) -> hicc::Exception<()>;  // 省略 v2 缺省参数，忽略返回值
}
```

### `noexcept` 函数

`noexcept` 标记的函数保证不抛出异常，可以直接使用普通返回类型：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // noexcept 函数不需要 Exception<T> 包装
    #[cpp(func = "int safe_operation(int) noexcept")]
    fn safe_operation(x: i32) -> i32;
}
```

### 自定义异常类映射

自定义异常类可以通过普通的 `import_class!` 映射：

```rust
hicc::import_class! {
    #[cpp(class = "MyException")]
    class MyException {
        #[cpp(method = "int get_code() const")]
        fn get_code(&self) -> i32;

        #[cpp(method = "const char* what() const")]
        fn what(&self) -> *const i8;
    }
}
```

## cpp2rust-demo 的异常检测策略

cpp2rust-demo 通过以下启发式方法检测可能抛出的函数：
1. 函数体中含有 `CXXThrowExpr` 节点
2. 调用了其他已知可能抛出的函数
3. 函数未标记 `noexcept`

对于检测到可能抛出的函数，生成注释提示开发者考虑使用 `hicc::Exception<T>`。

## 注意事项

1. **异常信息精度**：hicc 的异常信息转换是尽力而为，复杂异常类型可能只能给出类型名
2. **`noexcept` 违反**：若 `noexcept` 函数实际抛出异常，C++ 会调用 `std::terminate`，Rust 侧无法捕获
3. **异常与 Rust 的 panic**：C++ 异常和 Rust panic 是不同的机制，`hicc::Exception<T>` 只处理 C++ 异常
4. **性能开销**：即使没有异常发生，`hicc::Exception<T>` 的 try/catch 包装也有轻微性能开销
5. **嵌套异常**：C++11 的 `std::nested_exception` 在当前版本中不特别支持
