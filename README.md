# cpp2rust-demo

`cpp2rust-demo` 是一个面向 C++ 项目的命令行工具，使用 `LD_PRELOAD` 捕获真实编译过程中的 **C++ 编译单元（translation units）**，生成 `.cpp2rust` 预处理中间件，并基于 `hicc` 生成 Rust FFI 脚手架。

## 核心流程（init）

```text
C++ 项目目录
   │
   ├─ cpp2rust-demo init --link <libname> -- <构建命令>
   │    ├─ 编译 hook/libhook.so
   │    ├─ 通过 LD_PRELOAD 注入构建过程，仅捕获 C++ 编译单元并生成 `.cpp2rust` 中间件（例如 `a.cpp -> a.cpp.cpp2rust`）
   │    ├─ 扫描 .cpp2rust/<feature>/cpp/**/*.cpp2rust
   │    ├─ 交互式选择参与转换的中间件文件（非交互环境自动全选）
   │    ├─ 对每个选中文件执行 clang -ast-dump=json
   │    ├─ 识别 hicc 所需信息（函数名、参数类型、类/命名空间等）
   │    └─ 生成 hicc Rust 项目与 init-interface-report.md
   │
   └─ cpp2rust-demo merge [--feature <name>]
         ├─ 按 mod_<group> 汇总 include/types/free/class/method/global（global 当前为可选目录）
        ├─ 产出 rust/src.2/mod_<group>.rs + rust/src.2/lib.rs + rust/src.2/merged_ffi.rs
        └─ 切换 rust/src 为指向 src.2 的符号链接（rust/src.1 备份 init 原始输出）
```

## 环境要求

- Linux（依赖 `LD_PRELOAD`）
- Rust/Cargo
- `gcc`/`g++`/`clang`/`clang++`
- `make`（用于构建 `hook/libhook.so`）

## 构建与测试

```bash
cargo build
cargo test
```

## 使用方式

### 1) init

```bash
cpp2rust-demo init --link mylib -- make -j4
```

也可直接用单个翻译单元触发完整流程（推荐用于 header-only 库）：

```bash
cat > entry.cpp <<'CPP'
#include "mylib.hpp"
CPP
cpp2rust-demo init --link mylib -- clang++ -x c++ -std=c++17 -fsyntax-only -Iinclude entry.cpp
```

说明：

- `--feature` 默认为 `default`
- `--` 后为真实构建命令，工具不再要求用户单独手工输入头文件列表
- 自动捕获仅面向 C++ 编译单元（`.cc/.cpp/.cxx/.c++/.C`），不会直接捕获 `.h/.hpp/.hh/.hxx`
- 头文件信息通过编译单元预处理展开进入 `*.cpp2rust`，后续 AST/`hicc` 提取均基于这些预处理中间件

### 2) merge

```bash
cpp2rust-demo merge
cpp2rust-demo merge --feature myfeature
```

## 输出目录

`init` 后：

```text
.cpp2rust/<feature>/
├── cpp/                    # 预处理中间件（*.cpp2rust）与对应 .opts
├── ast/                    # 每个选中文件的 clang AST JSON
├── meta/
│   ├── build_cmd.txt
│   ├── selected_files.json
│   ├── headers.json        # link_name + 选中的中间件文件
│   └── init-interface-report.md
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── common/
        │   ├── mod.rs
        │   ├── includes.rs
        │   └── types.rs
        └── mod_<group>/
            ├── mod.rs
            ├── include/mod.rs
            ├── types/mod.rs
            ├── free/mod.rs + fn_*.rs
            ├── class/mod.rs + cls_*.rs（类级 inventory/元信息）
            ├── method/mod.rs + mtd_*.rs (有实例方法时)
            ├── global/ (可选，当前默认不生成)
            └── meta.json
```

`merge` 后新增：

```text
.cpp2rust/<feature>/
├── meta/merge-report.md
└── rust/
    ├── src.1/     # init 原始拆分输出备份
    ├── src -> src.2
    └── src.2/
        ├── lib.rs
        ├── mod_<group>.rs
        └── merged_ffi.rs
```

## hicc 集成约定

- 生成代码统一使用 `hicc::cpp!`、`hicc::import_class!`、`hicc::import_lib!`
- `build.rs` 使用 `hicc_build::Build` 作为唯一 Rust 侧框架搭建方式
- `build.rs` 始终引用 `src/...`（活跃视图）；merge 后通过 `src -> src.2` symlink 指向最新产物
- include 路径来自选中的 `*.cpp2rust` 文件所在目录
- 第一版语义分类以 middleware 路径分组：`src/foo/bar.cpp.cpp2rust -> mod_src_foo_bar`
- 当前能力边界（v1）：
  - `include/`：`hicc::cpp!` include 上下文
  - `free/`：自由函数 + 静态方法（`hicc::import_lib!`）
  - `method/`：实例方法绑定（当前唯一承接 `hicc::import_class!` 的目录）
  - `class/`：类级 inventory/元信息（例如 class 名称清单），不是方法绑定层
  - `types/`：当前定位为 type inventory（类型清单），后续可演进为更完整类型绑定层
  - `common/*`：当前定位为 shared inventory/shared context（共享清单/上下文），不是共享绑定层
  - `global/`：暂未做独立 AST 产物，当前默认不生成该目录

- merge 语义（当前）：
  - `include/`、`method/`、`free/`、`types/` 会参与 `src.2/*` 产物拼装
  - `method/` 负责输出 `import_class!` 绑定块，`free/` 负责输出 `import_lib!` 绑定块
  - `class/` 与 `common/*` 主要用于 init 视图中的语义清单/元信息，不构成 merged_ffi 的主绑定面

## CI 与脚本

- CI: `.github/workflows/validate-rapidjson.yml`
- 本地复现脚本: `scripts/validate-rapidjson.sh`

二者均覆盖新的“编译→捕获→中间件→选择→转换→hicc 项目生成”流程。
