# cpp2rust-demo 开发文档

> 本文档记录项目的设计目标、架构、当前进度和后续计划，供所有开发者参考。

---

## 1. 项目目标

**cpp2rust-demo** 是一个 C++ → Rust Safe FFI 自动化脚手架生成工具（方案 v5）。

**核心目标**：给定一个任意 C++ 项目，开发者只需执行 `cpp2rust-demo init -- <构建命令>` 一条命令，工具自动完成以下流程：

1. 编译拦截（LD_PRELOAD hook）：捕获实际被编译的 C++ 文件及其预处理内容
2. AST 解析：用 libclang 解析宏展开后的 C++ 代码，提取类/函数/枚举/模板实例化
3. 代码生成：输出 `hicc` 宏格式的 Rust FFI 脚手架（`hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式）

工具**不**负责：
- 生成 `fn main()`（业务逻辑由开发者手写）
- 实现完整的语义等价翻译（只生成 FFI 绑定层）

**参考实现**：`references/c2rust-demo/`（同架构的 C 语言版本，LD_PRELOAD + clang + hicc）

---

## 2. 架构概览

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
    ├── layout.rs                # 目录布局（.cpp2rust/<feature>/c|meta|rust）
    ├── selector.rs              # 交互式文件选择
    ├── ffi_model.rs             # FFI 中间表示（FfiSpec / ClassSpec / FnBinding 等）
    ├── error.rs                 # 统一错误类型
    ├── ast_parser.rs            # C++ AST 解析（clang crate → CppAst）
    ├── extractor/               # Phase 2：CppAst → FfiSpec IR 提取
    │   ├── mod.rs               # 公共入口 extract()、命名空间模式检测、类/函数/枚举提取
    │   └── type_mapper.rs       # C++ 类型 → Rust 类型映射
    ├── postprocessor/           # Phase 4：特殊情况后处理
    │   ├── mod.rs
    │   ├── operator_handler.rs  # 运算符重载 [OP]
    │   └── diamond_handler.rs   # 菱形继承路径检测与命名 shim 生成
    └── generator/               # Phase 5：FfiSpec → Rust 代码
        ├── mod.rs
        ├── hicc_codegen.rs      # hicc 三段式代码生成
        └── project_generator.rs # Cargo.toml / lib.rs 生成
```

### 2.1 三阶段处理流程

```
编译拦截 (hook.cpp)                    AST 提取 (ast_parser + extractor)        代码生成 (generator)
LD_PRELOAD → g++ -E -C               clang crate 解析 .cpp2rust              FfiSpec → hicc 三段式 Rust
→ .cpp2rust 预处理文件         →      → CppAst（类/函数/枚举/模板）     →      → lib.rs + <unit>.rs
```

### 2.2 输出目录结构

```
.cpp2rust/<feature>/
├── c/                   # 预处理文件（.cpp2rust 后缀）
├── meta/                # build_cmd.txt、selected_files.json
└── rust/                # 生成的 Rust 项目
    └── src/
        ├── lib.rs
        ├── <unit>.rs         # 扁平文件（C++ 项目根目录下的编译单元）
        └── <subdir>/<unit>.rs# 带子目录（C++ 源文件位于 src/ 等子目录时）
```

### 2.3 生成代码格式（三段式）

```rust
// 段 1：C++ 实现内联（含必要 shim）
hicc::cpp! {
    #include "foo.h"
    Foo* foo_new(int value) { return new Foo(value); }
    void foo_delete(Foo* self) { delete self; }
}

// 段 2：类方法绑定（import_class!）
hicc::import_class! {
    #[cpp(class = "Foo")]
    pub class Foo {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

// 段 3：全局函数绑定（import_lib!）
hicc::import_lib! {
    #![link_name = "foo"]
    class Foo;
    #[cpp(func = "Foo* foo_new(int value)")]
    fn foo_new(value: i32) -> *mut Foo;
}
```

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

### 3.1 降级特性说明（4 项）

| TAG | 编号 | C++ 特性 | 不能完全自动的原因 | 自动降级策略 |
|-----|------|---------|-----------------|------------|
| `[OP]` | 019 | 运算符重载 | C ABI 无运算符符号 | 生成命名 shim（`{class}_add` 等）+ 内联 TODO |
| `[VA]` | 028 | 可变参数模板 | `...Args` 编译期展开，FFI 无法表达 | 生成 wrapper 类 + 按数量展开版本 |
| `[LM]` | 039 | 有状态 Lambda | 匿名闭包类型，FFI 无法表达捕获列表 | 无状态→函数指针；有状态→class wrapper |
| `[LM]` | 040 | std::function | 类型擦除容器，签名可推断但捕获不透明 | class wrapper + opaque pointer |

---

## 4. 测试体系

测试分五层，位于 `tests/` 目录：

| 层 | 文件 | 验证内容 | 运行命令 |
|----|------|---------|---------|
| L1 | `l1_golden_tests.rs` | 工具生成的 FFI 脚手架与 `rust_hicc/src/main.rs` 中对应段落一致 | `cargo test --test l1_golden_tests -- --include-ignored --test-threads=1` |
| L2 | `l2_compile_tests.rs` | 仓库中现有的 `rust_hicc/` 能通过 `cargo build` | `cargo test --test l2_compile_tests` |
| L3 | `l3_run_tests.rs` | `cargo run` 输出与 README 中"运行结果"一致 | `cargo test --test l3_run_tests -- --include-ignored --test-threads=1` |
| L4 | `rapidjson_e2e_test.rs` | 对 rapidjson 开源项目执行完整 init + merge 转换，验证 hicc 三段式格式 | `cargo test --test rapidjson_e2e_test -- --include-ignored` |
| L5 | `l5_nm_symbol_tests.rs` | 用 `nm` 双向验证 C++ 导出符号均已链接进 Rust FFI 二进制 | `cargo test --test l5_nm_symbol_tests -- --include-ignored` |

**L1 核心逻辑**：从 `rust_hicc/src/main.rs` 提取 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三种块作为黄金片段，与工具生成的 `lib.rs` 对应块比对，忽略 `fn main()` 和注释差异。

**重要**：L1 测试须单线程运行（`--test-threads=1`），多线程下 clang 全局状态存在竞争。

---

## 5. 当前进度（截至 2026-06-02，最新更新）

### 5.1 Phase 完成状态

| Phase | 内容 | 状态 |
|-------|------|------|
| **Phase T** | 测试基础设施（L1/L2/L3 框架） | ✅ 完成 |
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

### 5.3 测试通过率

| 层 | 状态 |
|----|------|
| **L1**（golden 比对） | ✅ **49 / 49**（全部通过） |
| **L2**（编译测试）| ✅ **48 / 48**（全部通过）|
| **L3**（运行测试）| ✅ **48 / 48**（全部通过）|

---

## 6. 后续计划

### 6.1 P2/P3 - 待后续跟进

- 模板跨翻译单元合并（当前每个 `.cpp2rust` 文件独立解析，跨文件的模板实例化可能遗漏；merge 阶段已通过去重部分缓解）
- L3 运行测试本地化（当前仅 CI 验证，建议补充本地快速运行脚本）

---

## 7. 开发环境搭建

```bash
# 系统依赖（Ubuntu 24.04）
apt-get install clang libclang-dev g++ libstdc++-14-dev

# 构建工具
cargo build

# 运行 L1 测试（必须单线程）
cargo test --test l1_golden_tests -- --include-ignored --test-threads=1

# 运行单个示例测试
cargo test --test l1_golden_tests test_006_class_basic -- --include-ignored

# 更新 golden 文件（工具输出有意变更时使用）
cargo test --test l1_golden_tests update_all_goldens -- --include-ignored

# 合并单个 feature 的输出
cargo run -- merge --feature default

# 运行 L2 编译测试
cargo test --test l2_compile_tests

# 运行 L3 运行测试
cargo test --test l3_run_tests -- --include-ignored --test-threads=1
```

---

## 8. 关键设计决策

| 决策 | 原因 |
|------|------|
| 只处理模板实例化结果，不处理模板声明 | 模板的价值在于实例化结果；未实例化的模板对 FFI 无意义 |
| 使用 `g++ -E -C` 产出 `.cpp2rust` 而非直接解析原始源文件 | 宏展开后 clang 可获得完整类型信息；保留 `-C` 注释和行号 marker 便于溯源 |
| 系统头过滤（`is_in_system_header()`） | `g++ -E -C` 会展开 `#include <vector>` 等，产生数万行无关代码，不过滤会提取大量 `std::allocator` 等无用模板特化 |
| 最小 shim 策略：方法用 `import_class!`，只为必要场景建 shim | 减少生成代码量；ctor/dtor/operator/static成员/placement new 才需要 shim |
| 降级特性用内联 `cpp2rust-todo[TAG]` 注释 | 让开发者在工具生成的代码中直接看到待手动完善的位置 |
| L1 测试须 `--test-threads=1` | clang crate 使用全局 libclang 状态，多线程并发解析会竞争 |
