# cpp2rust-ffi

C++ 到 Rust Safe FFI 自动化工具 —— 基于 [hicc](https://crates.io/crates/hicc) 宏格式生成绑定代码。

## 目录

- [背景](#背景)
- [快速开始](#快速开始)
- [工作原理](#工作原理)
- [生成代码格式](#生成代码格式)
- [支持的 C++ 特性](#支持的-c-特性)
- [类型映射表](#类型映射表)
- [架构说明](#架构说明)
- [开发指南](#开发指南)
- [测试](#测试)
- [CI 门禁](#ci-门禁)
- [路线图](#路线图)

---

## 背景

直接手写 C++ ↔ Rust FFI 绑定既繁琐又容易出错。本工具通过解析 C++ 头文件，自动生成符合 `hicc` 宏生态的 Rust FFI 绑定代码，使所有生成的代码能**立即通过 `cargo check`**。

### 演进历史

| 版本 | 核心突破 | 遗留问题 |
|------|---------|---------|
| v1 | 头文件解析 → 自动生成 hicc 脚手架 | ❌ 模板实例化 |
| v2 | AST 编译捕获 → 解决模板实例化 | ❌ 运算符/友元/lambda/RTTI |
| v3 | 后处理降级 → 5 类特性生成可编译代码 | ⚠️ 代码格式不对齐、TODO 系统过重 |
| **v4** | **格式对齐 hicc 生态，全面覆盖 48 个示例** | — |

---

## 快速开始

### 安装

```bash
git clone <repo>
cd cpp2rust-demo
cargo install --path .
```

### 用法

```bash
# 从 C++ 头文件目录生成 Rust hicc 项目
cpp2rust-ffi init \
    --input  ./my_cpp_lib/include \
    --output ./rust_hicc \
    --lib-name my_cpp_lib
```

生成结果：

```
rust_hicc/
├── Cargo.toml      # 包含 hicc 依赖
├── build.rs        # cc + hicc-build 编译脚本
└── src/
    └── main.rs     # hicc 宏格式 FFI 绑定
```

### 最简示例

输入头文件 `hello.h`：

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif
void hello(void);
#ifdef __cplusplus
}
#endif
```

运行工具：

```bash
cpp2rust-ffi init --input ./cpp --output ./rust_out --lib-name hello
```

生成 `rust_out/src/main.rs`：

```rust
hicc::cpp! {
    #include "hello.h"
}

hicc::import_lib! {
    #![link_name = "hello"]

    #[cpp(func = "void hello(void)")]
    fn hello();
}

fn main() {}
```

---

## 工作原理

工具采用**四步流水线**处理每一个头文件：

```
1. 读取头文件
   └── 过滤注释、预处理器指令

2. 解析（src/parser.rs）
   ├── 提取 extern "C" 作用域内的自由函数
   ├── 提取 class / struct 声明及其公开方法
   └── 推断构造/析构/普通方法分类

3. IR 构建（src/ir.rs）
   ├── ParsedHeader
   │   ├── Vec<Function>
   │   └── Vec<Class>  →  Vec<Method>
   └── 类型规范化（src/typemap.rs）

4. 代码生成（src/codegen.rs）
   ├── hicc::cpp!{}       C++ 头文件内嵌 + shim 函数
   ├── hicc::import_class!{}  类方法绑定
   └── hicc::import_lib!{}    全局/wrapper 函数绑定
```

---

## 生成代码格式

所有输出均遵循 hicc **三段式宏格式**：

```rust
// ① C++ 实现内嵌
hicc::cpp! {
    #include "foo.h"

    // 由工具自动生成的 shim 函数（仅当 C++ class 无现成 C 包装时）
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ② 类方法绑定（每个类一个块）
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int)")]
        fn set_value(&mut self, value: i32);
    }
}

// ③ 全局/wrapper 函数绑定
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo*)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

### Unsafe 规则

| 场景 | 是否 unsafe |
|------|------------|
| 析构函数（`_delete`） | ✅ `unsafe fn` |
| 参数含 `const char*` | ✅ `unsafe fn` |
| 参数含任意裸指针 | ✅ `unsafe fn` |
| 构造函数（`_new`） | ❌ 安全函数 |
| 返回裸指针的构造函数 | ❌ 安全函数 |

---

## 支持的 C++ 特性

| 特性 | 状态 | 说明 |
|------|------|------|
| 纯 C 函数（`extern "C"`） | ✅ | 直接映射到 `import_lib!` |
| 函数重载 | ✅ | 各重载生成独立条目 |
| 默认参数 | ✅ | 参数解析时丢弃默认值 |
| 基础类（构造/析构/方法） | ✅ | opaque ptr + `import_class!` |
| `const` 成员函数 | ✅ | 映射到 `&self` |
| 非 `const` 成员函数 | ✅ | 映射到 `&mut self` |
| `static` 成员函数 | ✅ | 生成独立 shim + `import_lib!` 条目 |
| `struct`（公开成员默认） | ✅ | 与 `class` 处理一致 |
| 运算符重载 | 🔜 v4.1 | 计划支持命名 shim |
| 友元函数 | 🔜 v4.1 | 计划直接提取为全局函数 |
| Lambda / `std::function` | 🔜 v4.2 | fn ptr / class wrapper 双策略 |
| RTTI / `typeid` | 🔜 v4.2 | 整数枚举 + 虚函数注入 |

---

## 类型映射表

| C++ 类型 | Rust 类型 |
|---------|---------|
| `void` | `()` |
| `int` | `i32` |
| `unsigned int` | `u32` |
| `long` | `i64` |
| `long long` | `i64` |
| `unsigned long long` | `u64` |
| `short` | `i16` |
| `char` | `i8` |
| `unsigned char` | `u8` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `size_t` | `usize` |
| `int8_t` … `int64_t` | `i8` … `i64` |
| `uint8_t` … `uint64_t` | `u8` … `u64` |
| `const char*` | `*const i8` |
| `char*` | `*mut i8` |
| `void*` | `*mut std::ffi::c_void` |
| `const T*` | `*const T` |
| `T*` | `*mut T` |

---

## 架构说明

```
src/
├── main.rs        CLI 入口（clap 子命令 init）
├── lib.rs         公共 API：build_project / write_project
├── ir.rs          中间表示：ParsedHeader / Function / Class / Method
├── parser.rs      C++ 头文件解析器（正则 + 状态机）
├── typemap.rs     C++ → Rust 类型映射
└── codegen.rs     hicc 格式代码生成器
```

### 模块职责

**`parser.rs`**
- 剥离注释、预处理指令
- 提取顶层函数声明（`extern "C"` 块内外）
- 提取类声明及其公开方法（构造/析构/普通/静态）

**`typemap.rs`**
- 规范化 C++ 类型字符串（去除多余空格、`struct`/`class` 前缀）
- 精确映射原始类型及指针/引用类型到 Rust 等价形式

**`codegen.rs`**
- `generate_output_cargo_toml` → 输出项目的 `Cargo.toml`
- `generate_build_rs` → 输出项目的 `build.rs`
- `generate_rust_source` → 完整的 `main.rs` hicc 绑定

---

## 开发指南

### 依赖

- Rust 1.75+（stable）
- `cargo`

### 克隆并构建

```bash
git clone <repo>
cd cpp2rust-demo
cargo build
```

### 运行所有测试

```bash
cargo test
```

### 运行单个测试

```bash
cargo test parses_free_functions
cargo test generates_free_function_bindings
cargo test cli_init_writes_output_project
```

### 代码格式化

```bash
cargo fmt
```

### Clippy 静态分析

```bash
cargo clippy -- -D warnings
```

---

## 测试

本项目严格遵循**测试先行（TDD）**原则，测试是质量保障网。

### 测试分层

| 层次 | 位置 | 说明 |
|------|------|------|
| 单元测试 | `src/*.rs` 中的 `#[cfg(test)]` 块 | 测试每个模块的核心逻辑 |
| 集成测试 | `tests/integration_test.rs` | 端到端验证工具对真实示例的处理结果 |

### 关键测试覆盖

| 测试名称 | 验证内容 |
|---------|---------|
| `maps_primitive_types` | C++ 基础类型 → Rust 类型映射正确 |
| `maps_pointer_types` | 裸指针类型映射（`const T*` / `T*`） |
| `normalizes_cpp_types` | `struct Foo*` 规范化为 `Foo*` |
| `parses_free_functions` | 顶层 C 函数解析 |
| `parses_class_methods` | 类方法解析（构造/析构/const/static） |
| `parses_struct_pointer_params` | `struct Counter*` 参数正确处理 |
| `generates_free_function_bindings` | `hello_world` 示例端到端生成 |
| `generates_class_bindings_and_filters_instance_wrappers` | `class_basic` 示例 + 去重逻辑 |
| `generates_static_method_shims_when_missing` | 静态方法 shim 自动生成 |
| `build_rs_uses_relative_cpp_dir` | `build.rs` 使用相对路径 |
| `parses_example_headers` | 真实示例 001/002/006 解析 |
| `generates_expected_code_for_reference_examples` | 002 示例代码生成验证 |
| `cli_init_writes_output_project` | 006 示例完整 CLI 流程 |

### 新增测试规范

每当添加新功能，请同步添加：
1. **单元测试**：在对应模块的 `#[cfg(test)]` 块中
2. **集成测试**：在 `tests/integration_test.rs` 中，对齐 `examples/` 下的参考输出

---

## CI 门禁

每次 Push 或 Pull Request 都会触发以下四道门禁：

| 检查 | 命令 | 失败条件 |
|------|------|---------|
| 代码格式 | `cargo fmt --check` | 存在未格式化代码 |
| Clippy 静态分析 | `cargo clippy -- -D warnings` | 任何 warning 均视为错误 |
| 构建 | `cargo build --verbose` | 编译失败 |
| 测试 | `cargo test --verbose` | 任何测试失败 |
| 文档构建 | `cargo doc --no-deps` | 文档注释有误 |

配置文件：`.github/workflows/ci.yml`

---

## 路线图

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 1 | C++ 头文件解析 + hicc 代码生成基础框架 | ✅ 完成 |
| Phase 2 | 类（构造/析构/方法/静态）完整支持 | ✅ 完成 |
| Phase 3 | CI 门禁（fmt / clippy / test / doc） | ✅ 完成 |
| Phase 4 | 运算符重载 → named shim + `[OP]` TODO 注释 | 🔜 计划中 |
| Phase 5 | 友元函数 → `import_lib!` + `[FR]` TODO 注释 | 🔜 计划中 |
| Phase 6 | Lambda → fn ptr / class wrapper 双策略 | 🔜 计划中 |
| Phase 7 | RTTI → 整数枚举 + 虚函数注入 | 🔜 计划中 |
| Phase 8 | 48 个示例全部通过 `cargo check` | 🔜 计划中 |

详细设计请参见 [`docs/plans/v4/automated-cpp2rust-ffi-v4.md`](docs/plans/v4/automated-cpp2rust-ffi-v4.md)。
