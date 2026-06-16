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
| L4 | `l4_merge_integration_tests.rs` | merge 核心逻辑集成测试：`merge_in_place` 备份与原子 rename、`merge_units` 去重与类绑定提取、`collect_unit_rs_files` 目录扫描；不依赖 libclang 或 g++ | `cargo test --test l4_merge_integration_tests` |
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

**冒烟测试生成（L_smoke）**：`init` 默认在 `.cpp2rust/<feature>/rust/tests/smoke.rs` 生成冒烟测试，对生成的 pub class 类型做编译期可用性断言（私有工厂函数以 `cpp2rust-todo[SMOKE]` 占位列出）。进入生成目录执行 `cargo test` 即可验证 FFI 编译链接闭环。生成逻辑见 `src/generator/smoke_test_gen.rs`，写文件见 `project_generator::write_smoke_test`（幂等，已存在不覆盖）。**v7 起默认生成、无开关**。

**模板泛型骨架生成（v6 Phase A/B）**：`ast_parser` 现已提取模板类（`ClassTemplate`）与模板函数（`FunctionTemplate`）的结构化信息（`CppAst.template_classes` / `template_functions`，见 `src/ast_parser/collector.rs` 的 `extract_template_class` / `extract_template_function`）。`extractor::template_spec` 据此构建 `TemplateClassSpec` / `TemplateFnSpec`，生成器 `hicc_codegen` 输出泛型 `import_class!`（`#[cpp(class = "template<class T> Name<T>")] pub class Name<T> { ... }`）与 `import_lib!`（`#[cpp(func = "ret name<T>(...)")]`）骨架。**v7 起该能力默认生成、不再有任何环境变量开关**；因未实例化的模板无可链接符号、泛型 `<T>` 不可直接编译，模板类 / 函数骨架默认以**注释**形式输出（带 `cpp2rust-todo[TMPL]` 占位注释，保证默认产物可编译），提示用户按实际实例化类型补全签名与 `AbiType` 约束后取消注释。生成行为测试见 `tests/template_gen_tests.rs`。

**模板实例化别名（v6 Phase B 增强）**：在上述泛型骨架基础上，`extractor::template_spec::build_template_instances` 从当前编译单元中「以具体类型实例化本文件模板类」的使用点（目前为包装类的字段类型，如 `Stack<int> impl;`）收集 `(模板名, 实参)`，生成器 `emit_template_instances` 输出类型别名骨架（POD 标量用 `hicc::Pod<...>` 包装，如 `pub type StackI32 = Stack<hicc::Pod<i32>>;`；类类型实参保留原名并附 `cpp2rust-todo[TMPL]` 提示替换为对应 hicc 类型）。别名随模板类骨架一并以注释形式默认输出（v7 起无开关）。

**模板实例化追踪扩展 + 构造工厂骨架（v6 Phase B 增强（续））**：`build_template_instances` 的实例化追踪来源从「字段类型」扩展到**方法参数 / 返回类型**与**全局函数参数 / 返回类型**（如 `void use(Stack<short>& s)` 也能收集到 `Stack<short>` 别名）。在此基础上，`extractor::template_spec::build_template_factories` 由模板类的公有构造函数派生**构造工厂骨架**：将类型参数 `T` 按实例化的具体类型替换（`substitute_type_params`，以完整标识符为单位替换，避免误伤 `Time` 等子串），生成器 `emit_template_factory` 在 `import_lib!` 中输出工厂函数（如 `#[cpp(func = "Stack<int>* stack_i32_new(int initial)")] pub unsafe fn stack_i32_new(initial: i32) -> StackI32;`）。工厂对应的 C++ 符号通常需用户显式实例化 / 包装后才存在，故附 `cpp2rust-todo[TMPL]` 提示。该能力随模板类骨架一并以注释形式默认输出（v7 起无开关）。

**显式实例化 + 局部变量声明追踪（v6 Phase B 增强（再续）/ 收尾）**：实例化追踪进一步扩展到 **显式实例化** `template class Foo<int>;`（libclang 表现为带模板实参的 `ClassDecl`，实参由 `ClassInfo::template_args` 携带）与 **函数 / 方法体内局部变量声明**（如 `Stack<int> s;`、`Stack<int>* p = new Stack<int>();`）。后者由 `ast_parser` 第三遍 `collector::collect_local_var_types` 递归收集函数 / 方法体内 `VarDecl` 的类型显示名（跳过系统头子树、仅限当前编译单元），写入 `CppAst.local_var_types`，再由 `build_template_instances` 的「来源 5」经 `collect_instance_from_type` 解析并去重。两者均复用既有 `build_instance_spec` / `build_template_factories`，随模板骨架一并以注释形式默认输出（v7 起无开关）。

**`@make_proxy` 代理工厂骨架（v6 Phase C：高级映射）**：在纯虚接口已映射为 `#[interface]`（`class_spec.rs`，参与默认产物）的基础上，`extractor::proxy_spec::build_proxy_factories` 识别「**继承 C++ 抽象接口的具体类**」（非抽象、且存在某个纯虚接口基类，接口判定复用 `class_spec::is_interface_class`），由其公有构造函数（排除拷贝 / 移动构造）派生 `ProxyFactorySpec`。生成器 `hicc_codegen::emit_proxy_factory` 在 `import_lib!` 中输出结合 `#[interface(name = "<直接接口基类>")]` 的 `@make_proxy` 工厂骨架（如 `#[cpp(func = "Baz @make_proxy<Baz>()")] #[interface(name = "Bar")] fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;`），第一个参数固定为 Rust 实现类（`hicc::Interface<具体类>`），其后接构造函数参数。**v7 起该能力默认生成、不再有任何环境变量开关**；`@make_proxy` 为 hicc 内建指令、对接具体类型，默认输出为可编译的活动绑定。骨架带 `cpp2rust-todo[PROXY]` 占位注释，提示用户提供接口实现并校验 `@make_proxy` 参数类型列表。生成行为测试见 `tests/proxy_gen_tests.rs`。

**`@dynamic_cast` 下行转换骨架（v6 Phase C（续）：高级映射）**：`extractor::dynamic_cast_spec::build_dynamic_casts` 识别当前编译单元中「**继承自多态基类的派生类**」，派生 `DynamicCastSpec`。多态判定（`is_polymorphic`）为类自身或任一递归基类含虚函数 / 虚析构 / 纯虚方法，与 C++ 中 `dynamic_cast` 要求源类型为多态类型的约束一致。v6 Phase C（收尾）起，除直接基类外还遍历**递归祖先链**（`collect_ancestors`）中的所有多态祖先，为「跨层（间接）继承」派生下行转换（如 `Foo <- Bar <- Baz` 额外派生 `Foo → Baz`），结果按 `(src, dst)` 去重，与 C++ 允许跨任意层级向下转换的语义一致。生成器 `hicc_codegen::emit_dynamic_cast` 在 `import_lib!` 中输出下行转换骨架（如 `#[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")] pub unsafe fn dynamic_cast_foo_to_bar(src: *const Foo) -> *const Bar;`），用于 RTTI 场景把多态基类指针向下转换为派生类指针，替代 v5 的整数枚举绕过方案（见 `references/hicc/examples/dynamic_cast`）。转换失败返回空指针，调用方需判空。**v7 起该能力默认生成、不再有任何环境变量开关**；`@dynamic_cast` 为 hicc 内建指令、对接具体类型，默认输出为可编译的活动绑定。骨架带 `cpp2rust-todo[DCAST]` 占位注释。v6 Phase C（收尾续）起，每个下行转换在裸指针形式之外再派生**引用形式**（`pub unsafe fn dynamic_cast_foo_to_bar_ref(src: &Foo) -> &Bar;`，函数名以 `_ref` 结尾，复用同一指针型 C++ 签名，见 `references/hicc/examples/dynamic_cast` 的 `as_foo(&self) -> &Foo`）。引用形式更符合 Rust 习惯，但**要求转换必定成功**——失败时由空指针构造引用属未定义行为，调用方无法确保类型成立时应改用裸指针形式并判空。生成行为测试见 `tests/dynamic_cast_gen_tests.rs`。

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

## 4.5 代码质量

### clippy 配置

项目在 CI 中运行 `cargo clippy` 且设定为零警告模式：

```sh
cargo clippy -- -D warnings
```

如需仅在本地快速检查（允许警告），可使用：

```sh
cargo clippy
```

### CI Gate

CI（`.github/workflows/ci.yml`）在每次 Push / Pull Request 时执行以下门控检查（顺序）：

1. `cargo fmt --check` — 格式化一致性（非阻塞 lint，不影响 clippy）
2. `cargo clippy -- -D warnings` — 零警告门控（所有 Warning 级别 lint 均阻塞合并）
3. `cargo test` — L2 编译测试（含 `--lib` 单元测试）

### 已豁免的 lint

项目中以下 lint 经过评估后通过 `#[allow(...)]` 局部豁免，不属于代码缺陷：

| lint | 位置 | 原因 |
|------|------|------|
| `clippy::too_many_lines` | 部分生成逻辑函数 | 历史原因，重构计划中 |
| `clippy::match_wildcard_for_single_variants` | `operator_handler.rs` | 枚举变体后续可扩展 |
| `dead_code` | 测试辅助结构体字段 | 字段在条件编译路径中使用 |

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
| **Phase 14** | 五大主流开源库 E2E 测试（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib），多平台 CI 覆盖（Linux / Windows MinGW / Windows MSVC） | ✅ 完成 |
| **Phase 15** | 举一反三：为全部 E2E 测试补充 merge 阶段 + `cargo check` 验证（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib），完整覆盖 init→merge→编译可达性三段链路 | ✅ 完成 |

### 5.3 测试通过率

| 层 | 状态 |
|----|------|
| **L1**（golden 比对） | ✅ **49 / 49**（全部通过） |
| **L2**（编译测试）| ✅ **48 / 48**（全部通过）|
| **L3**（运行测试）| ✅ **48 / 48**（全部通过）|
| **L4 E2E**（五大库）| ✅ tinyxml2 / pugixml / nlohmann-json / fmtlib 全平台通过；sqlite3 Linux 通过（Windows 因系统头路径差异自动跳过）|
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

### 6.2 L3 运行测试快速启动

L3 运行测试需要预先编译各示例的 C++ 动态库（每个约 1-3 秒，共 48 个示例）。
有三种方式准备环境：

#### 方式 A：自动编译（推荐）

```bash
# 直接运行测试 — 缺少的库会自动编译，首次约 2-4 分钟，二次直接运行
cargo test --test l3_run_tests --features full-test -- --test-threads=1
```

`common::ensure_cpp_lib()` 在每个测试执行前检查库文件是否存在，若不存在则自动调用
`g++`（Linux）编译。已有库走快速路径，零额外开销。

#### 方式 B：Makefile 快捷命令

```bash
make l3-setup   # 仅编译所有 C++ 库（不运行测试）
make l3-test    # 编译库 + 运行所有 L3 测试
```

#### 方式 C：批量预编译脚本

```bash
# Linux
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
