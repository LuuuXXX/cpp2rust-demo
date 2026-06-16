# 044_enum_class - enum class（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++11 **enum class**（强类型枚举）的 FFI 处理方式。采用 idiomatic 命名空间风格（`enum_class_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；`OperationResult` 直接持有 `ErrorCode` / `State` / `Flags`。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### enum_class.h

```cpp
namespace enum_class_ns {

enum class ErrorCode : int { None = 0, InvalidInput = 1, OutOfMemory = 2, NotFound = 3, PermissionDenied = 4, Unknown = 99 };
enum class State : unsigned char { Idle = 0, Running = 1, Paused = 2, Stopped = 3 };
enum class Flags : unsigned int { None = 0, Read = 1, Write = 2, Execute = 4, All = 7 };

class OperationResult {
    ErrorCode error_;
    State state_;
    Flags flags_;
public:
    OperationResult();
    void set_error(int code);
    int get_error() const;
    void set_state(unsigned char s);
    unsigned char get_state() const;
    void set_flags(unsigned int f);
    unsigned int get_flags() const;
};

unsigned int combine_flags(unsigned int f1, unsigned int f2);
int has_flag(unsigned int flags, unsigned int flag);

} // namespace enum_class_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类、`make_unique` 工厂与命名空间自由函数：

```rust
hicc::cpp! {
    #include "enum_class.h"
}

hicc::import_class! {
    #[cpp(class = "enum_class_ns::OperationResult")]
    pub class OperationResult {
        #[cpp(method = "void set_error(int)")]
        pub fn set_error(&mut self, code: i32);
        #[cpp(method = "int get_error() const")]
        pub fn get_error(&self) -> i32;
        // state / flags 略

        pub fn new() -> Self { operation_result_new() }
    }
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    #[cpp(func = "std::unique_ptr<enum_class_ns::OperationResult> hicc::make_unique<enum_class_ns::OperationResult>()")]
    pub fn operation_result_new() -> OperationResult;

    #[cpp(func = "unsigned int enum_class_ns::combine_flags(unsigned int, unsigned int)")]
    pub fn combine_flags(f1: u32, f2: u32) -> u32;
}
```

## FFI 对比分析

| 方面 | C++ enum class | Rust FFI |
|------|----------------|----------|
| 类型持有 | `enum class` 成员 | hicc 绑定内部持有，对外透明 |
| 错误码 | `ErrorCode` | `i32` 底层整数 |
| 状态 | `State : unsigned char` | `u8` |
| 标志位 | `Flags : unsigned int` | `u32` |
| 转换 | `static_cast` | 直接使用标量 |
| 析构 | C++ 默认析构 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 044_enum_class - enum class（hicc 直出）===

error=3 state=1 flags=7
combine_flags(1,2)=3 has_execute=1 has_execute_in_read=0

Rust FFI: hicc 直接绑定持有 enum class 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_operation_result_state_is_per_object` | set/get error、state、flags 的对象内状态 |
| `smoke_flags_helpers` | combine_flags / has_flag |

### 运行方式

```bash
cd examples/044_enum_class/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `enum class` 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 跨 FFI 只交换 `int` / `unsigned char` / `unsigned int` 等标量，枚举转换在 C++ 内部用 `static_cast` 完成
