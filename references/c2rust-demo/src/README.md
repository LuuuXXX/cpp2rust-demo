# c2rust-demo 源码模块说明

本文档详细介绍 `c2rust-demo` 各源码模块的功能、职责与使用方式。

## 模块概览

```text
src/
├── main.rs          # CLI 入口（init / merge 子命令）
├── capture.rs       # 构建捕获（libhook.so + LD_PRELOAD 注入）
├── error.rs         # 统一错误类型
├── layout.rs        # 目录结构管理（FeatureLayout）
├── selector.rs      # 交互式文件选择
└── split/
    ├── mod.rs       # split 子模块入口
    ├── feature.rs   # init 阶段的 Rust 脚手架生成
    ├── merge.rs     # merge 阶段的 FFI 合并与去重
    └── file.rs      # 单个翻译单元文件的处理
```

---

## `main.rs` — CLI 入口

**职责**：解析命令行参数，驱动 `init` 和 `merge` 两个子命令的完整流程。

### `init` 子命令流程

```text
1. 确定项目根目录（向上搜索 .c2rust/）
2. 调用 capture::build_hook() 编译 libhook.so
3. 调用 capture::run_with_hook() 注入构建并捕获 .c2rust 文件
4. 交互式选择参与转换的翻译单元（selector）
5. 为每个翻译单元调用 split::feature 生成 Rust 脚手架
6. 输出 init-interface-report.md
```

### `merge` 子命令流程

```text
1. 确定项目根目录
2. 读取 init 阶段的输出（rust/src/）
3. 调用 split::merge 合并去重
4. 生成 rust/src.2 并将 rust/src 符号链接指向它
5. 输出 merge-interface-report.md
```

### 命令行参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--feature <name>` | `default` | Feature 名称（输出目录隔离用） |
| `-- <构建命令...>` | 必填（init） | 实际的 C 构建命令 |

---

## `capture.rs` — 构建捕获

**职责**：编译 `hook/libhook.so` 并通过 `LD_PRELOAD` 将其注入用户的构建过程，捕获每个编译命令及其参数。

### 主要函数

#### `build_hook() -> Result<PathBuf>`

- 定位 `hook/` 目录（与 `cpp2rust-demo` 二进制文件相邻）
- 执行 `make -s` 编译 `libhook.so`
- 返回 `libhook.so` 的路径

#### `run_with_hook(build_dir, cmd, project_root, feature_root, hook_so) -> Result<()>`

- 设置环境变量 `LD_PRELOAD=<path_to_libhook.so>`
- 设置 `C2RUST_PROJECT_ROOT` 和 `C2RUST_FEATURE_ROOT`
- 执行用户构建命令（透传 stdout/stderr）

### 钩子捕获产物

每次编译命令触发时，`libhook.so` 在 `.c2rust/<feature>/c/` 下生成：

- `<hash>.c2rust`：Clang AST JSON 导出
- `<hash>.c2rust.opts`：编译选项

### 环境变量

| 变量 | 说明 |
|------|------|
| `C2RUST_CC` | 钩子识别的编译器名称（默认匹配 `gcc/clang/cc`） |
| `C2RUST_LD` | 钩子识别的链接器名称 |
| `C2RUST_DEBUG` | 非空时输出钩子调试日志到 stderr |
| `C2RUST_REMOVE_STATIC` | 非空时启用 static/inline 函数公开化 |

---

## `error.rs` — 错误处理

**职责**：定义统一的 `Result<T>` 类型别名，基于 `anyhow::Error`。

```rust
pub type Result<T> = anyhow::Result<T>;
```

所有模块的错误均使用 `anyhow::anyhow!()` 宏创建，统一通过 `?` 传播。

---

## `layout.rs` — 目录结构管理

**职责**：封装 `.c2rust/<feature>/` 目录结构，提供目录路径的统一访问点。

### `FeatureLayout` 结构体

```rust
pub struct FeatureLayout {
    pub project_root: PathBuf,
    pub feature_name: String,
    pub feature_root: PathBuf,  // .c2rust/<feature>/
    pub c_dir: PathBuf,         // .c2rust/<feature>/c/
    pub rust_dir: PathBuf,      // .c2rust/<feature>/rust/
    pub meta_dir: PathBuf,      // .c2rust/<feature>/meta/
}
```

### 目录发现

`find_project_root(start: &Path) -> PathBuf`：从当前目录向上搜索含 `.c2rust/` 目录的项目根，未找到时降级为当前目录。

### 主要方法

| 方法 | 说明 |
|------|------|
| `new(project_root, feature_name)` | 构造路径布局（不创建目录） |
| `create_dirs()` | 创建所有必要目录 |
| `rust_src_dir()` | 返回 `rust/src/` 路径（init 的输出目录） |

---

## `selector.rs` — 翻译单元选择器

**职责**：在 `init` 阶段提供交互式 UI 供用户选择哪些翻译单元（C 源文件）参与转换。

### 行为

- **交互环境**（TTY）：使用 `dialoguer` 库提供多选 UI
- **非交互环境**（CI/管道）：自动全选所有翻译单元（full-capture 原则）

### `InteractiveSelector`

```rust
pub struct InteractiveSelector;

impl InteractiveSelector {
    pub fn select(files: &[PathBuf]) -> Result<Vec<PathBuf>>;
}
```

- 非交互时：直接返回 `files.to_vec()`
- 交互时：展示复选框列表，用户确认后返回选中的文件列表

---

## `split/` — 代码生成子系统

### `split/mod.rs` — 子模块入口

导出 `feature`、`merge`、`file` 三个子模块的公开接口。

---

### `split/feature.rs` — init 阶段脚手架生成

**职责**：解析捕获的 AST JSON，为每个翻译单元生成 hicc 风格的 Rust FFI 脚手架。

#### 核心功能

1. **AST 解析**：读取 `.c2rust/<feature>/c/` 下的 JSON 文件
2. **符号提取**：从 `FunctionDecl`、`CXXRecordDecl`、`VarDecl` 等节点提取公开 C/C++ 符号
3. **Rust 代码生成**：生成 `import_class!`、`import_lib!` 脚手架
4. **报告生成**：输出 `init-interface-report.md`，列出所有生成的接口

#### 输出结构

```text
rust/src/
├── lib.rs              # crate 入口，汇总所有模块
└── mod_<name>/
    ├── mod.rs          # 模块入口
    ├── fun_<func>.rs   # 每个函数的 import_lib! 脚手架
    └── var_<var>.rs    # 全局变量的 import_lib! 脚手架
```

---

### `split/merge.rs` — merge 阶段合并

**职责**：将 init 阶段按符号拆分的文件合并为按模块组织的文件，并跨模块去重重复的 FFI 声明。

#### 核心功能

1. **重复声明去重**：同一结构体/函数在多个翻译单元中重复声明时，提升到 `lib.rs`
2. **模块合并**：将 `mod_*/fun_*.rs` 合并为 `mod_*.rs`
3. **符号链接管理**：将 `rust/src` 更新为指向 `src.2` 的符号链接，保留 `src.1` 备份
4. **报告生成**：输出 `merge-interface-report.md`

#### 合并规则

| 场景 | 处理方式 |
|------|---------|
| 单 TU 中的声明 | 保留在对应 `mod_*.rs` 中 |
| 多 TU 共享的声明 | 提升到 `lib.rs` |
| 完全相同的签名 | 只保留一份 |
| 同名不同签名 | 生成注释提示，保留所有版本 |

---

### `split/file.rs` — 单文件处理

**职责**：处理单个翻译单元的 AST JSON 文件，提取符号并生成对应的 Rust 文件。

---

## 数据流

```text
用户构建命令
    │
    ▼ (LD_PRELOAD = libhook.so)
libhook.so 捕获每条编译命令
    │
    ▼
.c2rust/<feature>/c/
├── aaa.c2rust       (AST JSON)
├── aaa.c2rust.opts  (编译选项)
├── bbb.c2rust
└── ...
    │
    ▼ (split/feature.rs)
.c2rust/<feature>/rust/src/
├── lib.rs
└── mod_*/
    ├── fun_*.rs
    └── var_*.rs
    │
    ▼ (split/merge.rs)
.c2rust/<feature>/rust/src.2/
├── lib.rs        (合并后，含共享声明)
└── mod_*.rs      (合并后，按模块组织)
```
