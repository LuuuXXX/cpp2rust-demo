# 039_lambda_basic - lambda 表达式（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **lambda 表达式**的 FFI 处理方式。采用 idiomatic 命名空间风格
（`lambda_basic_ns`），不再使用 extern-C 函数指针回调 + 不透明指针 + `*_new`/`*_delete` 桥接；
`Operation` 内部持有由 lambda 构造的 `std::function`，`Accumulator` 是捕获状态（`this`）的
闭包。lambda 完全在 C++ 侧内部持有，无需把函数指针跨 FFI 传递；析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### lambda_basic.h / .cpp

```cpp
namespace lambda_basic_ns {

class Operation {                       // 按 kind 选择 add/multiply/max 的 lambda
    std::function<int(int, int)> fn_;
public:
    explicit Operation(int kind);       // 0=add,1=multiply,2=max
    int apply(int a, int b) const { return fn_(a, b); }
};

class Accumulator {                     // 捕获状态的闭包
    int value_;
    std::function<int(int)> adder_;
public:
    explicit Accumulator(int initial);  // adder_ = [this](int d){ return value_ += d; }
    int apply(int delta) { return adder_(delta); }
    int value() const { return value_; }
};

} // namespace lambda_basic_ns

// Operation::Operation 在 .cpp 中按 kind 赋不同 lambda；Accumulator 捕获 this 累加状态。
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，也无需把 Rust 函数指针传入 C++，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "lambda_basic.h"
}

hicc::import_class! {
    #[cpp(class = "lambda_basic_ns::Operation")]
    pub class Operation {
        #[cpp(method = "int apply(int a, int b) const")]
        pub fn apply(&self, a: i32, b: i32) -> i32;
        pub fn new(kind: i32) -> Self { operation_new(kind) }
    }
}

// Accumulator 同理（apply(&mut) / value）

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    #[cpp(func = "std::unique_ptr<lambda_basic_ns::Operation> hicc::make_unique<lambda_basic_ns::Operation, int>(int&&)")]
    pub fn operation_new(kind: i32) -> Operation;
    // accumulator_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| lambda 持有 | `std::function` 成员 | hicc 绑定内部持有，对外透明 |
| 无状态 lambda | 选择运算 | `Operation::apply` |
| 捕获状态闭包 | `[this]` 累加 | `Accumulator::apply`，状态在 C++ 侧保留 |
| 析构 | `~Operation` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 039_lambda_basic - lambda 表达式（hicc 直出）===

add(3,4)=7
mul(3,4)=12
max(3,4)=4

acc.apply(5)=15 apply(3)=18 value=18

Rust FFI: hicc 绑定内部持有 lambda(std::function) 的类，闭包状态在 C++ 侧保留
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_operation_add` | add lambda 结果正确 |
| `smoke_operation_multiply` | multiply lambda 结果正确 |
| `smoke_operation_max` | max lambda 结果正确 |
| `smoke_accumulator_captures_state` | 闭包累加捕获状态 |

### 运行方式

```bash
cd examples/039_lambda_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ lambda 可通过 hicc 绑定内部持有 `std::function` 的类来表达，无需跨 FFI 传函数指针
- 无状态 lambda 与捕获状态的闭包均可，闭包状态在 C++ 侧保留
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
