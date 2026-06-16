# 041_functional_bind - std::bind（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::bind** 的 FFI 处理方式。采用 idiomatic 命名空间风格
（`functional_bind_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`Adder` / `Multiplier` 内部持有由 `std::bind` 构造的 `std::function`，`StringProcessor` 持有
字符串状态并统计字符。绑定对象完全在 C++ 侧内部持有，无需把函数指针跨 FFI 传递；析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### functional_bind.h / .cpp

```cpp
namespace functional_bind_ns {

class Adder {                         // 绑定 base 到 std::plus<int>() 的左操作数
    std::function<int(int)> add_;
public:
    explicit Adder(int base);         // std::bind(std::plus<int>(), base, _1)
    int add(int value) const { return add_(value); }
};

class Multiplier {                    // 绑定 factor 到 std::multiplies<int>() 的左操作数
    std::function<int(int)> mul_;
public:
    explicit Multiplier(int factor);  // std::bind(std::multiplies<int>(), factor, _1)
    int multiply(int value) const { return mul_(value); }
};

class StringProcessor {
    std::string target_;
public:
    StringProcessor() = default;
    void set_target(const char* t) { target_ = t ? t : ""; }
    int count_char(char ch) const;
};

} // namespace functional_bind_ns

// Adder / Multiplier 构造函数在 .cpp 中使用 std::bind；count_char 遍历 target_ 计数。
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，也无需把 Rust 函数指针传入 C++，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "functional_bind.h"
}

hicc::import_class! {
    #[cpp(class = "functional_bind_ns::Adder")]
    pub class Adder {
        #[cpp(method = "int add(int value) const")]
        pub fn add(&self, value: i32) -> i32;
        pub fn new(base: i32) -> Self { adder_new(base) }
    }
}

// Multiplier / StringProcessor 同理（set_target(&mut, *const i8) / count_char(&self)）

hicc::import_lib! {
    #![link_name = "functional_bind"]

    #[cpp(func = "std::unique_ptr<functional_bind_ns::Adder> hicc::make_unique<functional_bind_ns::Adder, int>(int&&)")]
    pub fn adder_new(base: i32) -> Adder;
    // multiplier_new / string_processor_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| bind 持有 | `std::function` 成员 | hicc 绑定内部持有，对外透明 |
| 加法绑定 | `std::bind(std::plus<int>(), base, _1)` | `Adder::add` |
| 乘法绑定 | `std::bind(std::multiplies<int>(), factor, _1)` | `Multiplier::multiply` |
| 字符串状态 | `std::string` 成员 | `StringProcessor::set_target/count_char`，状态在 C++ 侧保留 |
| 析构 | `~Adder` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 041_functional_bind - std::bind（hicc 直出）===

adder.add(5)=15
multiplier.multiply(4)=12
count('a')=3

Rust FFI: hicc 绑定内部持有 std::bind(std::function) 的类，状态在 C++ 侧保留
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_adder` | `Adder(10).add(5) == 15` |
| `smoke_multiplier` | `Multiplier(3).multiply(4) == 12` |
| `smoke_string_processor_count_char` | `StringProcessor("banana").count_char('a') == 3` |

### 运行方式

```bash
cd examples/041_functional_bind/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::bind` 可通过 hicc 绑定内部持有 `std::function` 的类来表达，无需跨 FFI 传函数指针
- `Adder` / `Multiplier` 的绑定状态与 `StringProcessor` 的字符串状态均保留在 C++ 侧
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
