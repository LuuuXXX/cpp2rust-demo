# cpp2rust-demo

`cpp2rust-demo` 是一个面向 C++ 项目的命令行工具，当前提供两步流程：

1. `init`：捕获 C++ 构建过程，生成 `.cpp2rust` 中间件并自动推导头文件信息，再生成按头文件拆分的 `hicc` FFI 脚手架。
2. `merge`：将 `init` 产出的各 `ffi_*.rs` 文件合并为单一的 `merged_ffi.rs`，并汇总接口报告。

## 当前能力范围

- ✅ 已实现：`init`、`merge`
- ❌ 未实现：`update`、`reinit`、`sync`

## 核心流程

```text
C++ 项目目录
   │
   ├─ cpp2rust-demo init --link <libname> -- <构建命令>
   │    ├─ 编译 hook/libhook.so
   │    ├─ 通过 LD_PRELOAD 注入构建过程，捕获并生成 .cpp2rust 中间件
   │    ├─ 交互式选择参与转换的 .cpp2rust 文件（非交互环境自动全选）
   │    ├─ 从选中的 .cpp2rust 中自动识别项目头文件（供 hicc 生成使用）
   │    ├─ 对识别到的头文件调用 clang -ast-dump=json 解析 AST
   │    ├─ 提取函数/类声明，生成 hicc FFI 脚手架（ffi_<header>.rs）
   │    └─ 生成 .cpp2rust/<feature>/rust 及 init-interface-report.md
   │
   └─ cpp2rust-demo merge [--feature <name>]
        ├─ 合并 rust/src/ffi_*.rs 为单一 merged_ffi.rs
        ├─ 更新 build.rs 和 lib.rs 引用 merged_ffi
        └─ 生成 merge-report.md
```

## 项目结构（关键文件）

- `src/main.rs`：CLI 入口（`init` / `merge` 子命令）
- `src/capture.rs`：hook 构建与带 `LD_PRELOAD` 环境变量的构建命令执行
- `src/layout.rs`：`.cpp2rust/<feature>/` 目录与元数据管理
- `src/selector.rs`：交互式中间件文件选择（`dialoguer`）
- `src/ast.rs`：clang AST JSON 解析与 IR 提取
- `src/codegen.rs`：hicc FFI 代码生成（`import_lib!` / `import_class!`）
- `src/merge.rs`：`merge` 阶段合并逻辑
- `hook/`：`libhook.so` 源码（`hook.c`）与 Makefile
- `tests/`：单元测试 + 集成测试
- `scripts/validate-rapidjson.sh`：对 Tencent/rapidjson 的端到端验证脚本（可本地复现 CI 结果）

## 环境要求

- Linux（依赖 `LD_PRELOAD` 和 `/proc/self/cmdline`）
- Rust / Cargo（`Cargo.toml` 要求 `rust-version = 1.82`）
- `clang`（用于 `-ast-dump=json` 解析，以及作为构建命令中的编译器）
- `gcc` + `make`（用于编译 `hook/libhook.so`）

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

## 使用方式

### 1) init：捕获构建并生成初始 Rust FFI 结构

在目标 C++ 项目根目录（或其子目录）执行：

```bash
cpp2rust-demo init --link <库名> -- <你的构建命令>
```

示例：

```bash
# 常规 C++ 编译命令（推荐）
cpp2rust-demo init --link mylib -- clang++ -std=c++17 -c src/mylib.cpp

# 通过 make 构建整个项目
cpp2rust-demo init --feature myfeature --link mylib -- make -j4

# 通过 cmake 构建（会自动捕获编译产物并生成 .cpp2rust 中间件）
cpp2rust-demo init --link mylib -- cmake --build build

# 如果构建系统需要 shell 语法
cpp2rust-demo init --link mylib -- sh -c "make -j4"
```

说明：

- `--link` 为必填项，对应生成代码中 `hicc::import_lib!` 的 `link_name`。
- `--feature` 默认值为 `default`；多个特性可以在同一项目下并存。
- `--` 之后的所有参数会原样作为构建命令传入。
- 捕获完成后，工具会扫描 `.cpp2rust/<feature>/cpp/**/*.cpp2rust` 作为中间件输入。
- 在非交互环境（CI / 管道 / 重定向）下会自动全选；在交互终端下会弹出多选菜单：

  ```
  ? Select files to include in this feature (space to toggle, enter to confirm) ›
  ✔ /path/to/.cpp2rust/default/cpp/src/mylib.cpp2rust
  ✔ /path/to/.cpp2rust/default/cpp/src/utils.cpp2rust
    /path/to/.cpp2rust/default/cpp/src/internal.cpp2rust
  ```

  按 `Space` 切换选中，按 `Enter` 确认。

### 2) merge：将按头文件的 FFI 文件合并

```bash
cpp2rust-demo merge
cpp2rust-demo merge --feature myfeature
```

`merge` 需要先完成对应 feature 的 `init`。执行后会将所有 `ffi_*.rs` 合并成单一的 `merged_ffi.rs`，同时更新 `build.rs` 和 `lib.rs` 的引用。

## 输入与输出说明

### 输入（init）

- 必填：`--link <库名>`、构建命令（`-- <BUILD_CMD...>`）
- 可选：`--feature <name>`、`--extra-clang-args <ARGS>`、`--clang <CLANG>`

### 输出目录

`init` 后（示意）：

```text
.cpp2rust/<feature>/
├── cpp/
│   └── <source>.cpp2rust        ← hook 生成的中间件（保留预处理行标记）
├── ast/
│   └── <header>.ast.json        ← raw clang AST JSON（调试用）
├── meta/
│   ├── build_cmd.txt            ← init 传入的构建命令
│   ├── selected_files.json      ← 用户交互选择的 .cpp2rust 文件
│   ├── headers.json             ← 最终用于 AST/codegen 的头文件集合 + link_name
│   └── init-interface-report.md ← 提取到的接口摘要
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        └── ffi_<header>.rs      ← 每个输入头文件对应一个 FFI 文件
```

`merge` 后（在上面基础上新增）：

```text
.cpp2rust/<feature>/
├── meta/
│   └── merge-report.md
└── rust/
    └── src/
        └── merged_ffi.rs        ← 合并后的单一 FFI 文件
```

## 开发与测试

运行全部测试：

```bash
cargo test
```

集成测试会自动调用主机上的 `clang` 和 `make`；若工具不存在则测试会自动跳过并打印提示。

可在本地执行 rapidjson 验证脚本：

```bash
./scripts/validate-rapidjson.sh
```

## 可选环境变量

- `CPP2RUST_CLANG`：覆盖默认 `clang` 可执行文件名（`--clang` 选项的等价环境变量）
- `CPP2RUST_CC`：hook 识别的编译器名称（默认自动匹配 `gcc/g++/clang/clang++/cc/c++` 及带版本后缀的变体）
- `CPP2RUST_DEBUG`：设为非空时输出 hook 调试日志到 stderr

## 注意事项

- 目前仅支持 Linux（依赖 `LD_PRELOAD` 和 `/proc/self/cmdline`）。
- 推荐提供真实编译命令（例如 `clang++ -c xxx.cpp` / `make` / `cmake --build`），这样 hook 才能生成 `.cpp2rust` 中间件并自动推导头文件。
- hook 只处理当前项目根目录（即存在 `.cpp2rust/` 的目录）下的源码/头文件；请在目标工程根目录执行 `init`。
- 对于 header-only 场景，仍兼容直接编译头文件（如 `clang++ -x c++ -fsyntax-only <header>`），此时会回退到旧的头文件捕获路径。
- `merge` 会覆盖 `rust/src/lib.rs` 和 `rust/build.rs`，使其只引用 `merged_ffi.rs`；如需保留 `init` 的原始结构，请先备份。
