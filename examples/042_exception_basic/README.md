# 042_exception_basic - 异常处理（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **异常处理** 在 FFI 场景下的安全处理方式。采用 idiomatic 命名空间风格
（`exception_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`Calculator` 在方法边界内部使用真实 `throw` / `catch`，把异常转换为对象内错误码，确保没有
C++ 异常跨 FFI 边界传播。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### exception_basic.h

```cpp
namespace exception_basic_ns {

class Calculator {
    int last_error_;  // 0=none,1=invalid_argument,2=out_of_range,3=runtime_error
public:
    Calculator();
    int last_error() const;
    void clear_error();
    int has_error() const;
    int divide(int a, int b);
    int parse_int(const char* s);
};

} // namespace exception_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "exception_basic.h"
}

hicc::import_class! {
    #[cpp(class = "exception_basic_ns::Calculator")]
    pub class Calculator {
        #[cpp(method = "int divide(int a, int b)")]
        pub fn divide(&mut self, a: i32, b: i32) -> i32;
        #[cpp(method = "int parse_int(const char* s)")]
        pub fn parse_int(&mut self, s: *const i8) -> i32;
        // last_error / clear_error / has_error 略

        pub fn new() -> Self { calculator_new() }
    }
}

hicc::import_lib! {
    #![link_name = "exception_basic"]

    #[cpp(func = "std::unique_ptr<exception_basic_ns::Calculator> hicc::make_unique<exception_basic_ns::Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 异常产生 | `throw std::runtime_error` / `std::stoi` | 不直接暴露 |
| 异常处理 | 方法内部 `catch` | 观察 `last_error()` / `has_error()` |
| 状态持有 | `Calculator::last_error_` 成员 | 每个 Rust 对象独立持有 |
| 析构 | `~Calculator` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 042_exception_basic - 异常处理（hicc 直出）===

10 / 2 = 5 error=0
1 / 0 = 0 error=3 has_error=1
after clear has_error=0
parse_int(123) = 123 error=0
parse_int(abc) = 0 error=1
parse_int(99999999999999999999) = 0 error=2

Rust FFI: hicc 直接绑定类，C++ 异常在方法边界内部捕获并转为错误码
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_calculator_error_state` | 正常除法、除零错误、清错、合法/非法/越界字符串转换 |

> 注：错误码保存在 `Calculator` 对象内，不使用全局状态，故并行测试下断言仍确定。

### 运行方式

```bash
cd examples/042_exception_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 异常必须在 FFI 方法边界内部捕获，不能跨语言边界传播
- hicc 可直接绑定持有错误状态的命名空间类，无需 extern-C shim 和手写 delete
- 错误码为对象内状态，调用者可用 `last_error()` / `has_error()` 查询并用 `clear_error()` 清除
