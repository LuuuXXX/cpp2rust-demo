# c2rust-demo 功能详解

## 概述

`c2rust-demo` 是一个面向 C 项目的命令行工具，用于将 C 代码逐步迁移到 Rust。该工具通过"构建捕获"的方式，无需修改原始 C 代码，即可生成与原始 C 代码结构对应的 Rust 脚手架代码。

项目地址：`./references/c2rust-demo`

## 核心功能

### 1. init 命令：构建捕获与 Rust 脚手架生成

`init` 命令执行两步核心操作：构建捕获和 Rust 项目初始化。

#### 构建捕获阶段

1. **编译 Hook 库**
   - `c2rust-demo` 使用 `LD_PRELOAD` 技术注入一个共享库 `libhook.so`
   - Hook 库位于 `hook/` 目录，由 `hook.c` 和 `Makefile` 构建
   - Hook 拦截编译器和链接器调用，捕获构建过程中的所有 C 源文件信息

2. **环境变量配置**
   - `C2RUST_PROJECT_ROOT`：项目根目录（包含 `.c2rust/` 的目录）
   - `C2RUST_FEATURE_ROOT`：特征根目录，捕获产物存放位置
   - `C2RUST_CC`：指定编译器名称（默认识别 gcc/clang/cc 及其版本后缀如 gcc-13）
   - `C2RUST_LD`：指定链接器名称（默认识别 ld/lld）
   - `C2RUST_DEBUG`：设为非空时输出 Hook 调试日志到 stderr

3. **捕获机制** (`hook/hook.c`)
   - Hook 通过 `LD_PRELOAD` 加载到编译器进程
   - 使用库构造函数 `__attribute__((constructor))` 在进程启动时自动执行
   - 读取 `/proc/self/cmdline` 获取实际编译命令参数
   - 拦截编译器调用，提取编译参数（`-I`, `-D`, `-U`, `-include`, `-isystem`, `-iquote`, `-std=`, `-fshort-enums`）
   - 对每个 C 源文件执行预处理，生成 `.c2rust` 文件（后缀从 `.c` 改为 `.c2rust`）
   - 预处理选项保存到对应的 `.c2rust.opts` 文件
   - 拦截链接器调用，记录链接目标文件和静态库到 `targets.list`

4. **C 文件预处理输出**
   - 每个 C 源文件预处理后生成 `.c2rust` 文件
   - 文件路径结构：`${C2RUST_FEATURE_ROOT}/c/${相对于项目根的路径}2rust`
   - 例如：`src/foo.c` → `.c2rust/<feature>/c/src/foo.c2rust`

#### Rust 项目初始化阶段

1. **文件选择** (`src/selector.rs`)
   - 支持交互式选择：使用 `dialoguer` 库显示多选菜单
   - 非交互环境（CI/管道）自动选择所有文件
   - 选中的文件列表保存到 `meta/selected_files.json`

2. **Rust 项目创建**
   - 使用 `cargo new --lib` 创建新的 Rust 库项目
   - 修改 `Cargo.toml`：设置 `crate-type = ["staticlib"]`
   - 生成 `lib.rs` 和 `lib.normalized`（格式化前的备份）

3. **模块生成** (`src/split/feature.rs`)
   - 每个 C 源文件对应一个 Rust 模块（`mod_xxx`）
   - 模块名生成规则：将 C 文件路径中的非字母数字字符替换为下划线，前缀加 `mod_`
   - 例如：`src/foo.c` → `mod_src_foo`

4. **类型生成** (`src/split/file.rs`)
   - 使用 `bindgen` 生成 Rust FFI 类型声明
   - 为每个模块生成 `types.h` 头文件（包含所有类型定义）
   - `bindgen` 配置参数：
     - `--no-layout-tests`：禁用布局测试
     - `--default-enum-style consts`：枚举生成常量而非 Rust 枚举
     - `--no-prepend-enum-name`：枚举成员不使用枚举名作为前缀
     - `--disable-nested-struct-naming`：禁用嵌套结构体命名
     - `--ctypes-prefix ::core::ffi`：使用 `core::ffi` 作为 C 类型前缀
   - 生成 `mod.rs`：包含所有类型定义和 FFI 外部块声明

5. **符号处理**
   - **弱符号处理**：同名弱符号（weak symbol）仅保留一个，强符号优先
   - **静态符号公开化**：可选功能，设置 `C2RUST_REMOVE_STATIC=1` 启用
     - 静态函数/变量重命名为 `_c2rust_private_${md5}_${原名}`
     - 生成宏定义使原符号名指向新名称
   - **可变参数函数**：跳过不处理
   - **重复定义处理**：去重重复的函数声明

6. **初始化报告** (`meta/init-interface-report.md`)
   - 记录每个模块的函数和变量信息
   - 包含：原始 C 名称、Rust 符号名、生成的文件路径、FFI 声明

#### init 输出目录结构

```
.c2rust/<feature>/
├── c/                          # 捕获的 .c2rust 文件
│   ├── src/
│   │   └── foo.c2rust          # 预处理后的 C 代码
│   └── src/
│       └── foo.c2rust.opts     # 编译选项
├── meta/
│   ├── build_cmd.txt           # 原始构建命令
│   ├── selected_files.json      # 选中的文件列表
│   └── init-interface-report.md # 初始化接口报告
└── rust/                       # 生成的 Rust 项目
    ├── Cargo.toml
    └── src/
        ├── lib.rs              # 库入口
        ├── lib.normalized      # 格式化前的备份
        └── mod_<name>/         # 每个 C 文件一个模块
            ├── mod.rs          # 类型定义和 FFI 声明
            ├── mod.normalized
            ├── types.h         # bindgen 用的临时头文件
            ├── fun_*.rs        # 函数实现骨架
            ├── fun_*.c        # 对应的 C 代码片段
            ├── var_*.rs       # 变量声明骨架
            ├── var_*.c        # 对应的 C 代码片段
            └── decl_*.rs       # 独立声明文件
```

### 2. merge 命令：按符号文件合并为按模块文件

`merge` 命令将 `init` 阶段生成的细粒度 Rust 文件（每个符号一个文件）合并为每个模块一个文件。

#### 合并流程

1. **目录发现**
   - 扫描 `rust/src/mod_*` 目录
   - 在每个模块目录中，通过文件名前缀识别符号文件（`fun_*.rs` 和 `var_*.rs`）
   - **关键适配**：不使用 `mod.rs` 中的 `mod fun_*;` 声明，而是直接扫描文件系统

2. **模块文件合并**
   - 读取 `mod.rs` 中的所有项（类型定义、FFI 块等）
   - 依次读取每个 `fun_*.rs` 和 `var_*.rs` 文件
   - 将所有项合并到一个文件中
   - 自动去重 `use super::*;` 导入

3. **FFI 去重**
   - 跨模块检测重复的 FFI 声明（函数和静态变量）
   - 重复的 FFI 声明上移到 `lib.rs`
   - 各模块保留仅属于自己的 FFI 声明

4. **输出目录管理**
   - 合并结果写入 `rust/src.2/`
   - 原始 `rust/src` 重命名为 `rust/src.1`（备份）
   - `rust/src` 成为指向 `rust/src.2` 的符号链接

5. **合并报告** (`meta/merge-interface-report.md`)
   - 汇总统计：模块数、函数总数、变量总数、局部 FFI 数、共享 FFI 数
   - 列出上移到 `lib.rs` 的共享 FFI
   - 每个模块详情：最终函数列表、最终变量列表、模块局部 FFI、源文件列表

#### merge 输出目录结构

```
.c2rust/<feature>/
├── meta/
│   └── merge-interface-report.md  # 合并接口报告
└── rust/
    ├── src.1/                    # init 原始输出备份
    │   └── ...
    ├── src -> src.2              # 符号链接
    └── src.2/                    # 合并后的输出
        ├── lib.rs                # 包含模块声明和共享 FFI
        └── mod_xxx.rs            # 每个模块一个合并后的文件
```

## 技术实现细节

### Hook 机制 (`hook/hook.c`)

1. **编译器识别**
   - 支持 `gcc`, `clang`, `cc` 及其版本后缀（如 `gcc-13`）
   - 支持通过 `C2RUST_CC` 环境变量指定
   - 使用 basename 匹配，支持绝对路径调用

2. **预处理执行**
   - Fork 新进程执行预处理
   - 使用 `clang` 的 `-E -C -P` 参数
   - `-P` 避免生成行号信息，混合构建时定位信息指向新生成的文件

3. **路径处理**
   - `strip_prefix`：去除项目根路径前缀，生成相对路径
   - 目录自动创建（`mkdir -p`）

### AST 解析 (`src/split/file.rs`)

1. **Clang AST JSON 格式**
   - 使用 `clang-ast` 库解析 Clang 导出的 JSON AST
   - 支持的节点类型：
     - `TranslationUnitDecl`：翻译单元
     - `FunctionDecl`：函数声明/定义
     - `VarDecl`：变量声明/定义
     - `TypedefDecl`：类型定义
     - `EnumDecl`：枚举声明/定义
     - `RecordDecl`：结构体/联合体声明/定义

2. **节点属性**
   - `name`：符号名称
   - `loc`/`range`：源代码位置信息
   - `type`：Clang 类型信息（`qualType`, `desugaredQualType`）
   - `storageClass`：存储类别（`static`, `extern`, `register` 等）
   - `init`：初始化值
   - `isImplicit`：隐式生成
   - `git_commit`：已提交标记

3. **行号信息初始化**
   - 修正展开位置信息（宏展开后的真实位置）
   - 处理 `typedef struct Foo {} Foo` 类型包含问题

### Rust 代码生成

1. **lib.rs 属性**
```rust
// 对应__attribute__((weak))弱链接符号.
// 构建环境需要设置变量RUSTC_BOOTSTRAP=1
#![feature(linkage)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(improper_ctypes)]
#![allow(unused_imports)]
#![allow(dead_code)]
```

2. **C 标准类型别名**
```rust
use ::core::ffi::*;
type c_size_t = usize;
type c_ssize_t = isize;
// ... 更多类型别名
```

3. **指针类型转换**
   - `const T*` → `Option<&T>`
   - `T*` → `Option<&mut T>`

4. **FFI 外部块标记**
   - 使用 `#[link_name = "..."]` 属性指定原始 C 符号名
   - 私有符号重命名：`_c2rust_private_${md5}_${name}` → `name`

### Feature 多版本支持

`c2rust-demo` 支持通过 `--feature` 参数管理多个独立的转换版本：

- `c2rust-demo init --feature foo -- make`：为 `foo` feature 执行初始化
- `c2rust-demo merge --feature foo`：合并 `foo` feature
- 不同 feature 共享同一个 `.c2rust/` 目录结构，每个 feature 有独立子目录

## 依赖工具

- **Rust / Cargo**：编译 `c2rust-demo` 本身（需要 Rust 1.82+）
- **gcc**：C 代码编译
- **make**：构建系统
- **clang**：AST 导出和预处理
- **bindgen-cli**：生成 Rust FFI 绑定（需单独安装：`cargo install bindgen-cli`）
- **libclang**：bindgen 的依赖

## 使用示例

### 基本迁移流程

```bash
# 1. 进入 C 项目目录
cd /path/to/c-project

# 2. 初始化（捕获构建并生成 Rust 脚手架）
c2rust-demo init -- make

# 3. 查看生成的文件
ls -la .c2rust/default/

# 4. 合并按符号文件为按模块文件
c2rust-demo merge

# 5. 查看最终输出
ls -la .c2rust/default/rust/src/
```

### 多 feature 管理

```bash
# 为不同模块分别创建 feature
c2rust-demo init --feature core -- make -C core
c2rust-demo init --feature utils -- make -C utils

# 分别合并
c2rust-demo merge --feature core
c2rust-demo merge --feature utils
```

### 调试

```bash
# 启用 hook 调试日志
C2RUST_DEBUG=1 c2rust-demo init -- make

# 指定编译器
C2RUST_CC=gcc-13 c2rust-demo init -- make

# 启用静态符号公开化
C2RUST_REMOVE_STATIC=1 c2rust-demo init -- make
```

## 测试

项目包含单元测试和集成测试：

```bash
# 运行所有测试
cargo test

# 仅运行集成测试
cargo test --test integration

# 运行特定测试
cargo test -- test_name
```

集成测试会自动检测外部工具（gcc, make, clang, bindgen），缺失时打印跳过信息。

## 与 c2rust-code-analyse 的关系

`c2rust-demo` 是从 `c2rust-code-analyse` 项目精简而来，保留了核心的构建捕获和 Rust 脚手架生成功能，移除了 `update`、`reinit`、`sync` 等高级功能。

关键适配：
- 使用目录扫描发现符号模块，而非解析 `mod.rs` 声明
- 精简的 `merge` 实现
- 更少的外部依赖
