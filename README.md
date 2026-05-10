# cpp2rust-demo

`cpp2rust-demo` 是一个面向 C++ 项目的命令行工具，能够从真实构建过程中捕获头文件并生成
[hicc](https://github.com/LuuuXXX/hicc) Rust FFI 脚手架。

当前提供两步流程：

1. `init`：捕获 C++ 构建过程，通过交互式头文件选择，解析 clang AST，生成 Rust FFI 代码。
2. `merge`：将 `init` 产出的多个 `ffi_*.rs` 文件合并为单一的 `merged_ffi.rs`。

---

## 当前能力范围

- ✅ 已实现：`init`、`merge`
- ❌ 未实现：`update`、`reinit`、`sync`

支持的 C++ 特性：

| 特性 | 状态 |
|------|------|
| 自由函数（含命名空间） | ✅ 支持 |
| 函数重载（自动添加数字后缀） | ✅ 支持 |
| 类的公有实例方法 | ✅ 支持 |
| `const` 方法 | ✅ 支持（映射为 `&self`） |
| `static` 方法 | ✅ 支持（映射为自由函数） |
| 构造函数 / 析构函数 | ⚠️ 跳过，建议用工厂函数替代 |
| 虚方法 / 纯虚方法 | ⚠️ 跳过，需手工补充 |
| 继承 | ❌ 尚未支持 |
| 模板 | ❌ 尚未支持 |
| 运算符重载 | ❌ 尚未支持（hicc 限制） |

---

## 核心流程

```text
C++ 项目目录
   │
   ├─ cpp2rust-demo init --link <lib> -- <构建命令>
   │    ├─ 编译 hook/libhook.so
   │    ├─ 通过 LD_PRELOAD 注入构建过程，捕获涉及的头文件
   │    │    └─ 写入 .cpp2rust/<feature>/meta/captured_headers.list
   │    ├─ 交互式选择参与生成的头文件（非交互环境自动全选）
   │    │    └─ 写入 .cpp2rust/<feature>/meta/selected_headers.json
   │    ├─ 调用 clang 解析每个选中头文件的 AST
   │    │    └─ 写入 .cpp2rust/<feature>/ast/<header>.ast.json
   │    └─ 生成 Rust FFI 脚手架
   │         └─ 写入 .cpp2rust/<feature>/rust/src/ffi_<header>.rs
   │
   └─ cpp2rust-demo merge [--feature <name>]
        ├─ 读取 .cpp2rust/<feature>/rust/src/ffi_*.rs
        ├─ 合并所有 import_class! / import_lib! 块
        └─ 输出 .cpp2rust/<feature>/rust/src/merged_ffi.rs
```

---

## 项目结构（关键文件）

- `src/main.rs`：CLI 入口（`init` / `merge`）
- `src/capture.rs`：hook 构建与带 `LD_PRELOAD` 的构建命令执行
- `src/layout.rs`：`.cpp2rust/<feature>/` 目录与元数据管理
- `src/selector.rs`：交互式头文件选择（`dialoguer`）
- `src/ast.rs`：clang AST 解析与声明提取
- `src/codegen.rs`：Rust FFI 代码生成（`hicc` 格式）
- `src/merge.rs`：`merge` 阶段合并逻辑
- `hook/`：`libhook.so` 源码与 Makefile
- `tests/`：单元测试 + 集成测试
- `examples/`：示例项目

---

## 环境要求

- **Linux**（`LD_PRELOAD` 和编译时 hook 依赖 Linux）
- **Rust / Cargo**（`rust-version = 1.82`，见 `Cargo.toml`）
- **clang**（用于 AST 解析，需要 `clang` 命令可用）
- **gcc + make**（用于编译 `hook/libhook.so`）

---

## 构建

```bash
git clone https://github.com/LuuuXXX/cpp2rust-demo.git
cd cpp2rust-demo
cargo build
```

发布构建：

```bash
cargo build --release
```

查看帮助：

```bash
cargo run -- --help
```

---

## 使用方式

### 1) init：捕获构建并生成初始 Rust FFI 结构

在目标 C++ 项目根目录执行：

```bash
cpp2rust-demo init --link <库名> -- <构建命令>
```

**`--link` 为必填参数**，指定 Rust 侧要链接的库名（对应 `hicc::import_lib!` 中的 `link_name`）。

常用示例：

```bash
# 使用 make 构建
cpp2rust-demo init --link mylib -- make -j4

# 指定 feature 名，使用 cmake 构建
cpp2rust-demo init --feature myfeature --link mylib -- cmake --build build

# 直接用 clang 解析单个头文件（无需完整构建系统）
cpp2rust-demo init --link mylib -- clang -x c++ -fsyntax-only mylib.hpp

# 构建命令需要 shell 语法时
cpp2rust-demo init --link mylib -- sh -c "make -j4 && echo done"
```

完整选项：

```text
cpp2rust-demo init [OPTIONS] -- <BUILD_CMD...>

Options:
  --feature <FEATURE>          特性名，默认 default
  --link <LINK>                链接库名（必填）
  --extra-clang-args <ARGS>    传给 clang 的额外参数（如 -std=c++17 -I./include）
  --clang <CLANG>              clang 可执行文件路径，默认读取 CPP2RUST_CLANG 环境变量或 clang
```

`init` 执行步骤：

1. 编译 `hook/libhook.so`。
2. 以 `LD_PRELOAD=hook/libhook.so` 运行构建命令，捕获所有涉及的项目头文件。
3. 展示交互式多选菜单，让用户选择参与生成的头文件：
   ```
   ? Select headers to include in this feature (space to toggle, enter to confirm) ›
   ✔ /path/to/mylib.hpp
   ✔ /path/to/utils.hpp
     /path/to/internal.hpp
   ```
   - 按 `Space` 切换，按 `Enter` 确认；默认全部选中。
   - stdin 非终端时（CI / 管道）自动全选。
4. 对每个选中头文件调用 clang 解析 AST，提取函数与类声明。
5. 生成 `ffi_<header>.rs` 并写入 `rust/src/`。

### 2) merge：合并多个 FFI 文件为单一文件

```bash
cpp2rust-demo merge
cpp2rust-demo merge --feature myfeature
```

`merge` 会读取 `init` 生成的所有 `ffi_*.rs`，将其中的 `import_class!` 和 `import_lib!` 块合并，
输出为 `merged_ffi.rs`，同时更新 `build.rs` 和 `lib.rs` 以指向合并后的文件。

`merge` 需要先完成对应 feature 的 `init`。

---

## 输入与输出说明

### 输入（init）

| 参数 | 是否必填 | 说明 |
|------|---------|------|
| `-- <BUILD_CMD...>` | **必填** | 完整构建命令，放在 `--` 之后 |
| `--link <LINK>` | **必填** | 链接库名 |
| `--feature <FEATURE>` | 可选 | 特性名，默认 `default` |
| `--extra-clang-args` | 可选 | 额外 clang 参数 |
| `--clang` | 可选 | clang 路径，默认 `clang` |

### 输出目录

`init` 完成后：

```text
.cpp2rust/<feature>/
├── ast/
│   └── <header>.ast.json         # 各头文件的 clang AST（JSON 格式，用于调试）
├── meta/
│   ├── build_cmd.txt             # 本次捕获使用的构建命令
│   ├── captured_headers.list     # LD_PRELOAD hook 捕获到的所有头文件路径
│   ├── selected_headers.json     # 用户交互选择的头文件
│   ├── headers.json              # 最终用于 AST 解析与代码生成的头文件集合（含 link_name）
│   └── init-interface-report.md  # 本次 init 生成的接口报告
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        └── ffi_<header>.rs       # 各头文件对应的 Rust FFI 代码
```

`merge` 完成后，在上面基础上新增：

```text
.cpp2rust/<feature>/
├── meta/
│   └── merge-report.md
└── rust/
    └── src/
        └── merged_ffi.rs         # 合并后的单一 FFI 文件
```

---

## 示例

### examples/simple：自由函数示例

演示如何为包含自由函数（含重载）的 C++ 头文件生成 FFI。

```bash
# 从仓库根目录运行
cpp2rust-demo init --link mylib -- clang -x c++ -fsyntax-only examples/simple/mylib.hpp
cpp2rust-demo merge
cat .cpp2rust/default/rust/src/merged_ffi.rs
```

详见 [`examples/simple/README.md`](examples/simple/README.md)。

### examples/class：类方法示例

演示如何为包含类（含 `const`、`static` 方法）的 C++ 头文件生成 FFI。

```bash
cpp2rust-demo init --feature widget --link widget -- clang -x c++ -fsyntax-only examples/class/widget.hpp
cpp2rust-demo merge --feature widget
cat .cpp2rust/widget/rust/src/merged_ffi.rs
```

详见 [`examples/class/README.md`](examples/class/README.md)。

### examples/rapidjson：真实项目示例

演示如何在真实 C++ 开源项目 [Tencent/rapidjson](https://github.com/Tencent/rapidjson) 上运行
`cpp2rust-demo`，生成初始 FFI 脚手架。

```bash
git clone https://github.com/Tencent/rapidjson.git
cd rapidjson
cpp2rust-demo init \
  --feature rapidjson-doc \
  --link rapidjson \
  -- clang++ -x c++ -std=c++14 -Iinclude -fsyntax-only include/rapidjson/document.h
cpp2rust-demo merge --feature rapidjson-doc
```

详见 [`examples/rapidjson/README.md`](examples/rapidjson/README.md)。

---

## 当前限制 / 注意事项

- 仅支持 **Linux**（`LD_PRELOAD` hook 方案依赖 Linux）。
- hook 通过分析构建命令中 clang/gcc 的参数来发现头文件；若构建命令未触发实际编译调用，则无法捕获头文件。
- hook 只记录当前项目根目录下的头文件（请在目标工程根目录执行 `init`）。
- 对于模板类、虚继承、运算符重载等复杂 C++ 特性，自动生成的 FFI 可能需要手工调整。
- 生成结果应视为**初始脚手架**，而非可直接发布的最终绑定。

---

## 测试

运行全部测试：

```bash
cargo test
```

集成测试会自动检测外部工具（如 `clang`、`gcc`、`make`），缺失时打印跳过信息。

---

## 可选环境变量

| 变量 | 说明 |
|------|------|
| `CPP2RUST_CLANG` | 覆盖默认 `clang` 可执行文件路径 |
