# 示例 13：Lambda 与函数式编程

## 特性概述

本示例展示 C++ 的 **Lambda 表达式与函数式编程**，包括 Lambda 的捕获方式（值捕获、引用捕获、混合捕获）、`std::function` 包装以及高阶函数模式。hicc 通过 `hicc::Function<fn(...)>` 在 Rust 侧对应处理。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| Lambda 表达式 | `[捕获列表](参数列表) -> 返回类型 { 函数体 }` |
| 值捕获 `[=]` | 拷贝外部变量 |
| 引用捕获 `[&]` | 引用外部变量 |
| `mutable` Lambda | 允许修改值捕获的变量 |
| `std::function<R(Args...)>` | 类型擦除的函数包装器 |
| 高阶函数 | 接受或返回函数的函数 |
| 可变参数 Lambda | `[](auto... ys)` |

### 代码结构

```cpp
// 接受 std::function 的函数
int apply(int x, int y, std::function<int(int, int)> op);

// 返回 std::function
std::function<int(int)> make_multiplier(int factor);

// 有状态的 Lambda（mutable 捕获）
auto make_counter(int start) {
    int count = start;
    return [count]() mutable { return count++; };
}

// 高阶过滤函数
int sum_if(const std::vector<int>& v, std::function<bool(int)> pred);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `LambdaExpr` | Lambda 表达式 |
| `CXXRecordDecl`（匿名，含 `lambda` 标志） | Lambda 闭包类型 |
| `qualType: "std::function<int (int, int)>"` | `std::function` 参数类型 |
| `FunctionDecl` 参数含 `std::function` | 高阶函数 |

## hicc 处理方式

### `std::function` → `hicc::Function`

C++ 的 `std::function<R(Args...)>` 在 Rust 侧通过 `hicc::Function<fn(Args...) -> R>` 映射：

| C++ 类型 | Rust 类型 |
|----------|-----------|
| `std::function<void()>` | `hicc::Function<fn()>` |
| `std::function<int(int)>` | `hicc::Function<fn(i32) -> i32>` |
| `std::function<int(int, int)>` | `hicc::Function<fn(i32, i32) -> i32>` |
| `std::function<bool(int)>` | `hicc::Function<fn(i32) -> bool>` |

### 传递函数给 C++

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int apply(int, int, std::function<int(int, int)>)")]
    fn apply(x: i32, y: i32, op: hicc::Function<fn(i32, i32) -> i32>) -> i32;

    #[cpp(func = "int sum_if(const std::vector<int>&, std::function<bool(int)>)")]
    fn sum_if(v: &IntVec, pred: hicc::Function<fn(i32) -> bool>) -> i32;
}

fn main() {
    // 将 Rust 闭包转换为 hicc::Function
    let result = apply(3, 4, hicc::Function::new(|a, b| a + b));
    println!("3 + 4 = {}", result);

    // 过滤偶数求和
    let sum = sum_if(&vec, hicc::Function::new(|x| x % 2 == 0));
}
```

### 从 C++ 返回函数

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "std::function<int(int)> make_multiplier(int)")]
    fn make_multiplier(factor: i32) -> hicc::Function<fn(i32) -> i32>;
}

fn main() {
    let double_fn = make_multiplier(2);
    // 调用 C++ 返回的函数
    let result = double_fn.call(5);  // 返回 10
    println!("2 * 5 = {}", result);
}
```

### Lambda 的限制

Lambda 表达式本身是匿名的编译期类型，无法直接通过 AST 导出为 FFI 接口。只能通过以下方式在 FFI 边界传递：

1. **`std::function` 包装**：类型擦除后可跨 FFI 边界
2. **函数指针**：无捕获的 Lambda 可隐式转换为函数指针

```rust
// 无捕获 Lambda ↔ 函数指针
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "void set_callback(void (*)(int))")]
    fn set_callback(cb: extern "C" fn(i32));
}
```

### `hicc::Function` 内部实现

`hicc::Function<fn(Args...) -> R>` 在 C++ 侧对应 `std::function<R(Args...)>`，通过 hicc 的 ABI 兼容层实现跨语言闭包传递：

```rust
let captured_value = 42_i32;
let func = hicc::Function::new(move |x: i32| x + captured_value);
// func 内部持有闭包，生命周期需比 C++ 侧使用更长
```

## 注意事项

1. **生命周期要求**：传递给 C++ 的 `hicc::Function` 必须保持有效，直到 C++ 侧不再使用该函数对象
2. **线程安全**：`hicc::Function` 与 `std::function` 一样，默认不保证线程安全
3. **捕获变量生命周期**：引用捕获（`[&]`）的变量在 Rust 侧使用时需格外小心，避免悬垂引用
4. **`va_list` 与 `std::function` 区别**：可变参数函数（`...`）通过 `va_list` 处理，与 `std::function` 是不同的机制
5. **Lambda 可变性**：`mutable` Lambda 在 Rust 侧通过 `FnMut` 闭包对应
