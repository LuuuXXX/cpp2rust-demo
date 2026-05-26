# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路

v5 通过 **LD_PRELOAD 编译拦截**机制，在真实编译过程中捕获 C++ 代码信息，生成 Rust FFI 脚手架。

**关键理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换时只关注实际被实例化的具体类型（如 `std::vector<int>`），不关注模板声明。

### 1.2 版本定位

v5 是完全独立的新版本，所有输入都必须通过 LD_PRELOAD 编译拦截方式获取。

### 1.3 版本演进

| 版本 | 核心突破 | 局限性 |
|------|---------|--------|
| v1-v4 | 静态分析方案 | 模板实例化/宏展开依赖推断 |
| **v5** | **LD_PRELOAD 编译拦截，捕获真实编译过程** | 仅支持 Linux |

---

## 2. 快速开始

### 2.1 工作流程

```bash
# Step 1: 编译拦截（复用项目构建系统）
cd cpp-project/
C2RUST_FEATURE_ROOT=.c2rust/v5 \
C2RUST_CC=g++ \
C2RUST_LD=g++ \
LD_PRELOAD=/path/to/libhook.so \
    make -j4

# Step 2: 初始化（基于捕获结果生成 Rust 脚手架）
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc

# Step 3: 合并（如需要）
cpp2rust-ffi merge -i ./rust_hicc
```

### 2.2 高级选项

```bash
# 多构建配置合并
cpp2rust-ffi init --merge-configs \
    --inputs .c2rust/v5-debug/capture .c2rust/v5-release/capture \
    -o ./rust_hicc

# 强制实例化指定模板
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc \
    --instantiate-templates=std::vector,std::map

# 跳过编译失败的文件
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --skip-failed

# 增量处理
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --incremental
```

---

## 3. 技术架构

### 3.1 整体架构

```
cpp2rust-ffi tool (v5)
├── src/
│   ├── main.rs                    # CLI 入口 (init / merge)
│   ├── hook/                     # LD_PRELOAD Hook 库
│   │   ├── hook.c               # C 拦截器（基于 c2rust-demo 移植）
│   │   └── Makefile             # 编译 libhook.so
│   ├── compiler/                 # 编译捕获
│   │   ├── ast_capturer.rs      # AST 解析
│   │   └── instantiation_tracker.rs  # 模板实例化追踪
│   ├── extractor/                # 信息提取
│   │   ├── class_extractor.rs
│   │   ├── function_extractor.rs
│   │   ├── template_extractor.rs
│   │   ├── macro_expander.rs
│   │   └── enum_extractor.rs
│   ├── postprocessor/           # 后处理
│   │   ├── operator_handler.rs
│   │   ├── friend_handler.rs
│   │   └── lambda_handler.rs
│   ├── generator/               # 代码生成
│   │   ├── hicc_codegen.rs
│   │   └── project_generator.rs
│   └── todo_collector.rs
└── Cargo.toml
```

### 3.2 四阶段处理流程

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 编译拦截 (hook/)                                          │
│    LD_PRELOAD 注入 → 预处理 → AST 导出 → 符号记录            │
│    输出: .cpp2rust, .ast.json, .symbols.json                │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. 提取 (extractor/)                                        │
│    类/函数/模板实例化/Lambda/宏展开/枚举                      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 后处理 (postprocessor/)                                   │
│    运算符 → named shim │ 友元 → import_lib! │ Lambda → wrapper│
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. 生成 (generator/)                                         │
│    hicc 宏格式 Rust 代码 → lib.rs + <unit>.rs               │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 编译拦截流程

```
真实编译: g++ -c foo.cpp -o foo.o
              ↓
    LD_PRELOAD=libhook.so
              ↓
    hook.so 拦截编译器调用
              ↓
    1. 预处理: clang++ -E -C -P foo.cpp → foo.cpp2rust
    2. AST导出: clang++ -Xclang -ast-dump=json foo.cpp → foo.ast.json
    3. 符号记录: 生成 foo.symbols.json
```

---

## 4. 输出格式

### 4.1 目录结构

```
rust_hicc/
├── Cargo.toml
└── src/
    ├── lib.rs              # 库入口
    ├── foo.rs              # 编译单元 foo
    ├── bar.rs              # 编译单元 bar
    └── baz.rs              # 编译单元 baz
```

> 每个 C++ 编译单元（.cpp）对应一个同名 .rs 文件

### 4.2 Rust 文件格式（三段式）

```rust
// ========== 1. C++ 实现内联（hicc::cpp! 块） ==========
hicc::cpp! {
    #include "foo.h"

    // shim 函数
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// ========== 2. 类方法绑定（hicc::import_class! 块） ==========
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

// ========== 3. 全局函数绑定（hicc::import_lib! 块） ==========
hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;

    #[cpp(func = "void foo_delete(Foo* self)")]
    unsafe fn foo_delete(self_: *mut Foo);
}
```

### 4.3 lib.rs 格式

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod foo;
pub mod bar;
pub mod baz;

use ::core::ffi::*;
type c_size_t = usize;
type c_ssize_t = isize;
```

### 4.4 Cargo.toml 格式

```toml
[package]
name = "rust_hicc"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib"]

[dependencies]
hicc = { path = "../../hicc" }
```

---

## 5. C++ 特性支持

### 5.1 总览

| 类别 | 数量 | ✅ | ⚠️ |
|------|------|----|----|
| 基础类型与函数 | 5 | 5 | 0 |
| 类与对象 | 7 | 7 | 0 |
| 面向对象特性 | 6 | 6 | 0 |
| 运算符与类型 | 5 | 3 | 2 |
| 模板实例化 | 5 | 5 | 0 |
| 智能指针与内存 | 5 | 5 | 0 |
| STL 容器 | 5 | 5 | 0 |
| 函数对象 | 4 | 3 | 1 |
| 其他高级特性 | 6 | 6 | 0 |
| **总计** | **48** | **45** | **3** |

### 5.2 ⚠️ 降级处理（3 项）

| 特性 | 示例 | 处理方式 | TODO tag |
|------|------|---------|----------|
| 运算符重载 | 019 | named shim，提示可实现 std::ops trait | `[OP]` |
| 友元函数 | 020 | 直接入 import_lib! | `[FR]` |
| typeid/RTTI | 023 | 枚举注入 | `[RTTI]` |

### 5.3 模板实例化（5/5 ✅）

| 示例 | 特性 | 处理方式 |
|------|------|---------|
| 024 | 函数模板 | 捕获实际实例化 |
| 025 | 类模板 | 捕获 ClassTemplateSpecialization |
| 026 | 模板偏特化 | 捕获偏特化实例化 |
| 027 | 显式实例化 | 捕获显式实例化声明 |
| 028 | 可变参数模板 | 捕获固定元数展开 |

> **v5 核心优势**：通过编译拦截直接捕获实例化结果，比 v4 的静态推断更准确。

---

## 6. 局限性及处理方案

### 6.1 局限性总览

| 限制 | 处理方案 |
|------|---------|
| 仅支持 Linux | Docker 容器方案 |
| 需要完整构建环境 | `--skip-failed` 跳过失败文件 |
| 模板实例化依赖编译 | `--instantiate-templates` 强制实例化 |
| 宏展开后代码膨胀 | `--incremental` + `--prune-macros` |
| 无法捕获运行时信息 | 降级为 opaque pointer + TODO |

### 6.2 详细方案

#### Linux only
```bash
docker run --rm -v $(pwd):/project \
    -e C2RUST_FEATURE_ROOT=/project/.c2rust \
    cpp2rust-ffi:v5 bash -c "make -j4"
```

#### 编译失败跳过
```bash
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc --skip-failed
# 生成 meta/failed-builds.txt 报告
```

#### 强制模板实例化
```bash
cpp2rust-ffi init -i .c2rust/v5/capture -o ./rust_hicc \
    --instantiate-templates=std::vector,std::map,std::string
```

---

## 7. 实现计划

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| Phase 0 | 移植 hook.c → hook.cpp（支持 C++） | P0 | c2rust-demo |
| Phase 1 | AST 编译引擎（基于拦截输入） | P0 | Phase 0 |
| Phase 2 | 模板实例化追踪器 | P0 | Phase 1 |
| Phase 3 | 宏展开处理器 | P1 | Phase 1 |
| Phase 4 | hicc 代码生成器 | P0 | Phase 2, 3 |
| Phase 5 | 基础提取器 | P0 | Phase 1 |
| Phase 6 | 后处理器（OP/FR/Lambda） | P1 | Phase 5 |
| Phase 7 | 多构建配置合并 | P2 | Phase 1-6 |
| Phase 8 | 局限性处理（Docker/增量） | P1 | Phase 1-7 |
| Phase 9 | 集成测试 | P1 | Phase 1-8 |

---

## 8. 技术依赖

```toml
[dependencies]
clang = "2"            # libclang 绑定
clap = "4"             # CLI
anyhow = "1"           # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"       # AST JSON 解析

[build-dependencies]
cc = "1"               # C++ 编译器调用
```

```bash
# 系统依赖
apt-get install clang libclang-dev g++ libstdc++-dev
```
