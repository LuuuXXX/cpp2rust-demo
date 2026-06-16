# 040_std_function - std::function（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::function** 的 FFI 处理方式。采用 idiomatic 命名空间风格
（`std_function_ns`），不再使用 extern-C 函数指针回调 + 不透明指针 + `*_new`/`*_delete` 桥接；
`Callback` 内部持有由 lambda 构造的 `std::function`，`Pipeline` 内部持有多个 `std::function`
并按顺序执行。回调完全在 C++ 侧内部持有，无需把 Rust 函数指针跨 FFI 传递；析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### std_function.h / .cpp

```cpp
namespace std_function_ns {

class Callback {                        // 按 kind 选择 double/triple/negate 的 lambda
    std::function<int(int)> fn_;
public:
    explicit Callback(int kind);        // 0=double,1=triple,2=negate
    int invoke(int v) const { return fn_(v); }
};

class Pipeline {                        // 按顺序持有 std::function 序列
    std::vector<std::function<int(int)>> fns_;
public:
    Pipeline() = default;
    void add(int kind);                 // 0=double,1=triple,2=negate
    int run(int v) const;               // 依次应用所有回调
    int size() const { return (int)fns_.size(); }
};

} // namespace std_function_ns

// Callback::Callback / Pipeline::add 在 .cpp 中按 kind 赋不同 lambda；run 依次应用。
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，也无需把 Rust 函数指针传入 C++，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "std_function.h"
}

hicc::import_class! {
    #[cpp(class = "std_function_ns::Callback")]
    pub class Callback {
        #[cpp(method = "int invoke(int v) const")]
        pub fn invoke(&self, v: i32) -> i32;
        pub fn new(kind: i32) -> Self { callback_new(kind) }
    }
}

// Pipeline 同理（add(&mut) / run / size）

hicc::import_lib! {
    #![link_name = "std_function"]

    #[cpp(func = "std::unique_ptr<std_function_ns::Callback> hicc::make_unique<std_function_ns::Callback, int>(int&&)")]
    pub fn callback_new(kind: i32) -> Callback;
    // pipeline_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 回调持有 | `std::function` 成员 | hicc 绑定内部持有，对外透明 |
| 单个回调 | `Callback::invoke` | 同名方法 |
| 回调链 | `Pipeline` 顺序执行 | `add` / `run` / `size` |
| 析构 | `~Callback` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 040_std_function - std::function（hicc 直出）===

double(5)=10
triple(5)=15
negate(5)=-5

pipeline size=2 run(2)=12

Rust FFI: hicc 绑定内部持有 std::function 的类，回调状态在 C++ 侧保留
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_callback_double` | double 回调结果正确 |
| `smoke_callback_triple` | triple 回调结果正确 |
| `smoke_callback_negate` | negate 回调结果正确 |
| `smoke_pipeline_add_and_run` | Pipeline 按顺序执行 double 再 triple |
| `smoke_pipeline_size_and_state` | Pipeline size 与每对象状态 |

### 运行方式

```bash
cd examples/040_std_function/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::function` 可通过 hicc 绑定内部持有它的类来表达，无需跨 FFI 传函数指针
- 单个回调与回调链均可，回调状态在 C++ 侧保留
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
