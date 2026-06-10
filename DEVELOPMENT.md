# cpp2rust-demo 开发文档

> 本文档记录项目的设计目标、架构、当前进度和后续计划，供所有开发者参考。

---

## 1. 架构概览

> 工具介绍与功能特性见 [README.md](../README.md), 深度技术方案见 [docs/INTRODUCTION.md](../docs/INTRODUCTION.md)。

```
cpp2rust-demo (bin)
│
├── hook/                        # LD_PRELOAD 拦截器源码（内嵌进 binary）
│   ├── hook.cpp                 # 拦截 g++/clang++ 调用，产出 .cpp2rust 预处理文件
│   └── Makefile
│
└── src/
    ├── main.rs                  # CLI 入口：init / merge
    ├── capture.rs               # hook 编译 + LD_PRELOAD 注入执行
    │                            #   - hook.cpp/Makefile 通过 include_str! 内嵌进 binary
    │                            #   - ensure_hook_data_dir() 首次运行时解压到用户数据目录
    │                            #     Linux: ~/.local/share/cpp2rust-demo/hook/
    │                            #     macOS: ~/Library/Application Support/cpp2rust-demo/hook/
    │                            #   - build_hook() mtime 快路径：.so 比 hook.cpp 新则跳过 make
    ├── layout/                  # 目录布局（.cpp2rust/<feature>/c|meta|rust）
    │   ├── mod.rs               # 公开接口及辅助工具函数
    │   ├── types.rs             # 纯数据结构（ApiManifest、FeatureLayout 等）
    │   └── io.rs                # FeatureLayout 的 I/O 方法与报告生成
    ├── selector.rs              # 交互式文件选择
    ├── ffi_model.rs             # FFI 中间表示（FfiSpec / ClassSpec / FnBinding 等）
    ├── error.rs                 # 统一错误类型
    ├── ast_parser/              # Phase 2：C++ AST 解析（clang crate → CppAst）
    │   ├── mod.rs               # 公共入口 parse_preprocessed()
    │   ├── collector.rs         # 类/函数/枚举收集逻辑
    │   └── range_scanner.rs     # 文件字节范围扫描（区分用户代码 vs 头文件）
    ├── extractor/               # Phase 3：CppAst → FfiSpec IR 提取
    │   ├── mod.rs               # 公共入口 extract()、命名空间模式检测、类/函数/枚举提取
    │   ├── type_mapper.rs       # C++ 类型 → Rust 类型映射
    │   ├── class_spec.rs        # ClassSpec 构建：方法/构造器/析构器/关联函数归属
    │   ├── lib_spec.rs          # import_lib! 块构建：全局函数绑定与前向声明生成
    │   └── cpp_block.rs         # hicc::cpp! 块的 #include 行提取与 C++ shim 内联片段构造
    ├── postprocessor/           # Phase 4：特殊情况后处理
    │   ├── mod.rs
    │   ├── operator_handler.rs  # 运算符重载 [OP]
    │   └── diamond_handler.rs   # 菱形继承路径检测与命名 shim 生成
    ├── merger/                  # Phase 6：merge 命令核心逻辑
    │   ├── mod.rs               # merge_units()：多 unit .rs 文件合并为 MergedSpec，去重 + 冲突检测
    │   └── block_parser.rs      # parse_unit_rs()：解析单个 .rs 文件中的 hicc 块
    ├── metrics.rs               # merge 命令辅助模块：源码行数统计、cpp2rust-todo 标签解析（非独立 phase）
    └── generator/               # Phase 5：FfiSpec → Rust 代码
        ├── mod.rs
        ├── hicc_codegen.rs      # hicc 三段式代码生成
        └── project_generator.rs # Cargo.toml / lib.rs / build.rs 生成
```

### 2.1 六阶段处理流程

```
Phase 1              Phase 2                 Phase 3              Phase 4              Phase 5              Phase 6
编译拦截 (hook.cpp)   AST 解析                IR 提取              后处理              代码生成              合并整理
LD_PRELOAD           ast_parser/             extractor/           postprocessor/      generator/           merger/
→ g++ -E -C          clang crate 解析        CppAst → FfiSpec     FfiSpec 特殊情况    FfiSpec → hicc       merge_in_place()
→ .cpp2rust   →      .cpp2rust → CppAst →   FfiSpec IR       →   处理（菱形继承）→   三段式 Rust 代码  →  src/ 整理 + 报告
```

### 2.2 输出目录结构

```
.cpp2rust/<feature>/
├── c/                   # 预处理文件（.cpp2rust 后缀）
├── meta/                # build_cmd.txt、selected_files.json
│   ├── init-report.md   # init 阶段摘要报告（由 init 生成）
│   ├── merge-report.md  # merge 阶段摘要报告（由 merge 生成）
│   └── api-manifest.md  # C++ → Rust API 对账清单（由 merge 生成，Markdown 格式）
└── rust/                # 生成的 Rust 项目
    ├── src.1/           # merge 阶段备份的 init 原始输出（首次 merge 时由 src/ rename 而来）
    └── src/             # merge 输出，真实目录（与 C++ 项目目录结构一致）
        ├── lib.rs
        ├── <unit>.rs         # 扁平文件（C++ 项目根目录下的编译单元）
        └── <subdir>/<unit>.rs# 带子目录（C++ 源文件位于 src/ 等子目录时）
```

`api-manifest.md` 是 merge 阶段生成的 C++ → Rust API 对账清单（Markdown 格式），包含：
- 类绑定：每个类的属性、方法列表（C++ 签名、Rust 签名、状态）
- 独立函数列表（C++ 签名、Rust 签名、状态）
- 状态：✓ 表示绑定正常，⚠ 降级 表示含 `cpp2rust-todo` 注释，需人工完善

> 三段式代码格式说明见 [README.md — 生成代码格式](../README.md#生成代码格式三段式)。

---

## 3. 48 个示例的支持矩阵

> 图例：✅ 完全自动生成可编译代码　⚠️ 降级生成 + 内联 TODO　❌ 尚未实现

| 类别 | 编号范围 | 状态 |
|------|---------|------|
| 基础类型与函数 | 001–005 | ✅ |
| 类与对象 | 006–012 | ✅ |
| 面向对象特性 | 013–017 | ✅ |
| 菱形继承 | 018 | ✅ (`diamond_handler.rs` 生成命名 shim) |
| **运算符重载** | 019 | ⚠️ [OP] L1/L2 通过（降级为命名 shim） |
| 友元/explicit/mutable/RTTI | 020–023 | ✅ |
| 模板实例化 | 024–028 | ✅ |
| 智能指针与内存 | 029–033 | ✅ |
| STL 容器 | 034–038 | ✅ |
| **有状态 Lambda** | 039 | ✅ [LM] L1/L2 通过（函数指针 + class wrapper 方案） |
| **std::function** | 040 | ✅ [LM] L1/L2 通过（class wrapper 方案） |
| functional_bind / 异常 | 041–042 | ✅ |
| 高级特性 | 043–048 | ✅ |

### 3.1 降级特性说明（6 项）

| TAG | 编号 | C++ 特性 | 不能完全自动的原因 | 自动降级策略 |
|-----|------|---------|-----------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号 | 生成命名 shim（`{class}_add` 等）+ 内联 TODO |
| `[VA]` | 028 | 可变参数模板 | `...Args` 编译期展开，FFI 无法表达 | 生成 wrapper 类 + 按数量展开版本 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态→函数指针；有状态→class wrapper |
| `[LM]` | 040 | std::function | 类型擦除容器，签名可推断但捕获不透明 | class wrapper + opaque pointer |
| `[CV]` | 005 | C 可变参数函数 | `...` 参数在运行时通过 `va_list` 访问，FFI 要求静态类型列表 | 含 `...` 函数整体跳过；头文件中固定参数 wrapper 直接绑定 |
| `[FP]` | 039, 040 | 函数指针参数 | C++ 成员函数指针无法映射为 Rust FFI 类型 | C 函数指针自动映射为 `unsafe extern "C" fn(...)`，加 `[FP]` 注释；C++ 成员函数指针整体跳过 |
| `[VM]` | 012 | volatile 成员函数 | `volatile this` 方法指针在 Rust 无对应语义，导致 hicc 类型不匹配 | `is_volatile` 方法从 `import_class!` 移除；对应 `extern "C"` shim 仍进入 `import_lib!` |

---

## 4. 测试体系

测试分五层，位于 `tests/` 目录：

| 层 | 文件 | 验证内容 | 运行命令 |
|----|------|---------|---------|
| L1 | `l1_golden_tests.rs` | 工具生成的 FFI 脚手架与 `rust_hicc/src/main.rs` 中对应段落一致 | `cargo test --test l1_golden_tests --features full-test -- --test-threads=1` |
| L2 | `l2_compile_tests.rs` | 仓库中现有的 `rust_hicc/` 能通过 `cargo build` | `cargo test --test l2_compile_tests` |
| L3 | `l3_run_tests.rs` | `cargo run` 输出与 README 中"运行结果"一致 | `cargo test --test l3_run_tests --features full-test -- --test-threads=1` |
| L4 | `rapidjson_e2e_test.rs` | 对 rapidjson 开源项目执行完整 init + merge 转换，验证 hicc 三段式格式；merge 阶段同时执行 `cargo check` | `cargo test --test rapidjson_e2e_test -- --test-threads=1` |
| L4 | `tinyxml2_e2e_test.rs` | tinyxml2 单文件项目 init 阶段 + merge 阶段（`cargo check`）验证 | `cargo test --test tinyxml2_e2e_test -- --test-threads=1` |
| L4 | `pugixml_e2e_test.rs` | pugixml 单文件项目 init 阶段 + merge 阶段（`cargo check`）验证 | `cargo test --test pugixml_e2e_test -- --test-threads=1` |
| L4 | `sqlite3_e2e_test.rs` | sqlite3 extern-C 接口 init 阶段 + merge 阶段（`cargo check`）验证 | `cargo test --test sqlite3_e2e_test -- --test-threads=1` |
| L4 | `nlohmann_json_e2e_test.rs` | nlohmann/json 超大头文件 init 阶段 + merge 阶段（`cargo check`）验证 | `cargo test --test nlohmann_json_e2e_test -- --test-threads=1` |
| L4 | `fmtlib_e2e_test.rs` | fmtlib 多文件项目 init 阶段 + merge 阶段（`cargo check`）验证 | `cargo test --test fmtlib_e2e_test -- --test-threads=1` |
| L4 | `multi_feature_e2e_test.rs` | 多 feature 合并 & output-dir 导出完整流程验证 | `cargo test --test multi_feature_e2e_test -- --test-threads=1` |
| L5 | `l5_nm_symbol_tests.rs` | 用 `nm` 双向验证 C++ 导出符号均已链接进 Rust FFI 二进制 | `cargo test --test l5_nm_symbol_tests -- --ignored` |

**L1/L3 测试控制**：L1 和 L3 测试通过 `full-test` feature flag 控制。不加 `--features full-test` 时，这两类测试会被自动跳过（ignored）；加上后则正常运行。这比 `--include-ignored` 语义更清晰，也不会误触其他被标记为 ignored 的测试。

**L1 核心逻辑**：从 `rust_hicc/src/main.rs` 提取 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三种块作为黄金片段，与工具生成的 `lib.rs` 对应块比对，忽略 `fn main()` 和注释差异。

**L4 merge 阶段**：各 E2E 测试均包含 `<lib>_merge_phase` 测试函数，执行 init → merge → `cargo check` 完整链路，确保生成的 Rust 项目在无 `build.rs` 情形下可通过类型检查。依赖外部子模块或系统头文件时自动跳过（graceful skip）。

**重要**：L1 测试须单线程运行（`--test-threads=1`），多线程下 clang 全局状态存在竞争。

### 分层快速运行命令

```sh
# L2 编译测试（无需 libclang，最快）
cargo test

# L1/L3 golden + 运行测试（需要 libclang 和系统 g++/clang++）
cargo test --features full-test -- --test-threads=1

# L5 nm 符号测试（需要 nm 工具）
cargo test -- --ignored
```

---

## 5. 当前进度（截至 2026-06-08，最新更新）

### 5.1 Phase 完成状态

| Phase | 内容 | 状态 |
|-------|------|------|
| **Phase T** | 测试基础设施（L1–L5 框架） | ✅ 完成 |
| **Phase 0** | Hook 机制（`hook.cpp` + `capture.rs`） | ✅ 完成 |
| **Phase 1** | AST 解析（`ast_parser.rs`，clang crate） | ✅ 完成 |
| **Phase 2** | 基础提取器（class/function/enum extractor） | ✅ 完成 |
| **Phase 3** | 模板实例化追踪（集成于 `ast_parser.rs`，`template_class_ranges` 字段） | ✅ 完成 |
| **Phase 4** | 后处理器（operator/friend/lambda handler，含菱形继承） | ✅ 完成 |
| **Phase 5** | hicc 代码生成器（`hicc_codegen.rs`） | ✅ 完成 |
| **Phase 6** | `merge` 命令 + 增量/多 feature 支持 | ✅ 完成 |
| **Phase 7** | CI 环境修复（Ubuntu 24.04 依赖对齐） | ✅ 完成 |
| **Phase 8** | 代码质量清理（`cargo clippy` 零警告） | ✅ 完成 |
| **Phase 9** | L3 运行测试修复（`compare_run_output`、030 SIGSEGV、14 个 README 对齐） | ✅ 完成 |
| **Phase 10** | 路径生成修复（`derive_unit_path` 消除双重 `src/` 前缀） | ✅ 完成 |
| **Phase 11** | Codegen 精确度修复（Dtor/Ctor 归属、接口类检测、`namespace_class_mode` cpp! 块、枚举重复定义、volatile 方法跳过、`is_from_current_file` 来源追踪） | ✅ 完成 |
| **Phase 12** | `merge output` 子命令（导出 Cargo 项目结构到任意目录） | ✅ 完成 |
| **Phase 13** | `api-manifest.md` 生成（merge 阶段生成 C++ → Rust API 对账清单，Markdown 格式，含降级标记） | ✅ 完成 |
| **Phase 14** | 五大主流开源库 E2E 测试（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib），多平台 CI 覆盖（Linux / macOS / Windows MinGW / Windows MSVC） | ✅ 完成 |
| **Phase 15** | 举一反三：为全部 E2E 测试补充 merge 阶段 + `cargo check` 验证（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib），完整覆盖 init→merge→编译可达性三段链路 | ✅ 完成 |

### 5.3 测试通过率

| 层 | 状态 |
|----|------|
| **L1**（golden 比对） | ✅ **49 / 49**（全部通过） |
| **L2**（编译测试）| ✅ **48 / 48**（全部通过）|
| **L3**（运行测试）| ✅ **48 / 48**（全部通过）|
| **L4 E2E**（五大库）| ✅ tinyxml2 / pugixml / nlohmann-json / fmtlib 全平台通过；sqlite3 Linux 通过（macOS / Windows 因系统头路径差异自动跳过）|
| **L4 merge + cargo check** | ✅ tinyxml2 / pugixml / fmtlib / nlohmann-json / sqlite3（Linux）全部 merge 阶段 + `cargo check` 通过 |

---

## 6. 开发环境搭建

### 6.1 Linux（Ubuntu 24.04）

```bash
# 系统依赖
apt-get install clang libclang-dev g++ libstdc++-14-dev

# 构建工具
cargo build

# 运行 L1 测试（必须单线程）
cargo test --test l1_golden_tests --features full-test -- --test-threads=1

# 运行单个示例测试
cargo test --test l1_golden_tests test_006_class_basic --features full-test

# 更新 golden 文件（工具输出有意变更时使用）
cargo test --test l1_golden_tests update_all_goldens --features full-test

# 合并单个 feature 的输出
cargo run -- merge --feature default

# 运行 L2 编译测试
cargo test --test l2_compile_tests

# 运行 L3 运行测试（首次自动编译 C++ 库，二次直接运行）
cargo test --test l3_run_tests --features full-test -- --test-threads=1
```

### 6.2 macOS

#### 前提条件

```bash
# 1. 安装 Xcode Command Line Tools（提供 make、ar、nm 等基础工具）
xcode-select --install

# 2. 安装 Homebrew LLVM（提供 libclang + clang++；版本不受 SIP 保护）
brew install llvm

# 3. 设置 LIBCLANG_PATH（建议永久写入 shell 配置文件）
echo 'export LIBCLANG_PATH=$(brew --prefix llvm)/lib' >> ~/.zprofile
source ~/.zprofile

# 4. 将 Homebrew LLVM 加入 PATH（让 clang++ 被工具和测试找到）
echo 'export PATH="$(brew --prefix llvm)/bin:$PATH"' >> ~/.zprofile
source ~/.zprofile
```

#### 构建和测试命令（与 Linux 完全一致）

```bash
# 构建
cargo build

# 运行 L1 测试
cargo test --test l1_golden_tests --features full-test -- --test-threads=1

# 运行 L2 编译测试
cargo test --test l2_compile_tests

# 运行 L3 运行测试（首次自动编译 C++ 动态库，约 2-4 分钟；二次直接运行）
cargo test --test l3_run_tests --features full-test -- --test-threads=1

# 也可通过 make 一步完成（参见 §7.3）
make l3-test

# 运行 L5 nm 符号验证测试
cargo test --test l5_nm_symbol_tests -- --ignored --nocapture --test-threads=4
```

#### 已知限制：macOS SIP（系统完整性保护）

`DYLD_INSERT_LIBRARIES`（工具用于编译拦截的 macOS 等价 `LD_PRELOAD`）对受 SIP 保护的系统
二进制（`/usr/bin/g++`、`/usr/bin/clang++`）无效，SIP 会静默忽略注入。

**解决方案**：始终使用 Homebrew 安装的编译器（位于 `/opt/homebrew/bin/` 或
`/usr/local/bin/`，不受 SIP 保护）：

```bash
# 推荐：将 Homebrew LLVM 加入 PATH 最前面
export PATH="$(brew --prefix llvm)/bin:$PATH"
cpp2rust-demo init -- make -j4

# 或：通过环境变量显式指定编译器
CPP2RUST_CXX=$(brew --prefix llvm)/bin/clang++ cpp2rust-demo init -- make -j4
```

### 6.3 L3 运行测试快速启动

L3 运行测试需要预先编译各示例的 C++ 动态库（每个约 1-3 秒，共 48 个示例）。
有三种方式准备环境：

#### 方式 A：自动编译（推荐）

```bash
# 直接运行测试 — 缺少的库会自动编译，首次约 2-4 分钟，二次直接运行
cargo test --test l3_run_tests -- --include-ignored --test-threads=1
```

`common::ensure_cpp_lib()` 在每个测试执行前检查库文件是否存在，若不存在则自动调用
`g++`（Linux）或 `clang++`（macOS）编译。已有库走快速路径，零额外开销。

#### 方式 B：Makefile 快捷命令

```bash
make l3-setup   # 仅编译所有 C++ 库（不运行测试）
make l3-test    # 编译库 + 运行所有 L3 测试
```

#### 方式 C：批量预编译脚本

```bash
# Linux / macOS
bash scripts/build_cpp_libs.sh

# 只编译指定示例
bash scripts/build_cpp_libs.sh 001_hello_world 006_class_basic

# Windows PowerShell
.\scripts\build_cpp_libs.ps1
```

---

## 7. 关键设计决策

| 决策 | 原因 |
|------|------|
| 只处理模板实例化结果，不处理模板声明 | 模板的价值在于实例化结果；未实例化的模板对 FFI 无意义 |
| 使用 `g++ -E -C` 产出 `.cpp2rust` 而非直接解析原始源文件 | 宏展开后 clang 可获得完整类型信息；保留 `-C` 注释和行号 marker 便于溯源 |
| 系统头过滤（`is_in_system_header()`） | `g++ -E -C` 会展开 `#include <vector>` 等，产生数万行无关代码，不过滤会提取大量 `std::allocator` 等无用模板特化 |
| 最小 shim 策略：方法用 `import_class!`，只为必要场景建 shim | 减少生成代码量；ctor/dtor/operator/static成员/placement new 才需要 shim |
| 降级特性用内联 `cpp2rust-todo[TAG]` 注释 | 让开发者在工具生成的代码中直接看到待手动完善的位置 |
| L1 测试须 `--test-threads=1` | clang crate 使用全局 libclang 状态，多线程并发解析会竞争 |
