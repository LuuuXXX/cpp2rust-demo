# cpp2rust-demo

`cpp2rust-demo` 是一个把 C++ 项目构建中提取的接口转换为 Rust `hicc` FFI 脚手架的演示工具。

本仓库当前实现与 `LuuuXXX/c2rust-demo` 对齐：`init` 先通过真实构建命令 + `LD_PRELOAD` hook 捕获输入，再经过**交互式头文件选择**，最后进行 AST 解析与代码生成；`merge` 负责后处理合并。

---

## 项目用途

- 从真实构建中捕获到的 C++ 头文件提取函数/类声明。
- 交互式选择需要生成绑定的头文件（非交互环境自动全选）。
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
4. **交互式选择**参与生成的头文件（非交互环境/CI 自动全选）。
5. 将选择结果保存到 `.cpp2rust/<feature>/meta/selected_headers.json`。
6. 以选择结果为主进行 clang AST 解析与 Rust FFI 生成。

`init` 必须使用 `--` 传入完整构建命令（如 `make` / `cmake --build` / 自定义脚本）。

---

## 头文件选择（与 c2rust-demo 对齐）

捕获完成后，工具会展示一个多选菜单（与 `c2rust-demo` 中选择 `.c2rust` 文件的行为对齐）：

```
? Select headers to include in this feature (space to toggle, enter to confirm) ›
✔ /path/to/mylib.hpp
✔ /path/to/utils.hpp
  /path/to/internal.hpp
```

- 按 `Space` 切换选中状态，按 `Enter` 确认。
- 默认全部选中。
- 如果 stdin 不是终端（CI / 管道 / 测试环境），自动全选所有头文件。

选择结果保存在：

```
.cpp2rust/<feature>/meta/selected_headers.json
```

---

## 当前限制与默认回退行为

- 当前 `LD_PRELOAD` hook 方案主要面向 Linux。
- hook 目前依赖编译进程参数中的头文件路径（clang/gcc）来发现输入；若构建命令未触发相关编译调用，则无法生成绑定。
- hook 只记录当前项目根目录下的头文件（`init` 的执行目录/其已存在 `.cpp2rust` 的上级目录）；请在目标工程根目录执行 `init`。
- 如果构建系统需要 shell 语法，请直接在 `--` 后传 `sh -c ...`（例如：`cpp2rust-demo init --link mylib -- sh -c "make -j4"`）。

---

## 运行示例

### 示例 1：推荐流程（与 c2rust-demo 一致）

```bash
cpp2rust-demo init --link mylib -- make -j4
```

### 示例 2：使用 cmake 构建命令

```bash
cpp2rust-demo init \
  --feature myfeature \
  --link mylib \
  -- cmake --build build
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
│   ├── build_cmd.txt             # init 捕获阶段使用的构建命令
│   ├── captured_headers.list     # LD_PRELOAD hook 捕获到的所有头文件
│   ├── selected_headers.json     # 用户交互选择的头文件（与 c2rust-demo selected_files.json 对齐）
│   ├── headers.json              # 最终用于 AST/codegen 的 header 集合 + link_name
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

## 与 c2rust-demo 对齐情况

| 能力                    | c2rust-demo          | cpp2rust-demo                   |
|------------------------|----------------------|---------------------------------|
| 构建命令捕获             | `-- <BUILD_CMD>`     | `-- <BUILD_CMD>`（相同）         |
| LD_PRELOAD hook        | ✅                   | ✅                               |
| 交互式文件选择           | ✅ `.c2rust` 文件    | ✅ 头文件（`captured_headers`）  |
| 非交互环境自动全选        | ✅                   | ✅                               |
| 选择结果持久化           | `selected_files.json`| `selected_headers.json`         |
| 生成 Rust 绑定          | `bindgen`            | `hicc`（架构差异）               |
| `merge` 合并            | ✅                   | ✅                               |

---

## 与原实现相比的差异

- 之前：header-first（`init ... <HEADER>...`）。
- 现在：build-command-first（`init ... -- <BUILD_CMD...>`），与 `c2rust-demo` 心智模型一致。
- 新增：交互式头文件选择步骤，与 `c2rust-demo` 的文件选择流程对齐。
- 好处：
  - hook 行为集中在 `hook/hook.c`，后续调整捕获策略更直观。
  - 直接复用真实构建命令，减少"测试命令"和"真实构建"不一致的问题。
  - 用户可精确控制哪些头文件参与绑定生成。

---

## 命令说明

### init

```text
cpp2rust-demo init [OPTIONS] -- <BUILD_CMD...>

Options:
  --feature <FEATURE>              特性名（默认 default）
  --link <LINK>                    链接库名（必填）
  --extra-clang-args <ARGS>        传给 clang 的额外参数
  --clang <CLANG>                  clang 可执行文件（默认 clang）
```

`init` 执行成功后，会在 `meta/` 目录下生成 `selected_headers.json`，记录本次选择的头文件。

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
