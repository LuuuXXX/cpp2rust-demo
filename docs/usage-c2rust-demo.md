# c2rust-demo 使用指南

## 简介

`c2rust-demo` 是一个面向 C 项目的命令行工具，用于将 C 代码构建过程捕获并生成 Rust 脚手架代码。它的核心思想是**复用现有的 C 构建系统**，通过拦截编译过程来收集类型信息和函数签名，然后生成对应的 Rust FFI 代码。

### 核心特性

1. **零侵入**：无需修改原有 C 代码
2. **构建感知**：理解 C 项目的编译参数、include 路径等
3. **自动生成**：自动生成 Rust FFI 声明和类型绑定
4. **模块化输出**：按符号或按模块组织生成的代码

### 工作流程

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   C 项目    │ ──► │   捕获阶段   │ ──► │  生成阶段   │
│  (原有代码)  │     │  (LD_PRELOAD)│     │ (bindgen)  │
└─────────────┘     └─────────────┘     └─────────────┘
                           │                    │
                           ▼                    ▼
                    ┌─────────────┐     ┌─────────────┐
                    │  .c2rust/  │     │ Rust 脚手架 │
                    │   捕获文件   │     │             │
                    └─────────────┘     └─────────────┘
```

## 环境要求

| 要求 | 说明 |
|------|------|
| **操作系统** | Linux（依赖 `LD_PRELOAD` 和 Unix 符号链接机制） |
| **Rust** | 支持 `rust-version = 1.82` 及以上 |
| **构建工具** | `gcc`、`clang`、`make` |
| **clang/libclang** | 用于 AST 解析和 bindgen |
| **bindgen-cli** | 用于生成 Rust FFI 绑定 |

### 安装依赖

```bash
# 1. 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安装 bindgen-cli
cargo install bindgen-cli

# 3. 验证 clang 安装
clang --version

# 4. 验证 make
make --version
```

## 构建 c2rust-demo

```bash
# 克隆项目
git clone <c2rust-demo-repo>
cd c2rust-demo

# 开发构建（带调试符号）
cargo build

# 发布构建（优化版本）
cargo build --release

# 运行测试
cargo test
```

构建完成后，可执行文件位于：
- 开发版：`target/debug/c2rust-demo`
- 发布版：`target/release/c2rust-demo`

## 核心命令详解

### 1. init：捕获构建过程

`init` 命令用于拦截 C 项目的构建过程，捕获编译参数和源代码信息。

#### 基本用法

```bash
# 最简用法：捕获 make 构建
c2rust-demo init -- make

# 指定并行数
c2rust-demo init -- make -j4

# 直接捕获编译器调用
c2rust-demo init -- gcc -c mylib.c -I./include

# 指定 feature 名称（用于多目标项目）
c2rust-demo init --feature static -- make
c2rust-demo init --feature shared -- make
```

#### 命令行参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `--feature <name>` | 指定 feature 名称 | `--feature myfeature` |
| `--` | 分隔符，之后是构建命令 | `-- make` |

#### 工作流程详解

`init` 命令执行以下步骤：

```
1. 准备阶段
   ├── 读取 C2RUST_PROJECT_ROOT 环境变量
   ├── 创建 .c2rust/<feature>/ 目录结构
   └── 编译 hook/hook.c 为 libhook.so

2. 捕获阶段
   ├── 设置 LD_PRELOAD=libhook.so
   ├── 执行构建命令
   └── hook 拦截编译器调用，生成 .c2rust 文件

3. 选择阶段
   ├── 读取捕获到的文件列表
   ├── 交互式选择要转换的文件（可跳过）
   └── 生成 selected_files.json

4. 生成阶段
   ├── 对每个选中的 .c2rust 文件运行 bindgen
   ├── 生成 Rust FFI 声明
   └── 生成报告文件
```

#### 输出文件

执行成功后，在 `.c2rust/<feature>/` 目录下生成：

```
.c2rust/<feature>/
├── c/                          # 捕获的 C 文件
│   ├── mylib.c2rust           # 预处理后的 C 源码
│   ├── mylib.c2rust.opts     # 编译选项
│   └── targets.list           # 链接目标列表
│
├── meta/                       # 元数据
│   ├── build_cmd.txt         # 构建命令记录
│   ├── selected_files.json    # 选中的文件列表
│   └── init-interface-report.md  # 接口报告
│
└── rust/                       # 生成的 Rust 代码
    ├── Cargo.toml
    └── src/
        ├── lib.rs             # 库入口
        ├── lib.normalized
        └── mod_*/            # 按模块组织的代码
            ├── mod.rs
            ├── fun_*.rs      # 函数绑定
            ├── var_*.rs      # 变量绑定
            └── decl_*.rs     # 类型声明
```

### 2. merge：合并和组织代码

`merge` 命令将 `init` 生成的分散文件合并为更有组织的结构。

#### 基本用法

```bash
# 合并默认 feature
c2rust-demo merge

# 合并指定 feature
c2rust-demo merge --feature static
```

#### 工作流程详解

```
1. 读取阶段
   ├── 扫描 rust/src/mod_*/ 目录
   └── 收集所有 fun_*.rs 和 var_*.rs 文件

2. 分析阶段
   ├── 识别跨模块重复的 FFI 声明
   ├── 分析函数和变量的依赖关系
   └── 生成去重策略

3. 合并阶段
   ├── 将同模块的 fun_*.rs 合并为 mod.rs
   ├── 将跨模块重复的 FFI 上提到 lib.rs
   ├── 去重重复的类型声明

4. 输出阶段
   ├── 输出到 rust/src.2/
   ├── 备份原输出到 rust/src.1/
   └── 创建 rust/src -> rust/src.2 符号链接
```

#### 合并后的结构

```
rust/
├── src.1/                      # init 原始输出（备份）
├── src.2/                      # 合并后的输出
│   ├── lib.rs                 # 主库文件（含公共 FFI）
│   └── mod_*/                 # 模块目录
│       └── mod.rs             # 模块文件
└── src -> src.2/             # 符号链接，指向 src.2
```

## 环境变量

c2rust-demo 通过环境变量控制行为：

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `C2RUST_PROJECT_ROOT` | C 项目根目录 | 必填 |
| `C2RUST_FEATURE_ROOT` | 输出目录 | 必填 |
| `C2RUST_CC` | C 编译器名称 | gcc/clang/cc |
| `C2RUST_LD` | 链接器名称 | ld/lld |
| `C2RUST_CLANG` | clang 可执行文件名 | clang |
| `C2RUST_REMOVE_STATIC` | 设为非空时移除 static/inline | 空 |
| `C2RUST_DEBUG` | 设为非空时输出调试日志 | 空 |

### 使用示例

```bash
# 基本用法
export C2RUST_PROJECT_ROOT=/path/to/c/project
export C2RUST_FEATURE_ROOT=/path/to/output
c2rust-demo init -- make

# 使用 clang
export C2RUST_CC=clang
c2rust-demo init -- make

# 调试模式
export C2RUST_DEBUG=1
c2rust-demo init -- make 2>&1 | grep "c2rust-hook"
```

## Hook 机制详解

hook 是 c2rust-demo 的核心组件，通过 `LD_PRELOAD` 技术拦截系统调用。

### 技术原理

```
正常编译流程：
  gcc ──► 读取 .c 文件 ──► 预处理 ──► 编译 ──► 输出 .o

hook 拦截后：
  gcc ──► hook.so (LD_PRELOAD) ──► 读取 .c 文件
                     │
                     ├── 复制源码到 .c2rust/
                     ├── 保存编译选项到 .c2rust.opts
                     └── 继续正常编译流程
```

### hook.c 关键函数

| 函数 | 说明 |
|------|------|
| `is_compiler()` | 判断是否为编译器调用 |
| `discover_cfile()` | 发现并处理 .c 文件 |
| `preprocess_cfile()` | 生成预处理后的 .c2rust 文件 |
| `discover_target()` | 记录链接目标 |
| `c2rust_hook()` | 库构造函数，入口点 |

### hook 编译

```bash
cd hook
make
# 生成 libhook.so
```

## 端到端示例：转换 cJSON

下面以 cJSON 库为例，演示完整转换流程。

### 1. 准备 cJSON

```bash
# 克隆 cJSON
git clone https://github.com/DaveGamble/cJSON.git
cd cJSON

# 查看项目结构
ls -la
# cJSON.c  cJSON.h  Makefile  ...
```

### 2. 执行 init

```bash
# 设置环境变量
export C2RUST_PROJECT_ROOT=$(pwd)
export C2RUST_FEATURE_ROOT=$(pwd)/.c2rust/default

# 初始化捕获
c2rust-demo init -- make

# 输出示例
[init] Building hook.so...
[init] Running with LD_PRELOAD=...
[init] Captured 1 .c file
[init] Generating Rust bindings...
[init] Done! Output in .c2rust/default/rust/
```

### 3. 检查生成结果

```bash
# 查看生成的 Rust 代码
cat .c2rust/default/rust/src/lib.rs

# 查看接口报告
cat .c2rust/default/meta/init-interface-report.md
```

### 4. 执行 merge（可选）

```bash
c2rust-demo merge

# 查看合并后的结构
ls -la .c2rust/default/rust/src.2/
```

### 5. 在 Rust 中使用

```toml
# Cargo.toml
[dependencies]
cjson = { path = "cjson/.c2rust/default/rust" }
```

```rust
use cjson::cJSON;

fn main() {
    // 使用生成的 cJSON bindings
    let json_str = r#"{"name": "test", "value": 42}"#;
    let json = cJSON::parse(json_str);
    println!("Parsed: {:?}", json);
}
```

## 项目结构

```
c2rust-demo/
├── src/
│   ├── main.rs           # CLI 入口
│   ├── capture.rs        # hook 构建和执行逻辑
│   ├── layout.rs         # 目录和元数据管理
│   ├── selector.rs       # 交互式文件选择
│   └── split/
│       ├── mod.rs        # 模块拆分
│       ├── feature.rs    # init 阶段生成逻辑
│       ├── merge.rs      # merge 阶段合并逻辑
│       └── file.rs       # .c2rust 文件解析
│
├── hook/
│   ├── hook.c            # LD_PRELOAD 库源码
│   └── Makefile          # hook 编译配置
│
├── tests/
│   ├── integration.rs    # 集成测试
│   └── fixtures/         # 测试用例
│       └── simple/
│           ├── src/
│           │   ├── math.c
│           │   └── counter.c
│           └── Makefile
│
├── scripts/
│   └── validate-cjson.sh # cJSON 完整验证脚本
│
├── Cargo.toml
└── README.md
```

## 测试

### 运行全部测试

```bash
cargo test
```

### 运行特定测试

```bash
# 仅运行集成测试
cargo test --test integration

# 运行单元测试
cargo test --lib

# 运行带日志的测试
RUST_LOG=debug cargo test
```

### 集成测试说明

集成测试会检测环境中的必要工具：

```
$ cargo test --test integration

running 1 test
test integration_test ... ok

External tool checks:
  - gcc: found at /usr/bin/gcc
  - make: found at /usr/bin/make
  - clang: found at /usr/bin/clang
  - bindgen: found
```

如果缺少工具，测试会跳过并显示警告：

```
test integration_test ... SKIPPED
Reason: Missing required tool: bindgen
```

## 高级用法

### 1. 多 feature 管理

```bash
# 为不同目标生成不同的 bindings
c2rust-demo init --feature static -- make
c2rust-demo init --feature shared -- make

# 合并特定 feature
c2rust-demo merge --feature static
```

### 2. 处理混合 C/C++ 项目

```bash
# hook 会自动识别 .c 和 .cpp 文件
c2rust-demo init -- make

# 对于 C++ 部分，需要额外处理
# 参考 hicc 项目进行 C++ FFI 绑定
```

### 3. 增量转换

```bash
# 首次完整转换
c2rust-demo init -- make

# 修改部分 C 文件后，只重新处理修改的文件
# 编辑 .c2rust/default/meta/selected_files.json
# 删除对应的 rust/src/mod_*/ 中的旧文件
c2rust-demo init -- make
```

### 4. 自定义 bindgen 选项

通过 `.c2rust.opts` 文件传递额外选项：

```
clang -c file.c -I./include -DDESIRED_FLAG
```

### 5. 调试 hook

```bash
# 启用调试输出
export C2RUST_DEBUG=1

# 运行 init
c2rust-demo init -- make 2>&1 | grep -E "(c2rust-hook|DEBUG)"
```

## 注意事项

1. **平台限制**：目前仅支持 Linux，不支持 macOS 或 Windows

2. **编译器限制**：
   - 支持 gcc、clang、cc
   - 不支持 MSVC（Windows）

3. **C 标准库**：hook 只处理用户代码，不处理系统头文件

4. **静态库处理**：
   - 链接的静态库（lib*.a）会被记录
   - 但不会解析静态库中的符号

5. **符号链接**：`merge` 会修改 `rust/src` 为符号链接，如需查看原始文件，查看 `rust/src.1`

6. **弱符号函数**：带 `__attribute__((weak))` 的函数会自动去重

7. **inline 函数**：默认 inline 函数会被展开，如需保留设置 `C2RUST_REMOVE_STATIC` 为空

## 常见问题

### Q: 提示 "C2RUST_PROJECT_ROOT not set"

```bash
# 必须设置这两个环境变量
export C2RUST_PROJECT_ROOT=/path/to/c/project
export C2RUST_FEATURE_ROOT=/path/to/output/.c2rust/default
```

### Q: hook 没有拦截到任何文件

```bash
# 确认环境变量设置正确
echo $C2RUST_PROJECT_ROOT

# 确认是编译 .c 文件
c2rust-demo init -- gcc -c test.c

# 使用调试模式
export C2RUST_DEBUG=1
c2rust-demo init -- gcc -c test.c 2>&1
```

### Q: 生成的 Rust 代码编译失败

可能原因：
1. bindgen 版本不兼容
2. C 代码中有复杂的宏
3. 缺少必要的 include 路径

解决方案：
```bash
# 升级 bindgen
cargo install bindgen-cli

# 检查并添加 include 路径
c2rust-demo init -- gcc -c file.c -I./include
```

### Q: 如何处理大型 C 项目？

建议：
1. 使用 `--feature` 分批处理
2. 先转换核心模块
3. 逐步扩展

### Q: 生成的代码如何使用？

```bash
# 1. 查看生成的 Cargo.toml
cat .c2rust/default/rust/Cargo.toml

# 2. 在自己的 Rust 项目中引用
[dependencies]
my_lib = { path = "path/to/.c2rust/default/rust" }

# 3. 调用生成的 FFI 函数
extern crate my_lib;
use my_lib::*;
```
