# cpp2rust-demo

`cpp2rust-demo` 是一个 **从真实 C++ 构建过程提取接口并生成 Rust FFI 项目** 的命令行工具。  
核心目标是：尽量复用现有 C++ 工程的构建命令，通过 `LD_PRELOAD` 捕获编译单元，自动生成可由 `hicc` 使用的 Rust 侧绑定脚手架。

> 它是 **hicc FFI 脚手架生成器**，不是完整的 C++ → Rust 语义翻译器。

## 项目介绍（它解决什么问题）

传统 C++ -> Rust 绑定常见痛点是手工维护头文件列表、手写大量 FFI 声明。  
本项目通过两步流程减少手工工作：

1. `init`：执行真实构建命令并捕获 `.cpp2rust` 中间件，再生成平铺的 Rust 绑定模块（每个 C++ 翻译单元对应一个 `<stem>.rs`）。
2. `merge`：把平铺模块整合，将所有 hicc 必要内容直接写入 `lib.rs`（消费端入口）。

> 自动捕获对象是 C++ 编译单元（`.cc/.cpp/.cxx/.c++/.C`），头文件内容通过预处理展开进入中间件。

## 仓库目录说明（基于当前源码）

```text
.
├── src/
│   ├── main.rs       # CLI 定义与 init/merge 主流程
│   ├── capture.rs    # 构建 hook 并通过 LD_PRELOAD 执行真实构建命令
│   ├── layout.rs     # .cpp2rust/<feature> 目录布局与元数据读写
│   ├── ast.rs        # clang AST JSON 解析与声明抽取
│   ├── codegen.rs    # 生成 hicc 所需 Rust 代码（render_flat_module / build.rs / Cargo.toml）
│   ├── merge.rs      # 将 rust/src/<stem>.rs 平铺文件合并到 rust/src.2
│   ├── selector.rs   # 交互式/非交互式中间件选择
│   └── error.rs      # 统一 Result/Error 类型封装
├── hook/
│   ├── hook.c        # 编译拦截逻辑（识别编译器调用并输出 *2rust 中间件）
│   └── Makefile      # 生成 libhook.so
├── tests/
│   └── cli_tests.rs  # 端到端与生成结果校验
├── examples/
│   ├── simple/       # 自由函数示例
│   ├── class/        # 类与方法示例
│   ├── features/     # 特性粒度单示例（inline、默认参数、&&方法、va_list、全局变量、静态成员、实例字段）
│   ├── rapidjson/    # RapidJSON 场景（枚举/别名/模板类/接口/虚方法/继承/运算符shim/多TU）
│   ├── semi-auto/    # 半自动示例（dynamic_cast、placement new）
│   ├── conditional/  # 条件支持示例（无别名模板、无特化函数模板、链式别名）
│   └── guided/       # 引导支持示例（std::string、std::function、函数指针）
├── docs/
│   ├── design.md               # 设计与语义边界说明
│   ├── clang-ast.md            # AST 提取说明（节点类型、过滤规则、AliasRegistry）
│   ├── hicc-usage.md           # hicc 生成约定（关键约定与特性用法）
│   ├── cpp-features.md         # C++ 特性支持矩阵（含不支持原因）
│   ├── 特性支持全景图.md        # 完整特性全景表（汇总统计与示例链接）
│   ├── cpp2rust-demo与hicc能力全景.md  # hicc 全览 + cpp2rust-demo 工具层全览
│   ├── future-plan.md          # 工具层功能改进计划（已全部落地）
│   └── rapidjson-support.md    # RapidJSON 完整验证流程与特性矩阵
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
- `--dry-run`：仅执行构建捕获和 AST dump，不写入 `rust/src/`；接口报告输出到 stdout（适合快速验证 entry.cpp 配置）
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
  - 同时将 `operator_shims.hpp` 与 `init-interface-report.md` 复制到 `<output>/meta/`

### 3) 查看模板别名建议（`suggest-aliases`）

当 `init` 后接口报告中出现 `tool_conservative` 的模板跳过项时，可运行：

```bash
cpp2rust-demo suggest-aliases
cpp2rust-demo suggest-aliases --feature myfeature
```

工具会从已保存的 AST JSON 中提取所有跳过的模板特化，输出可直接复制到 entry.cpp 的 `using Alias = FullType<...>;` 建议列表，帮助解锁对应的 FFI 提取。

## 产物目录说明

`init` 后：

```text
.cpp2rust/<feature>/
├── cpp/                    # 预处理中间件（*.cpp2rust）与对应 .opts；同时含 *.cpp 符号链接
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
        ├── <stem>.rs         # 平铺模块（1:1 对应 <stem>.cpp），包含完整 hicc 绑定
        └── <stem>.meta.json  # 该翻译单元的元数据（函数/类/方法清单）
```

每个 `<stem>.rs` 是单个翻译单元的完整 hicc 脚手架，包含（按顺序）：
1. `hicc::cpp!` — 引用中间件源文件
2. C++ enum 定义（`#[repr(C)] pub enum`）
3. `pub type` 别名
4. `hicc::import_class!` — 每个 C++ 类一个块
5. `hicc::import_lib!` — 自由函数、静态方法、构造工厂函数

`merge` 后新增：

```text
.cpp2rust/<feature>/
├── meta/merge-report.md
└── rust/
    ├── src.1/     # init 原始平铺输出备份
    ├── src -> src.2
    └── src.2/
        ├── lib.rs          # 合并后的 FFI 入口（hicc::cpp! + import_class! + import_lib!）
        └── <stem>.rs       # 每个翻译单元的参考文件（不直接编译）
```

## C++ 与 Rust 代码关系

- **C++ 侧输入**：`hook/hook.c` 拦截编译器调用，生成 `*.cpp2rust` 中间件；`init` 同时在 capture 目录下创建同名符号链接（`entry.cpp → entry.cpp.cpp2rust`）。
- **中间表示**：`*.cpp2rust` + clang AST JSON（`ast.rs` 解析）。
- **Rust 侧输出**：`codegen.rs` 生成平铺的 `<stem>.rs`，包含 `hicc::cpp!`、`hicc::import_class!`、`hicc::import_lib!` 三段式绑定；每个 C++ 翻译单元对应一个 RS 文件（1:1 映射）。`hicc::cpp!` 中的 `#include` 引用原始源文件名（如 `"entry.cpp"`）而非 `.cpp2rust` 后缀名。
- **合并阶段**：`merge.rs` 扫描 `src.1/` 中的平铺 `*.rs` 文件（跳过 `lib.rs` 和 `common/`），将各文件中的 `hicc::cpp!`/`hicc::import_class!`/`hicc::import_lib!` 块汇聚，直接写入 `lib.rs`。不生成独立的 `merged_ffi.rs`——因为编译单元已经 1:1 平铺，`lib.rs` 即是消费端唯一入口。非业务元数据常量（`CPP_TYPES`、`CLASS_NAMES` 等）不写入合并输出，保持生成项目精简。
- `common/types.rs`：跨 group 聚合类型块（仅 enum 定义和类型别名写入 `lib.rs`；`CPP_TYPES` 等元数据常量**不**写入合并输出）
- `common/includes.rs`：路径元数据（**不**写入合并输出）

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
  - `examples/rapidjson/01-enum/README.md`（枚举绑定）
  - `examples/rapidjson/02-typedef-alias/README.md`（typedef/using 别名）
  - `examples/rapidjson/03-template-class/README.md`（模板特化类）
  - `examples/rapidjson/04-abstract-interface/README.md`（纯虚接口 + @make_proxy）
  - `examples/rapidjson/05-virtual-methods/README.md`（非纯虚方法）
  - `examples/rapidjson/06-inheritance/README.md`（public 继承）
  - `examples/rapidjson/07-operator-shim/README.md`（运算符重载 shim）
  - `examples/rapidjson/08-multi-tu/README.md`（多翻译单元 + merge）
