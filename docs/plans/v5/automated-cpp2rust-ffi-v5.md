# C++ 到 Rust Safe FFI 自动化工具 - 方案 v5

## 1. 概述

### 1.1 核心思路

v5 通过 **LD_PRELOAD 编译拦截**机制，在真实编译过程中捕获 C++ 代码信息，生成 Rust FFI 脚手架。

**关键理念**：C++ 模板的价值在于**实例化结果**，而非模板本身。转换时只关注实际被实例化的具体类型（如 `std::vector<int>`），不关注模板声明。

### 1.2 参考实现来源

| 文件 | 源位置 | 用途 |
|------|--------|------|
| `hook/hook.c` | `references/c2rust-demo/hook/hook.c` | 预处理捕获逻辑（C 版本，需改写为 C++ 版本） |
| `src/capture.rs` | `references/c2rust-demo/src/capture.rs` | LD_PRELOAD 构建与执行逻辑（可直接复用） |

**注意**：需基于 `hook/hook.c` 创建新的 `hook/hook.cpp`，将编译器列表从 `gcc/clang/cc` 改为 `g++/clang++/c++`，文件扩展名从 `.c` 改为 `.cpp/.cc/.cxx` 等。`src/capture.rs` 可直接复用。

### 1.3 版本定位

v5 是完全独立的新版本，所有输入都必须通过 LD_PRELOAD 编译拦截方式获取。

---

## 2. 快速开始

### 2.1 工作流程

```bash
# Step 0: 准备 Hook 文件
# 基于 references/c2rust-demo/hook/hook.c 创建 hook/hook.cpp
# 直接复用 references/c2rust-demo/src/capture.rs
cp references/c2rust-demo/src/capture.rs ./src/

# Step 1: 编译拦截
cd cpp-project/
C2RUST_FEATURE_ROOT=.c2rust/v5 \
C2RUST_PROJECT_ROOT=/path/to/cpp-project \
C2RUST_CXX=g++ \
LD_PRELOAD=/path/to/libhook.so \
    make -j4

# Step 2: 初始化
cpp2rust-ffi init -i .c2rust/v5 -o ./rust_hicc

# Step 3: 合并（如需要）
cpp2rust-ffi merge -i ./rust_hicc
```

### 2.2 环境变量

| 变量 | 必填 | 说明 |
|------|------|------|
| `C2RUST_FEATURE_ROOT` | ✅ | 捕获产物输出目录 |
| `C2RUST_PROJECT_ROOT` | ✅ | C++ 项目根目录 |
| `C2RUST_CXX` | ❌ | C++ 编译器，默认检测 g++/clang++/c++ |
| `C2RUST_DEBUG` | ❌ | 设置为 1 输出调试日志 |

---

## 3. 技术架构

### 3.1 整体架构

```
cpp2rust-ffi tool (v5)
├── src/
│   ├── main.rs                    # CLI 入口 (init / merge)
│   ├── hook/                     # LD_PRELOAD Hook 库
│   │   ├── hook.cpp           # C++ 拦截器（新建）
│   │   └── Makefile
│   ├── capture.rs                # 复用：LD_PRELOAD 执行逻辑
│   ├── ast_parser.rs            # C++ AST 解析（clang crate）
│   ├── extractor/               # 信息提取
│   │   ├── class_extractor.rs
│   │   ├── function_extractor.rs
│   │   └── enum_extractor.rs
│   ├── postprocessor/          # 后处理
│   │   ├── operator_handler.rs
│   │   ├── friend_handler.rs
│   │   └── lambda_handler.rs
│   ├── generator/              # 代码生成
│   │   ├── hicc_codegen.rs
│   │   └── project_generator.rs
│   └── instantiation_tracker.rs # 模板实例化追踪
└── Cargo.toml
```

**关键依赖**：`clang = "2"` - libclang 绑定（C++ 支持）

### 3.2 三阶段处理流程

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 编译拦截 (hook.cpp)                                      │
│    LD_PRELOAD 注入 → 预处理捕获 → 编译                        │
│    输出: c/*.c2rust（宏展开后的 C++ 代码）                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. AST 提取 (ast_parser.rs + extractor/)                   │
│    clang crate 解析 .c2rust → 类/函数/模板实例化/枚举          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 代码生成 (generator/)                                    │
│    hicc 宏格式 Rust 代码 → lib.rs + <unit>.rs               │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Hook 机制

### 4.1 新建 hook.cpp（必须）

| 现有文件 | 问题 | 解决方案 |
|---------|------|---------|
| `hook/hook.c` | 只支持 C 编译器，只检测 `.c` 文件 | 合并到新 hook.cpp |
| `examples/cpp-hook/hook.cpp` | 输出 AST JSON（redirect bug） | 合并到新 hook.cpp，改为输出预处理文件 |

**新 hook.cpp 特性**：

| 特性 | 值 |
|------|---|
| 编译器检测 | `g++`, `clang++`, `c++`（支持 versioned：`g++-13`） |
| 文件扩展名 | `.cpp`, `.cc`, `.cxx`, `.c++`, `.C`, `.cp` |
| 预处理参数 | `-E -C -P`（删除行号，简化处理） |
| 输出 | `.c2rust` 宏展开文件 |

### 4.2 预处理捕获

```bash
# 预处理命令（hook.cpp 新实现）
g++ -E -C -P -I<inc_paths> -D<defs> foo.cpp -o foo.cpp.c2rust
```

**说明**：使用 `-P` 删除行号信息，简化后续解析。

### 4.3 输出目录结构

```
.c2rust/v5/
├── c/                           # 预处理文件
│   └── src/
│       └── foo.cpp.c2rust      # 宏展开后的 C++ 代码
└── targets.list                  # 链接目标列表
```

---

## 5. AST 解析

### 5.1 clang crate 解析

```rust
// ast_parser.rs
use clang::{Clang, EntityKind, Index};

pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new()?;
    let index = Index::new(&clang, false, false);

    let tu = index.parser(file)
        .detailed_preprocessing(true)
        .parse()
        .unwrap();

    for child in tu.get_entity().get_children() {
        match child.get_kind() {
            EntityKind::StructDecl | EntityKind::ClassDecl => {
                if child.get_template().is_some() {
                    /* 模板全实例化处理 */
                } else {
                    /* 普通类/结构体处理 */
                }
            }
            EntityKind::FunctionDecl => { /* 函数处理 */ }
            EntityKind::ClassTemplatePartialSpecialization => { /* 偏特化处理 */ }
            // ...
        }
    }
}
```

### 5.2 支持的 C++ AST 节点

| 节点类型（C++ 概念） | clang crate `EntityKind` | v5 用途 |
|---------------------|--------------------------|--------|
| `CXXRecordDecl` | `EntityKind::StructDecl` / `EntityKind::ClassDecl` | 类/结构体定义（含模板全实例化） |
| `CXXMethodDecl` | `EntityKind::Method` | 成员函数 |
| `FunctionDecl` | `EntityKind::FunctionDecl` | 全局函数 |
| 模板全实例化（如 `vector<int>`） | `EntityKind::ClassDecl` + `entity.get_template().is_some()` | 模板实例化结果（主要跟踪目标） |
| 模板显式偏特化 | `EntityKind::ClassTemplatePartialSpecialization` | 偏特化（次要） |
| `NamespaceDecl` | `EntityKind::Namespace` | 命名空间 |
| `EnumDecl` | `EntityKind::EnumDecl` | 枚举 |
| `CXXConstructorDecl` | `EntityKind::Constructor` | 构造函数 |
| `CXXDestructorDecl` | `EntityKind::Destructor` | 析构函数 |

> **注意**：模板全实例化（`std::vector<int>` 等）在 libclang 中被展开为普通 `ClassDecl`/`StructDecl` 节点，可通过 `entity.get_template()` 返回 `Some` 来识别其为实例化结果。`ClassTemplate` 节点（模板声明本身）在 v5 中**不处理**。

---

## 6. 输出格式

### 6.1 Rust 项目结构

```
rust_hicc/
├── Cargo.toml
└── src/
    ├── lib.rs              # 库入口
    ├── foo.rs              # 编译单元 foo
    ├── bar.rs              # 编译单元 bar
    └── baz.rs              # 编译单元 baz
```

### 6.2 Rust 文件格式（三段式）

```rust
// ========== 1. C++ 实现内联（hicc::cpp! 块） ==========
hicc::cpp! {
    #include "foo.h"

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

---

## 7. C++ 特性支持

### 7.1 总览

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

### 7.2 ⚠️ 降级处理（3 项）

| 特性 | 示例 | 处理方式 | TODO tag |
|------|------|---------|----------|
| 运算符重载 | 019 | named shim，提示可实现 std::ops trait | `[OP]` |
| 友元函数 | 020 | 直接入 import_lib! | `[FR]` |
| typeid/RTTI | 023 | 枚举注入 | `[RTTI]` |

---

## 8. 局限性及处理方案

### 8.1 局限性总览

| 限制 | 处理方案 |
|------|---------|
| 仅支持 Linux | Docker 容器方案 |
| 需要完整构建环境 | `--skip-failed` 跳过失败文件 |
| 模板实例化跨 TU | 合并多 AST 分析 |

---

## 9. 实现计划

### 9.1 Phase 顺序

| 阶段 | 内容 | 优先级 | 依赖 |
|------|------|--------|------|
| Phase 0 | Hook 机制（hook.cpp） | P0 | 新建 hook.cpp |
| Phase 1 | ast_parser.rs（C++ AST 解析） | P0 | Phase 0 |
| Phase 2 | 基础提取器（class/function/enum） | P0 | Phase 1 |
| Phase 3 | 模板实例化追踪器 | P0 | Phase 1 |
| Phase 4 | 后处理器（OP/FR/Lambda） | P1 | Phase 2 |
| Phase 5 | hicc 代码生成器 | P0 | Phase 2, 3 |
| Phase 6 | 局限性处理（Docker/增量） | P1 | Phase 1-5 |
| Phase 7 | 集成测试 | P1 | Phase 1-6 |

### 9.2 Phase 0-1 详细任务

**Phase 0 - Hook 机制（基于 `references/c2rust-demo/hook/hook.c` 改写）**：
1. [ ] 创建 `hook/hook.cpp`，基于 `references/c2rust-demo/hook/hook.c` 改写：
   - 保留预处理捕获逻辑（`-E -C -P` 选项、路径处理、环境变量等）
   - 仅修改编译器检测部分
2. [ ] 修改编译器列表：`gcc, clang, cc` → `g++, clang++, c++`（含版本后缀如 `g++-13`）
3. [ ] 修改文件扩展名支持：`.c` → `.cpp, .cc, .cxx, .c++, .C, .cp`
4. [ ] 输出预处理文件（`.c2rust`），使用 `-E -C -P`
5. [ ] 复制 `references/c2rust-demo/src/capture.rs`（直接复用，无需修改）
6. [ ] 复制 `references/c2rust-demo/hook/Makefile` 并适配（将 `hook.c` 替换为 `hook.cpp`，使用 g++ 编译）

**Phase 1 - AST 解析**：
1. [ ] 实现 `ast_parser.rs`，使用 `clang` crate
2. [ ] 支持 `CXXRecordDecl`、`CXXMethodDecl` 等 C++ 节点
3. [ ] 支持 `ClassTemplateSpecialization` 模板实例化

**验收标准**：
```bash
# 使用 clang crate 解析宏展开后的 C++ 文件
echo 'class Foo { public: int getValue(); };' | g++ -E -x c++ - > foo.c2rust
cargo run -- parse foo.c2rust
# 应输出:
# - CXXRecordDecl: Foo
#   - CXXMethodDecl: getValue
```

---

## 10. 技术依赖

### 10.1 Rust Crates

```toml
[dependencies]
clang = "2"            # libclang 绑定（C++ AST 解析）
clap = "4"             # CLI
anyhow = "1"            # 错误处理
serde = { version = "1", features = ["derive"] }
walkdir = "2"

[build-dependencies]
cc = "1"               # 编译 hook/hook.cpp → libhook.so
```

### 10.2 系统依赖

```bash
apt-get install clang libclang-dev g++ libstdc++-dev
```

---

## 11. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| hook.cpp 创建复杂 | 合并两个源码容易出错 | 基于 hook.c 修改，保留核心逻辑 |
| clang crate API 变化 | 解析失败 | 锁定版本 |
| C++ AST 节点遗漏 | 功能缺失 | 扩展解析逻辑 |
| 隐式模板跨 TU | 类型缺失 | 合并多 AST 分析 |
| 系统库展开代码庞大 | 解析慢、文件大 | 接受现状，暂不优化 |
