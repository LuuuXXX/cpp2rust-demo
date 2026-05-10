# cpp2rust-demo

`cpp2rust-demo` 是一个把 C++ 项目构建中提取的接口转换为 Rust `hicc` FFI 脚手架的演示工具。

本仓库与 `LuuuXXX/c2rust-demo` 的工作流对齐：`init` 先通过真实构建命令 + `LD_PRELOAD` hook 捕获输入头文件，再通过**交互式选择**确认要处理的 header 子集，最后进行 AST 解析与 hicc FFI 代码生成。`merge` 负责后处理合并。

---

## 项目用途

- 从真实构建中捕获到的 C++ 头文件提取函数/类声明。
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

## `init` 完整流程

`init` 执行步骤：

1. **构建 hook**：编译 `hook/libhook.so`。
2. **运行捕获**：使用 `LD_PRELOAD=hook/libhook.so` 执行传入的构建命令，hook 将捕获到的头文件路径写入
   `.cpp2rust/<feature>/meta/captured_headers.list`。
3. **交互式选择**：展示捕获到的 header 列表，由用户多选（空格切换，回车确认）。
   - 选中的 header 写入 `.cpp2rust/<feature>/meta/selected_headers.json`。
   - **非交互环境**（CI / pipe / 脚本）：自动全选，不阻塞。
   - 若用户一个都不选，则打印提示并跳过后续 Rust 生成。
4. **AST 解析 + hicc FFI 生成**：仅对被选中的 header 运行 clang AST 解析并生成 `ffi_*.rs`。

`init` 必须使用 `--` 传入完整构建命令（如 `make` / `cmake --build` / 自定义脚本）。

---

## 当前限制与默认回退行为

- 当前 `LD_PRELOAD` hook 方案主要面向 Linux。
- hook 目前依赖编译进程参数中的头文件路径（clang/gcc）来发现输入；若构建命令未触发相关编译调用，则无法生成绑定。
- hook 只记录当前项目根目录下的头文件（`init` 的执行目录/其已存在 `.cpp2rust` 的上级目录）；请在目标工程根目录执行 `init`。
- 如果构建系统需要 shell 语法，请直接在 `--` 后传 `sh -c ...`（例如：`cpp2rust-demo init --link mylib -- sh -c "make -j4"`）。

---

## 运行示例

### 示例 1：单头文件

```bash
cpp2rust-demo init --link mylib -- clang -x c++ -fsyntax-only include/mylib.hpp
```

### 示例 2：使用 make 构建命令

```bash
cpp2rust-demo init --link mylib -- make -j4
```

### 示例 3：使用 cmake 构建命令

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
│   ├── captured_headers.list     # LD_PRELOAD hook 捕获的全部头文件
│   ├── selected_headers.json     # 交互式选择后被选中的头文件
│   ├── headers.json              # 与 selected_headers.json 相同集合 + link_name（供 merge 使用）
│   ├── init-interface-report.md
│   └── merge-report.md
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── ffi_<header>.rs       # 每个被选中的 header 对应一个
        └── merged_ffi.rs         # merge 后的合并文件
```

---

## 与 c2rust-demo 的对齐情况

| 步骤 | c2rust-demo | cpp2rust-demo |
|------|-------------|---------------|
| build hook | ✅ | ✅ |
| LD_PRELOAD 捕获 | ✅ | ✅ |
| 交互式选择 | ✅（.c2rust 文件） | ✅（.hpp 头文件） |
| 选择结果持久化 | `selected_files.json` | `selected_headers.json` |
| 非交互自动全选 | ✅ | ✅ |
| 空选择跳过生成 | ✅ | ✅ |
| Rust 侧架构 | split（per-symbol） | **hicc**（per-header ffi_*.rs） |

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
