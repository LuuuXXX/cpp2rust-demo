# rapidjson_sys 功能详解

## 概述

`rapidjson_sys` 是 RapidJSON C++ 代码库迁移到 Rust 过程中的一个关键组件，作为一个 **FFI (Foreign Function Interface) 绑定层**，它允许 Rust 代码通过 C ABI 调用原始 RapidJSON C++ 库中的 `BigInteger` 实现。

项目地址：`./references/rapidjson-refactoring/rapidjson_sys`

## 核心功能

### 1. C++ BigInteger 类的 FFI 封装

`rapidjson_sys` 的核心职责是将 RapidJSON C++ 库中的 `rapidjson::internal::BigInteger` 类通过 FFI 暴露给 Rust 调用。

#### BigInteger 简介

`BigInteger` 是 RapidJSON 内部使用的大整数结构，用于处理超过 64 位的整数运算，主要出现在以下场景：
- JSON 数字解析时的精度处理
- 字符串到数值的转换
- 浮点数格式化

### 2. FFI Shim 层架构

`rapidjson_sys` 采用经典的 FFI Shim 模式：

```
┌─────────────────────────────────────────────────────────────┐
│                      Rust 代码                               │
│  (rapidjson-rs/tests/bigintegertest.rs)                     │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │   CxxBigInteger (safe Rust wrapper)                 │    │
│  │   src/bigintegertest_ffi.rs                        │    │
│  │   - 内存管理（实现 Drop trait）                      │    │
│  │   - 类型安全包装                                     │    │
│  │   - 错误处理（panic 而非返回错误码）                 │    │
│  └─────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │   Bindgen 生成的 FFI bindings                       │    │
│  │   ${OUT_DIR}/ffi_bigintegertest_bindings.rs        │    │
│  │   - extern "C" 函数声明                             │    │
│  │   - 类型映射                                        │    │
│  └─────────────────────────────────────────────────────┘    │
│                         │                                   │
└─────────────────────────┼───────────────────────────────────┘
                          │   C ABI (extern "C")
┌─────────────────────────┼───────────────────────────────────┐
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │   C++ Shim 实现 (bigintegertest_ffi.cpp)            │    │
│  │   - C 风格的 opaque handle 管理                     │    │
│  │   - 调用实际的 C++ BigInteger 类                    │    │
│  └─────────────────────────────────────────────────────┘    │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │   RapidJSON C++ BigInteger 类                       │    │
│  │   (rapidjson/internal/biginteger.h)                 │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│                      C++ 代码                                │
└─────────────────────────────────────────────────────────────┘
```

## 文件结构

```
rapidjson_sys/
├── Cargo.toml              # 包配置
├── build.rs               # 构建脚本：编译 C++ Shim + 生成 bindings
├── src/
│   ├── lib.rs             # 库入口
│   └── bigintegertest_ffi.rs   # Safe Rust 包装器
└── shim/
    ├── bigintegertest_ffi.h    # C ABI 头文件
    └── bigintegertest_ffi.cpp  # C ABI 实现
```

## 构建过程 (build.rs)

### 1. C++ Shim 编译

```rust
cc::Build::new()
    .cpp(true)                                    // 使用 C++ 编译器
    .file("shim/bigintegertest_ffi.cpp")         // Shim 源文件
    .include("../rapidjson_legacy/include")       // RapidJSON 头文件路径
    .compile("bigintegertest_ffi");              // 编译为静态库
```

`build.rs` 使用 `cc` crate 编译 C++ Shim 代码：
- 设置 `.cpp(true)` 启用 C++ 模式
- 指定 RapidJSON 头文件目录
- 编译为静态库 `bigintegertest_ffi`

### 2. Bindgen 绑定生成

```rust
let bindings = bindgen::Builder::default()
    .header("shim/bigintegertest_ffi.h")         // C 头文件
    .allowlist_type("RapidJsonBigIntegerHandle")  // 白名单类型
    .allowlist_function("rapidjson_biginteger_.*") // 白名单函数
    .generate()
    .expect("Unable to generate bindings");
```

`build.rs` 使用 `bindgen` 从 C 头文件自动生成 Rust FFI 绑定：
- 只生成白名单中的类型和函数
- 避免暴露不必要的实现细节
- 生成结果写入 `${OUT_DIR}/ffi_bigintegertest_bindings.rs`

## C ABI 设计

### 头文件 (bigintegertest_ffi.h)

```c
typedef struct RapidJsonBigIntegerHandle RapidJsonBigIntegerHandle;

// 生命周期管理
RapidJsonBigIntegerHandle* rapidjson_biginteger_new(void);
void rapidjson_biginteger_free(RapidJsonBigIntegerHandle* handle);

// 操作接口
int rapidjson_biginteger_from_decimal_literal(
    RapidJsonBigIntegerHandle* handle,
    const char* literal
);
void rapidjson_biginteger_add_u64(RapidJsonBigIntegerHandle*, unsigned long long);
void rapidjson_biginteger_mul_u64(RapidJsonBigIntegerHandle*, unsigned long long);
void rapidjson_biginteger_mul_u32(RapidJsonBigIntegerHandle*, unsigned int);
void rapidjson_biginteger_shl(RapidJsonBigIntegerHandle*, unsigned int);
int rapidjson_biginteger_compare(const RapidJsonBigIntegerHandle* a,
                                 const RapidJsonBigIntegerHandle* b);
int rapidjson_biginteger_to_string(const RapidJsonBigIntegerHandle*,
                                   char* out, unsigned long out_capacity);
```

**设计原则**：
- 使用 **opaque handle** 模式：外部只看到 `RapidJsonBigIntegerHandle*` 指针
- 所有函数遵循 C 约定：返回错误码（0=失败，非0=成功）
- 头文件用 `#pragma once` 和 `extern "C"` 确保 C/C++ 兼容

### C++ 实现 (bigintegertest_ffi.cpp)

```cpp
struct RapidJsonBigIntegerHandle {
    BigInteger value;
};

RapidJsonBigIntegerHandle* rapidjson_biginteger_new() {
    return new (std::nothrow) RapidJsonBigIntegerHandle();
}

void rapidjson_biginteger_free(RapidJsonBigIntegerHandle* handle) {
    delete handle;
}

int rapidjson_biginteger_from_decimal_literal(
    RapidJsonBigIntegerHandle* handle,
    const char* literal
) {
    if (!handle || !literal) return 0;
    handle->value = BigInteger(literal, std::strlen(literal));
    return 1;
}
```

**实现特点**：
- Opaque handle 内部包含实际的 C++ `BigInteger` 对象
- 使用 `std::nothrow` 避免构造函数中的异常
- 参数验证返回 0 表示失败

## Safe Rust 包装器

### bigintegertest_ffi.rs

```rust
pub struct CxxBigInteger {
    inner: *mut RapidJsonBigIntegerHandle,
}

impl CxxBigInteger {
    pub fn new() -> Self {
        let inner = unsafe { rapidjson_biginteger_new() };
        assert!(!inner.is_null(), "rapidjson_biginteger_new returned null");
        Self { inner }
    }

    pub fn from_decimal_literal(lit: &str) -> Self {
        let mut this = Self::new();
        let c = CString::new(lit).expect("decimal literal must not contain NUL");
        let ok = unsafe { rapidjson_biginteger_from_decimal_literal(this.inner, c.as_ptr()) };
        assert_ne!(ok, 0, "rapidjson_biginteger_from_decimal_literal failed");
        this
    }

    pub fn add_u64(&mut self, value: u64) {
        unsafe { rapidjson_biginteger_add_u64(self.inner, value) };
    }

    // ... 其他方法
}

impl Drop for CxxBigInteger {
    fn drop(&mut self) {
        unsafe { rapidjson_biginteger_free(self.inner) };
    }
}
```

**安全包装策略**：
1. **所有权管理**：实现 `Drop` trait 确保 handle 被正确释放
2. **内存安全**：使用 `assert!` 进行非空检查
3. **类型安全**：Rust 方法签名使用原生类型（`u64`, `u32`, `i32`）而非原始指针
4. **FFI 边界清晰**：`unsafe` 块明确标识 FFI 调用点

## 与 rapidjson-rs 的集成

### 依赖关系

`rapidjson-rs/Cargo.toml`:
```toml
[dependencies]
rapidjson_sys = { path = "../rapidjson_sys" }
```

### L1/L2 测试架构

在 `rapidjson-rs/tests/bigintegertest.rs` 中实现了 **L1/L2 镜像测试**：

```rust
// L1: C++ backend via FFI
use rapidjson_sys::bigintegertest_ffi::CxxBigInteger;

// L2: Rust backend via native implementation
use rapidjson_rs::internal::biginteger::BigInteger as RustBigInteger;

// 统一的测试接口
trait TestBigInteger {
    fn from_decimal_literal(lit: &str) -> Self;
    fn add_u64(&mut self, value: u64);
    fn mul_u64(&mut self, value: u64);
    // ...
}

// L1 测试：使用 C++ BigInteger
#[test]
fn l1_cxx_biginteger_add_uint64() {
    run_biginteger_add_uint64::<CxxBigIntegerAdapter>();
}

// L2 测试：使用 Rust BigInteger
#[test]
fn l2_rust_biginteger_add_uint64() {
    run_biginteger_add_uint64::<RustBigIntegerAdapter>();
}
```

**测试镜像策略**：
- L1 层通过 `CxxBigIntegerAdapter` 调用 C++ 实现
- L2 层通过 `RustBigIntegerAdapter` 调用 Rust 原生实现
- 两者共享相同的测试用例函数 `run_biginteger_*`
- 验证 Rust 实现与 C++ 原始实现的行为一致性

## 编译产物

### 构建脚本输出

1. **静态库**：`${OUT_DIR}/libbigintegertest_ffi.a`
   - 包含编译后的 C++ Shim 代码
   - 链接到最终二进制

2. **生成绑定**：`${OUT_DIR}/ffi_bigintegertest_bindings.rs`
   - bindgen 从 C 头文件自动生成
   - 包含 Rust FFI 声明

### 运行时依赖

- 无运行时依赖（静态链接）
- 不需要 C++ 标准库动态链接

## 技术特点

### 1. 最小化 FFI 表面

只暴露必要的操作接口：
- 构造/销毁
- 从十进制字符串创建
- 基本算术运算（加、乘、左移）
- 比较运算
- 序列化（当前未实现）

### 2. 错误处理策略

- C ABI：返回整数错误码（0=失败）
- Rust 包装：使用 `assert!`  panic，简化错误处理
- 未来可改进：返回 `Result` 类型

### 3. 内存管理

- C++ 侧：`new`/`delete` 管理 handle 生命周期
- Rust 侧：`Drop` trait 确保释放
- 无悬垂指针风险

### 4. 构建时代码生成

- `build.rs` 在编译时执行
- 自动编译 C++ Shim
- 自动生成 FFI bindings
- 无需手动维护绑定代码

## 局限性

### 1. to_string 未实现

```cpp
int rapidjson_biginteger_to_string(
    const RapidJsonBigIntegerHandle* handle,
    char* out,
    unsigned long out_capacity
) {
    // Placeholder: decimal formatting not yet implemented.
    return 0;  // 始终返回失败
}
```

### 2. 仅支持无符号大整数

当前实现仅支持非负整数运算。

### 3. 十进制转换依赖 C++ 实现

`rapidjson_biginteger_from_decimal_literal` 内部调用 C++ `BigInteger` 构造函数解析字符串。

## 使用场景

此模块主要用于：
1. **渐进式迁移验证**：通过 L1/L2 测试确保 Rust 实现正确性
2. **混合编程**：在 Rust 项目中使用已验证的 C++ 逻辑
3. **性能基准测试**：对比 Rust 和 C++ 实现性能差异

## 依赖工具

- `rustc`：Rust 编译器
- `cargo`：Rust 包管理器
- `bindgen`：FFI 绑定生成（build-dependencies）
- `cc`：C++ 编译器驱动（build-dependencies）
- C++ 编译器（g++ 或 clang++）
- RapidJSON 头文件（来自 `rapidjson_legacy`）

## 构建命令

```bash
# 在 rapidjson_sys 目录构建
cd references/rapidjson-refactoring
cargo build -p rapidjson_sys

# 在 workspace 根目录构建所有
cargo build
```
