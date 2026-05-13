# cpp2rust-demo

`cpp2rust-demo` 是一个 **从真实 C++ 构建过程提取接口并生成 Rust FFI 项目** 的命令行工具。  
核心目标是：尽量复用现有 C++ 工程的构建命令，通过 `LD_PRELOAD` 捕获编译单元，自动生成可由 `hicc` 使用的 Rust 侧绑定脚手架。

> 它是 **hicc FFI 脚手架生成器**，不是完整的 C++ → Rust 语义翻译器。

## 项目介绍（它解决什么问题）

传统 C++ -> Rust 绑定常见痛点是手工维护头文件列表、手写大量 FFI 声明。  
本项目通过两步流程减少手工工作：

1. `init`：执行真实构建命令并捕获 `.cpp2rust` 中间件，再生成分组的 Rust 绑定模块。
2. `merge`：把分组模块整合为更易消费的 `merged_ffi.rs`。

> 自动捕获对象是 C++ 编译单元（`.cc/.cpp/.cxx/.c++/.C`），头文件内容通过预处理展开进入中间件。

## 仓库目录说明（基于当前源码）

```text
.
├── src/
│   ├── main.rs       # CLI 定义与 init/merge 主流程
│   ├── capture.rs    # 构建 hook 并通过 LD_PRELOAD 执行真实构建命令
│   ├── layout.rs     # .cpp2rust/<feature> 目录布局与元数据读写
│   ├── ast.rs        # clang AST JSON 解析与声明抽取
│   ├── codegen.rs    # 生成 hicc 所需 Rust 代码（include/free/class/method/types/common）
│   ├── merge.rs      # 将 rust/src/mod_<group> 合并到 rust/src.2
│   └── selector.rs   # 交互式/非交互式中间件选择
├── hook/
│   ├── hook.c        # 编译拦截逻辑（识别编译器调用并输出 *2rust 中间件）
│   └── Makefile      # 生成 libhook.so
├── tests/
│   └── cli_tests.rs  # 端到端与生成结果校验
├── examples/
│   ├── simple/       # 自由函数示例
│   └── class/        # 类与方法示例
├── docs/
│   ├── design.md     # 设计与语义边界说明
│   ├── clang-ast.md  # AST 提取说明
│   └── hicc-usage.md # hicc 生成约定
└── scripts/
    └── validate-rapidjson.sh # CI 对应本地复现实验脚本
```

## 构建方式

### 依赖条件

- Linux（依赖 `LD_PRELOAD`）
- Rust/Cargo（`Cargo.toml` 声明 `rust-version = 1.82`）
- `make` + `gcc`（用于构建 `hook/libhook.so`）
- `clang` 或兼容 clang 的工具链（用于 AST dump；`init --clang` 可指定）

### 构建与测试命令

```bash
cargo build
cargo test
```

## 运行方式

### 1) 生成分组绑定（`init`）

```bash
cpp2rust-demo init --link mylib -- make -j4
```

常用参数（以 `cpp2rust-demo init --help` 为准）：

- `--feature <name>`：输出目录分组名，默认 `default`
- `--link <libname>`：写入 `hicc::import_lib!` 的 `link_name`
- `--clang <bin>`：指定 clang 可执行文件（默认 `clang`，也可用 `CPP2RUST_CLANG` 环境变量）
- `--extra-clang-args "<args>"`：附加 AST 阶段 clang 参数（例如 `-std=c++17 -Iinclude`）
- `--no-link` / `--header-only`：header-only/no-link 模式；生成的 `build.rs` 不再输出目标库 `cargo::rustc-link-lib=<link_name>`
- `-- <BUILD_CMD...>`：真实构建命令（必填）

也可用单个翻译单元触发流程（如 header-only 库）：

```bash
cat > entry.cpp <<'CPP'
#include "mylib.hpp"
CPP
cpp2rust-demo init --link mylib -- clang++ -x c++ -std=c++17 -fsyntax-only -Iinclude entry.cpp
```

如果库本身没有可链接目标（例如 RapidJSON）：

```bash
cpp2rust-demo init --link rapidjson --no-link -- clang++ -x c++ -std=c++17 -fsyntax-only -Iinclude entry.cpp
```

### 2) 合并输出（`merge`）

```bash
cpp2rust-demo merge
cpp2rust-demo merge --feature myfeature
cpp2rust-demo merge --feature myfeature --output /tmp/merged-rust
```

常用参数（以 `cpp2rust-demo merge --help` 为准）：

- `--feature <name>`：要合并的 feature，默认 `default`
- `-o, --output <dir>`：可选；将 merge 后 `rust/` 导出到目标目录
  - 导出会跳过顶层 `src.1` / `src.2`
  - 导出后的 `src` 会是实体目录（跟随 `src -> src.2` 符号链接复制）
  - 目标目录必须为空目录（或不存在）

## 产物目录说明

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
            ├── class/mod.rs + cls_*.rs
            ├── method/mod.rs + mtd_*.rs
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

## C++ 与 Rust 代码关系

- **C++ 侧输入**：`hook/hook.c` 拦截编译器调用，生成 `*.cpp2rust` 中间件。
- **中间表示**：`*.cpp2rust` + clang AST JSON（`ast.rs` 解析）。
- **Rust 侧输出**：`codegen.rs` 生成 `hicc::cpp!`、`hicc::import_lib!`、`hicc::import_class!` 及语义清单模块。
- **合并阶段**：`merge.rs` 将 `mod_<group>` 的 include/types/free/class/method 与 common 整合为 `merged_ffi.rs`。

语义边界（v1）：

- `include/`：`hicc::cpp!` include 上下文
- `free/`：自由函数与静态方法（`import_lib!`）
- `method/`：实例方法绑定（`import_class!`）
- `class/`：类级语义结构（类名、关系、计数等）
- `types/`：类型清单、C++→Rust 映射与查询函数
- `common/*`：跨 group 共享 include/type 语义

跳过规则（记录在 `init-interface-report.md`）：

- destructor（`HiccLimitation`：hicc 不支持显式析构绑定）
- copy/move constructor（自动识别并跳过，避免生成无意义绑定）
- operator overload（生成 `operator_shims.hpp` starter 辅助用户手写 C++ shim）
- 无别名的模板声明（`ToolConservative`：添加 `typedef`/`using` 别名后可解锁）
- 函数模板（需有 concrete specialization 可见于 AST）
- 含不支持类型的参数/返回值（`unsupported_type`：如函数指针、variadic、`auto`/`decltype`）

虚函数处理：

- **非纯 virtual 方法**：直接提取为 `#[cpp(method = "...")]`
- **全纯虚类**：提取为 `#[interface]` trait，并在 `import_lib!` 中生成 `@make_proxy` 反向绑定
- **混合类**（含普通方法 + 纯虚方法）：普通方法正常提取；纯虚方法生成 companion interface

## 相关文档与示例

- 设计说明：`docs/design.md`
- AST 处理：`docs/clang-ast.md`
- hicc 用法：`docs/hicc-usage.md`
- C++ 特性支持矩阵：`docs/cpp-features.md`
- 后续功能计划：`docs/future-plan.md`
- RapidJSON 支持文档（含完整验证流程）：`docs/rapidjson-support.md`
- 示例：
  - `examples/README.md`（总览，含能力速查与不支持特性说明）
  - `examples/simple/README.md`（自由函数、命名空间、重载）
  - `examples/class/README.md`（类、方法、构造函数、virtual、继承）
  - `examples/rapidjson-01-enum/README.md`（枚举绑定）
  - `examples/rapidjson-02-typedef-alias/README.md`（typedef/using 别名）
  - `examples/rapidjson-03-template-class/README.md`（模板特化类）
  - `examples/rapidjson-04-abstract-interface/README.md`（纯虚接口 + @make_proxy）
  - `examples/rapidjson-05-virtual-methods/README.md`（非纯虚方法）
  - `examples/rapidjson-06-inheritance/README.md`（public 继承）
  - `examples/rapidjson-07-operator-shim/README.md`（运算符重载 shim）
  - `examples/rapidjson-08-multi-tu/README.md`（多翻译单元 + merge）
