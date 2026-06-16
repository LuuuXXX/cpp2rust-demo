# 047_noexcept_basic - noexcept（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **noexcept** 基本函数、内部捕获异常的安全包装，以及 move-only 类型的 FFI 处理方式。采用 idiomatic 命名空间风格（`noexcept_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；`NoexceptMover` 直接持有 `int` 状态。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### noexcept_basic.h

```cpp
namespace noexcept_basic_ns {

inline int noexcept_add(int a, int b) noexcept { return a + b; }
inline int noexcept_multiply(int a, int b) noexcept { return a * b; }
inline int conditional_abs(int x) noexcept { return x < 0 ? -x : x; }

class NoexceptMover {
    int value_;
public:
    explicit NoexceptMover(int v) noexcept : value_(v) {}
    NoexceptMover(const NoexceptMover&) = delete;
    NoexceptMover& operator=(const NoexceptMover&) = delete;
    NoexceptMover(NoexceptMover&& o) noexcept : value_(o.value_) { o.value_ = 0; }
    NoexceptMover& operator=(NoexceptMover&&) noexcept = default;
    int get_value() const noexcept { return value_; }
};

int throwing_divide(int a, int b);
int safe_divide(int a, int b) noexcept;

} // namespace noexcept_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类、`make_unique` 工厂与命名空间自由函数：

```rust
hicc::cpp! {
    #include "noexcept_basic.h"
}

hicc::import_class! {
    #[cpp(class = "noexcept_basic_ns::NoexceptMover")]
    pub class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        pub fn get_value(&self) -> i32;

        pub fn new(v: i32) -> Self { noexcept_mover_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    #[cpp(func = "std::unique_ptr<noexcept_basic_ns::NoexceptMover> hicc::make_unique<noexcept_basic_ns::NoexceptMover, int>(int&&)")]
    pub fn noexcept_mover_new(v: i32) -> NoexceptMover;

    #[cpp(func = "int noexcept_basic_ns::safe_divide(int, int)")]
    pub fn safe_divide(a: i32, b: i32) -> i32;
    // noexcept_add / noexcept_multiply / conditional_abs 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 异常规格 | `noexcept` | 绑定为普通安全函数 |
| 异常处理 | `safe_divide` 内部 catch | 只接收 `i32` 结果 |
| 移动类型 | copy deleted / noexcept move | hicc 绑定内部持有，对外透明 |
| 析构 | C++ 默认析构 | Rust `Drop` 自动触发 |
| 跨 FFI 数据 | `int` | `i32` |

## 运行结果

```
=== 047_noexcept_basic - noexcept（hicc 直出）===

noexcept_add(2,3)=5
noexcept_multiply(4,5)=20
conditional_abs(-7)=7 conditional_abs(7)=7
safe_divide(10,2)=5 safe_divide(10,0)=-1
mover value=42

Rust FFI: hicc 直接绑定 noexcept 命名空间函数与 move-only 类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_noexcept_functions` | noexcept_add / noexcept_multiply / conditional_abs / safe_divide |
| `smoke_noexcept_mover_is_per_object` | NoexceptMover::new / get_value 的对象内状态 |

### 运行方式

```bash
cd examples/047_noexcept_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `noexcept` 命名空间函数可通过 hicc 直接绑定，无需 extern-C shim
- `throwing_divide` 的异常在 C++ `safe_divide` 内部捕获，跨 FFI 只返回标量
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
