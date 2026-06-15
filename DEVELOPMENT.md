# cpp2rust-demo 开发文档

> 工具介绍见 [README.md](../README.md)，深度技术方案见 [docs/INTRODUCTION.md](../docs/INTRODUCTION.md)。

---

## 1. 架构概览

```
cpp2rust-demo (bin)
│
├── hook/                        # LD_PRELOAD 拦截器（内嵌进 binary）
│   ├── hook.cpp                 # 拦截 g++/clang++ → .cpp2rust 预处理文件
│   └── Makefile
│
└── src/
    ├── main.rs                  # CLI 入口：init / merge
    ├── capture.rs               # hook 编译 + LD_PRELOAD 注入
    ├── layout/                  # 目录布局 .cpp2rust/<feature>/c|meta|rust
    │   ├── mod.rs               # 公开接口
    │   ├── types.rs             # ApiManifest、FeatureLayout 等
    │   └── io.rs                # I/O 方法与报告生成
    ├── selector.rs              # 交互式文件选择
    ├── ffi_model.rs             # FFI 中间表示（FfiSpec / ClassSpec / FnBinding）
    ├── error.rs                 # 统一错误类型
    ├── ast_parser/              # Phase 2：C++ AST 解析
    │   ├── mod.rs               # parse_preprocessed()
    │   ├── collector.rs         # 类/函数/枚举/模板收集
    │   └── range_scanner.rs     # 字节范围扫描（用户代码 vs 头文件）
    ├── extractor/               # Phase 3：CppAst → FfiSpec IR 提取
    │   ├── mod.rs               # extract()、命名空间模式检测
    │   ├── type_mapper.rs       # C++ → Rust 类型映射
    │   ├── class_spec.rs        # ClassSpec 构建
    │   ├── lib_spec.rs          # import_lib! 块构建
    │   ├── cpp_block.rs         # hicc::cpp! 块提取与 shim 内联
    │   ├── direct_binding.rs    # Direct Binding 模式分类与类/函数构建
    │   ├── dynamic_cast_spec.rs # @dynamic_cast 生成
    │   ├── proxy_spec.rs        # @make_proxy 生成
    │   └── template_spec.rs     # 模板实例化处理
    ├── postprocessor/           # Phase 4：特殊情况后处理
    │   ├── operator_handler.rs  # 运算符重载 [OP]
    │   └── diamond_handler.rs   # 菱形继承命名 shim
    ├── merger/                  # Phase 6：merge 命令
    │   ├── mod.rs               # merge_units() 去重 + 冲突检测
    │   └── block_parser.rs      # 解析单个 .rs 中的 hicc 块
    ├── metrics.rs               # 源码行数统计、todo 标签解析
    └── generator/               # Phase 5：FfiSpec → Rust 代码
        ├── hicc_codegen.rs      # 三段式代码生成
        └── project_generator.rs # Cargo.toml / lib.rs / build.rs 生成
```

### 六阶段处理流程

```
Phase 1        Phase 2          Phase 3         Phase 4        Phase 5         Phase 6
编译拦截       AST 解析         IR 提取         后处理         代码生成        合并整理
LD_PRELOAD     ast_parser/      extractor/      postprocessor/ generator/     merger/
→ .cpp2rust    → CppAst         → FfiSpec       → 特殊情况     → hicc Rust    → src/ 整理
```

### 输出目录结构

```
.cpp2rust/<feature>/
├── c/                    # .cpp2rust 预处理文件
├── meta/                 # build_cmd.txt、init/merge-report.md、api-manifest.md
└── rust/
    ├── src.1/            # init 原始备份（merge 时 rename from src）
    └── src/              # merge 输出（与 C++ 目录结构一致）
```

---

## 2. 示例支持矩阵

| 类别 | 编号 | 状态 |
|------|------|------|
| 基础函数 | 001–005 | ✅（005 ⚠️[CV]） |
| 类与对象 | 006–012 | ✅（012 ⚠️[VM]） |
| 面向对象 | 013–018 | ✅ |
| 运算符 | 019 | ⚠️[OP] |
| 友元/RTTI | 020–023 | ✅ |
| 模板实例化 | 024–028 | ✅（028 ⚠️[VA]） |
| 智能指针/内存 | 029–033 | ✅ |
| STL 容器 | 034–038 | ✅ |
| 函数对象 | 039–042 | ✅（039/040 ⚠️[LM]） |
| 高级特性 | 043–048 | ✅ |

降级标签说明见 [README.md](../README.md#降级特性摘要)。

---

## 3. 测试体系

| 层 | 文件 | 验证内容 |
|----|------|---------|
| L1 | `l1_golden_tests.rs` | 黄金文件比对（49/49 ✅） |
| L2 | `l2_compile_tests.rs` | 编译测试（48/48 ✅） |
| L3 | `l3_run_tests.rs` | 运行输出验证 |
| L4 | `tinyxml2_e2e_test.rs` | E2E 端到端 |
| L4 | `l4_merge_integration_tests.rs` | merge 集成（不依赖 C++ 工具链） |
| L_smoke | 各示例 `tests/smoke.rs` | 冒烟测试 |
| L5 | `l5_nm_symbol_tests.rs` | nm 符号验证（仅 Linux） |
| L6 | `gen_verify_e2e_test.rs` | gen-verify 生成验证 |

**L1 须单线程**：`--test-threads=1`（clang 全局状态竞争）。

**L1 通过 `full-test` feature 控制**：不加 `--features full-test` 自动跳过。

### 快速运行

```sh
cargo test --lib                                        # 单元测试
cargo test --test l1_golden_tests --features full-test -- --test-threads=1
cargo test --test l2_compile_tests
cargo test --test l5_nm_symbol_tests -- --ignored
```

---

## 4. 代码质量

CI 门控：`cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test`。

已豁免 lint：`clippy::too_many_lines`（生成逻辑）、`clippy::match_wildcard_for_single_variants`（operator_handler）、`dead_code`（测试辅助）。

---

## 5. 开发环境

### Linux

```bash
apt-get install clang libclang-dev g++ libstdc++-14-dev
cargo build
cargo test --test l1_golden_tests --features full-test -- --test-threads=1
```

### macOS

```bash
brew install llvm
export LIBCLANG_PATH=$(brew --prefix llvm)/lib
export PATH="$(brew --prefix llvm)/bin:$PATH"
cargo build
# 测试命令与 Linux 一致
```

> macOS SIP 限制：始终使用 Homebrew 编译器，不走 `/usr/bin/g++`。

---

## 6. 如何添加新示例

1. 创建 `examples/<N>_<name>/` 目录
2. 编写 `cpp/<name>.cpp`（含 `extern "C"` 导出的函数/类）
3. 编写 `rust_hicc/src/main.rs`（hicc 三段式 + `fn main()`）
4. 编写 `rust_hicc/Cargo.toml`（依赖 hicc + hicc-build）
5. 运行 `cargo test --test l1_golden_tests --features full-test -- --test-threads=1` 验证黄金比对
6. 运行 `cargo test --test l2_compile_tests` 验证编译
7. 更新 DEVELOPMENT.md 支持矩阵

---

## 7. Direct Binding 模式

`extractor/direct_binding.rs` 根据 AST 内容自动判定绑定模式：

- **判定规则**：若任何 `extern "C"` 函数的返回值或首参为类指针 → Shim；否则 Direct。
- **Shim 模式**（默认）：为 ctor/dtor/operator/static/placement new 生成 C 适配层（`hicc::cpp!` + 命名 shim）。
- **Direct 模式**：类方法通过 `#[cpp(method = "...")]` 直接绑定，工厂通过 `make_unique<T>` 生成，无需 C shim 包装。

两种模式的代码路径均在 `extractor/` + `hicc_codegen` 中保留。详见 [docs/direct-vs-shim-binding.md](../docs/direct-vs-shim-binding.md)。
