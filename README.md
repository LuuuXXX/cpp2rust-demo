# cpp2rust-demo

`cpp2rust-demo` 是一个把 C++ 头文件接口转换为 Rust `hicc` FFI 脚手架的演示工具。

本仓库当前实现**优先采用 `LD_PRELOAD` hook 方式**（参考 `LuuuXXX/c2rust-demo` 的 `hook/hook.c` 思路）来捕获头文件相关行为，再进行 AST 解析与代码生成，方便后续持续调整 hook 逻辑。

---

## 项目用途

- 从 C++ 头文件提取函数/类声明。
- 生成可用于 `hicc` 的 Rust FFI 代码（`import_lib!` / `import_class!`）。
- 支持 `merge` 将多个 `ffi_*.rs` 合并成单一 `merged_ffi.rs`。

---

## 依赖环境

- Linux（`LD_PRELOAD` 方案依赖 Linux）
- Rust（>= 1.82）
- `clang`
- `gcc` + `make`（用于编译 `hook/libhook.so`）

---

## 构建步骤

```bash
git clone https://github.com/LuuuXXX/cpp2rust-demo.git
cd cpp2rust-demo
cargo build
```

或直接运行：

```bash
cargo run -- --help
```

---

## `LD_PRELOAD` 使用方式

`init` 执行流程：

1. 构建 `hook/libhook.so`。
2. 使用 `LD_PRELOAD=hook/libhook.so` 运行捕获阶段。
3. hook 将捕获到的头文件记录到：
   - `.cpp2rust/<feature>/meta/captured_headers.list`
4. 以捕获结果为主进行 clang AST 解析与 Rust FFI 生成。

默认情况下，工具会对传入 header 逐个执行一次 `clang -fsyntax-only`（在 preload 环境下）来触发捕获。

如果你希望和真实工程构建对齐，可以使用 `--capture-cmd` 指定构建命令。

---

## 当前限制与默认回退行为

- 当前 `LD_PRELOAD` hook 方案主要面向 Linux。
- `--capture-cmd` 通过 `sh -c "<CMD>"` 执行，复杂命令请按 shell 语法正确引用和转义。
- 如果 hook 未捕获到头文件，`init` 会回退到命令行传入的 `<HEADER>...` 继续生成流程。

---

## 运行示例

### 示例 1：默认捕获模式（推荐起步）

```bash
cpp2rust-demo init --link mylib include/mylib.hpp
```

### 示例 2：使用真实构建命令触发 hook 捕获

```bash
cpp2rust-demo init \
  --feature myfeature \
  --link mylib \
  --capture-cmd "make -j4" \
  include/mylib.hpp include/types.hpp
```

### 合并输出

```bash
cpp2rust-demo merge --feature myfeature
```

---

## 生成目录

```text
.cpp2rust/<feature>/
├── ast/                          # clang AST JSON
├── meta/
│   ├── headers.json
│   ├── captured_headers.list     # LD_PRELOAD hook 捕获结果
│   ├── init-interface-report.md
│   └── merge-report.md
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── ffi_<header>.rs
        └── merged_ffi.rs
```

---

## 与原实现相比的差异

- 之前：主要是“直接代码内流程”处理 header。
- 现在：`init` 以 **LD_PRELOAD hook 捕获为主路径**，再进行后续 AST + 代码生成。
- 好处：
  - hook 行为集中在 `hook/hook.c`，后续调整捕获策略更直观。
  - 可通过 `--capture-cmd` 对接真实构建命令，便于持续演进。

---

## 命令说明

### init

```text
cpp2rust-demo init [OPTIONS] <HEADER>...

Options:
  --feature <FEATURE>              特性名（默认 default）
  --link <LINK>                    链接库名（必填）
  --extra-clang-args <ARGS>        传给 clang 的额外参数
  --clang <CLANG>                  clang 可执行文件（默认 clang）
  --capture-cmd <CMD>              可选：用于 preload 捕获的构建命令
```

### merge

```text
cpp2rust-demo merge [OPTIONS]

Options:
  --feature <FEATURE>              特性名（默认 default）
```

---

## 测试

```bash
cargo test
```
