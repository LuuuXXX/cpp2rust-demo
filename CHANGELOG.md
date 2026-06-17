# Changelog

本文件遵循 [Keep a Changelog](https://keepachangelog.com/) 格式。

## [Unreleased]

### 新增（工作流 A：实际项目本地验证脚本）

- **新增 7 个实际项目本地验证脚本 + 共享库 `verify-common.sh` + 统一入口 `verify-all.sh`**：为 E2E 已覆盖的 7 个真实库（tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib / magic_enum / tomlplusplus）各补一份可本地直接执行的 FFI 验证脚本 `usage/verify-<lib>-ffi.sh`，复刻 `usage/verify-rapidjson-ffi.sh` 的七阶段骨架（环境检查 → 安装工具 → 定位/编译源 → init → merge → build.rs 校验 → cargo check/test → 符号验证 → 汇报）。脚本只复用现有能力（cpp2rust-demo init/merge、cargo、nm、git 子模块），不引入新依赖、不改 `src/`。
- **共享库 `usage/lib/verify-common.sh`**：将与库无关的通用逻辑下沉为 `vc_*` 可复用函数（颜色/日志、`need_cmd`、`to_build_path` 跨平台、`SCRIPT_ERRORS` 汇总、EXIT trap 清理，及 `vc_check_env`/`vc_install_tool`/`vc_init`/`vc_merge`/`vc_check_build_rs`/`vc_cargo_check`/`vc_cargo_test`/`vc_verify_ffi`/`vc_report` 九个阶段函数 + `vc_run` 编排入口）；全部参数经环境变量（`LIB_NAME`/`FEATURE`/`SOURCES`/`INCLUDES`/`HEADER_ONLY`/`DRIVER_CPP`/`CXX_STD` 等）传入。子模块缺失时自动 `git submodule update --init`，失败则 `warn` 跳过、不中断。
- **统一入口 `usage/verify-all.sh`**：顺序（或按 `LIBS=` 过滤）调用全部 `verify-*-ffi.sh`，每库独立计错、末尾汇总通过/跳过/失败矩阵，单库失败不阻断其余库（最终非零退出供 CI 捕获）。
- **脚本可发现性测试**：新增 `tests/verify_scripts_discoverability_test.rs`，断言 `usage/verify-<lib>-ffi.sh` 集合与 `.github/workflows/e2e-<lib>.yml` 工作流集合一一对应，并校验每份 per-library 脚本均 `source` 共享库且调用 `vc_run`，防止未来新增真实库时漏配脚本或工作流。
- **文档对齐**：`usage/README.md` 的脚本表由 1 行扩为 11 行（rapidjson + 7 库 + 共享库 + 统一入口 + 本文档），补每库 header-only / 实现 .cpp / 系统头三类说明、通用与每库专属环境变量、`verify-all.sh` 用法；`README.md` 的 L4 真实库列表补全 magic_enum / tomlplusplus、新增「本地验证脚本」指引与 macOS（Homebrew：`llvm`、`cmake`）安装说明。

### 新增（方案 A：build.rs 自动注入捕获的编译元数据）

- **生成的 `build.rs` 不再需要外部脚本就地改写**：`init` 阶段从 LD_PRELOAD hook 记录的 `.opts`（`-I`/`-isystem`/`-iquote` include 路径与 `-std=`）还原编译选项，并由 `.cpp2rust` 路径反推被绑定符号定义所在的实现 `.cpp`，聚合为编译元数据落盘到 `meta/build-meta.json`（新增模块 `src/build_meta.rs`）。
- **`project_generator::write_build_rs` 据元数据注入 `cc::Build`**：当元数据非空时，生成的 `build.rs` 自动注入 `cc_build.std(...)` / `cc_build.include(...)` / `cc_build.file(...)` 并在非 MSVC 平台链接 `stdc++`，使端到端 `cargo check` / `cargo test` 可直接编译并链接第三方库实现；元数据为空时（黄金 / `gen-verify` 直接调用生成器）退化为最小化输出，产物逐字节不变。
- **`usage/verify-rapidjson-ffi.sh` §5a 改为信任工具产物**：检测工具生成的 `build.rs` 是否已自包含（含 `cc_build.include` + `cc_build.file`），是则跳过就地改写（方案 A 生效）；否则退回脚本就地补全（方案 B 兜底，兼容旧版工具或未捕获 `.opts` 的情形）。

### 变更（收尾三项：甄别对照 / 冒烟双值往返 / 文档对齐）

- **hicc-usages 甄别对照文档**：新增 `docs/references/hicc-usages-comparison.md`，系统分析 `references/hicc-usages`（48 特性 × hicc FFI 映射的参考实现）与本仓 `examples/` 的关系，逐项记录**采纳**（去 shim 直出形态、行为级冒烟样板、`tools/` AST 脚本）、**甄别修正**（继承绑定 `this` 偏移 SIGSEGV、`make_unique` 指针/标量实参 `&&` 转发、命名空间类型限定、友元函数类体内 inline 以保留直出）与**有意分歧**（黄金/L1–L6 测试体系、命名空间扁平化命名兼容）；从 `references/README.md` 链接。
- **冒烟生成器升级为「双值往返」行为级断言**：`src/generator/smoke_test_gen.rs` 的 setter/getter 往返由单值升级为双值——构造 → `set(A)` → `assert_eq!(get, A)` → `set(B)` → `assert_eq!(get, B)`（A≠B），进一步证明 getter 真实回读写入值而非恰好返回与首字面量相等的常量；安全约束不变（仅零参构造 + 严格命名 + 标量类型时生成，保证真实库 E2E `cargo test` 安全）。
- **文档冒烟措辞二次对齐**：`README.md` / `docs/INTRODUCTION.md` 的冒烟测试小节由「验证类型可编译链接」改述为「类型可用性 + 零参调用 + 标量 setter/getter 双值往返**行为级断言**」；L_smoke 层描述由「14 个迁移示例（015–018/023–027/034–038）」更正为「48/48 全示例（CI `l-smoke` 自动发现）」。

### 变更（文档瘦身：直出为默认）

- **README 重构为「hicc 直出（无 shim）为默认」叙事**：改写工具定位与主要特性，将「必要 C 桥接 shim」措辞替换为「直出 + 少数特性必要 `cpp!` 内联包装」；「生成代码格式（三段式）」示例由旧的 `*_new`/`*_delete` opaque shim 改为真实命名空间类 `#[cpp(class = "ns::T")]` + `make_unique` 工厂的实际直出产物；将「对纯 C++ 库使用 shim 工作流」整段（含 shim 头/实现示例）压缩为简短的「兼容性回落」说明；特性矩阵补充说明，澄清「FFI 策略」列为历史概念注解、实际产物以各 `examples/NNN/rust_hicc/src/lib.rs` 直出形态为准。文档变更，不影响工具行为与生成产物。

### 变更（去 shim 补齐 + 冒烟行为级 + 仓库瘦身）

- **001–005 示例去 shim**：将 `001_hello_world`…`005_variadic_functions` 的头/实现由 `extern "C"` 改为命名空间内自由函数（`namespace <feat>_ns`），`rust_hicc/src/lib.rs` 改用 `#[cpp(func = "ns::fn()")]` 经 `import_lib!` 直出绑定，与 006+ 的 hicc 直出形态对齐；补齐 `cpp/main.cpp`、`cpp/standalone.sh`、`cpp/Makefile`，README 改写为命名空间形态。验证通过 L1（48/48）/L2/L3/L5 与示例行为级冒烟。
- **冒烟测试生成器升级为行为级**：`src/generator/smoke_test_gen.rs` 新增表驱动的 setter/getter 往返检测——对「含零参构造 + 严格配对的 `set_<x>(标量)` 与 `<x>()`/`get_<x>()`/`is_<x>()` 标量 getter」的类，生成确定性 `assert_eq!` 往返断言（构造→set→断言 get 返回写入值）。断言仅在结果可静态确定（严格命名 + 标量类型）时生成，保证对真实项目 E2E `cargo test` 安全；其余项保留最小化 `cpp2rust-todo[SMOKE]` 占位。
- **仓库瘦身**：将被 Git 跟踪的 `examples-target/`（cargo 构建产物，915 文件）移出版本控制，并在 `.gitignore` 增补 `examples-target/` 忽略规则；该目录在 CI 中仅作为 `CARGO_TARGET_DIR` 使用，去版本控制不影响功能。
- **AST 可追溯工具**：新增 `scripts/dump_ast.sh` + `scripts/filter_ast.py`（源自 `hicc-usages/tools/`），对某示例转储宏展开 `.i`、完整 `ast.json` 与「仅用户自有声明」的过滤 `user-ast.json`，便于人工核对工具抽取的 IR；新增 `make dump-ast DIR=...` 目标，产物写入 `<dir>/../ast/` 并经 `.gitignore` 忽略（绝不入库百 MB 级 JSON）。
- **references 子模块决策文档化**：新增 `references/README.md` 说明各子模块用途，并明确 `references/rapidjson-refactoring` 保留 vendored 的理由——它是本仓特有的 rapidjson 重构工作区（含 `rapidjson_legacy`/`rapidjson_sys`/`baseline`/`inventory`/`reports`），无对应独立上游仓可指向，且 E2E 按固定相对路径取数，子模块化收益为负。
- **新增真实项目 E2E（独立 CI）**：新增两个 header-only 真实库作为 E2E 依赖——`magic_enum`（重度 `constexpr`/模板元编程）与 `tomlplusplus`（toml++，大型单头 + 重度模板），各自以子模块引入，新增 `tests/{magic_enum,tomlplusplus}_e2e_test.rs`（init+merge+`cargo check` 门禁，与 nlohmann/json E2E 同构）与独立工作流 `.github/workflows/e2e-{magic-enum,tomlplusplus}.yml`；`Makefile` 的 `submodules`/`l4-test` 同步纳入。E2E 真实库由 6 增至 8。

### 变更（v7：高级映射能力默认生成，移除环境变量开关）

- **移除全部 `CPP2RUST_GEN_*` 生成开关**：删除 `CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` / `CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE` 四个环境变量及相关的 `*_enabled()` / `*_ENV` 基础设施（`hicc_codegen` 的 `templates_enabled` / `proxy_enabled` / `dynamic_cast_enabled` / `env_switch_enabled`）。生成路径由「开/关双路径」收敛为「IR 非空即输出」单路径。`smoke_test_gen` 模块本身保留，但不再受环境变量控制。
- **模板类 / 模板函数 / 实例化别名 / 构造工厂、`@make_proxy`、`@dynamic_cast`、冒烟测试 `tests/smoke.rs` 一律默认生成**，命令签名（`init` + `merge`）与目录结构不变；以「文件级幂等」（已存在的用户改动不覆盖）替代开关。
- **模板骨架以注释形式输出**：因「未实例化的模板没有可链接符号、泛型 `<T>` 不可直接编译」，模板类 / 函数 / 别名 / 工厂默认以**注释骨架**（带 `cpp2rust-todo[TMPL]` 指引）输出，保证工具默认产物始终可通过 L6 gen-verify 编译；`@make_proxy` / `@dynamic_cast` 使用 hicc 内建指令、对接具体类型，默认输出为可编译的活动绑定。
- **行为变更提示**：依赖上述开关「默认关闭、产物逐字节不变」的旧行为不再成立——首次对含模板/接口/多态的示例运行 `init` 会额外生成对应骨架（模板为注释、proxy/dynamic_cast 为活动绑定）。

### 测试 / 文档

- 重写 `tests/{template,proxy,dynamic_cast}_gen_tests.rs`，去掉环境变量串行化（`set_var`/`remove_var`），改为断言默认产物；新增「模板骨架须为注释行」契约断言。
- `tests/l1_golden_tests.rs` 新增 `golden_test_scaffold!` 宏；024 模板函数示例黄金 `lib_scaffold.rs` 更新为注释骨架形式。
- README / INTRODUCTION / hicc.md / DEVELOPMENT 全面对齐「默认生成、无开关」；新增 `docs/plans/v7/`。

### 移除

- **FFI 冒烟测试环境变量控制**：移除 `smoke_test_gen::smoke_enabled` 环境变量开关及相关代码，冒烟测试默认生成且不再受 `CPP2RUST_GEN_SMOKE` 控制。`src/generator/smoke_test_gen.rs` 模块及 `project_generator::write_smoke_test` 函数保留，冒烟测试在 `init` 阶段默认生成。
- 移除以下已废弃的基础设施：
  - `layout::SmokeTestEntry` 结构体及 `ApiManifest::smoke_tests` 字段
  - `layout::parse_smoke_test_entries` 函数
  - `api-manifest.md` 中的冒烟测试章节
  - 测试文件 `tests/l1_smoke_test_gen_tests.rs`
  - `tests/rapidjson_e2e_test.rs` 中两个冒烟测试相关函数
  - CI 中 5 个冒烟测试专属 job（`l1-smoke-test-gen` 四平台 + `smoke-test-cargo-check`）

### 修复

- **`block_parser.rs`**：`parse_class_content` 现在正确处理 `pub class Foo {` 形式（codegen 生成的标准格式），修复跳过所有方法绑定的 bug。

### 优化（源码）

- **`lib_spec.rs`**：将文件头模块注释从日语改为中文，与代码库其他文件保持一致。
- **`init.rs`**：`first_pass_parse` 改为"收集所有失败"策略，解析失败的文件记录为警告并跳过，最终汇总打印失败文件列表，而非遇到第一个失败就中止。
- **`merge.rs`**：消除 `run_single_feature_merge` / `run_multi_feature_merge` 中重复的 `current_dir()` + `find_project_root()` 两行代码，统一在 `run_merge` 中获取后传入各子函数。
- **`hicc_codegen.rs`**：用索引判断代替 `out.ends_with("\n\n") { out.pop() }` 的字符串末尾 hack，通过 `if i + 1 < methods.len()` 有条件添加方法间空行。
- **`type_mapper.rs`**：统一使用 `#[cfg(target_os = "windows")]` 并添加注释说明与 `#[cfg(windows)]` 等价，选择前者以保持平台 cfg 写法一致性。
- **`capture.rs`**：将 `hook_dir()` 中 `[Option; 2].into_iter().flatten()` 改写为 `filter_map(|opt| opt)`，提升可读性。

### 优化（测试）

- **`tests/common/mod.rs`**：`normalize` 函数保留含 `cpp2rust-todo` 的降级标记注释行，不再将其当作普通注释剥除；`assert_valid_hicc_format` 改为块级精确检查，避免跨块的 `fn ` 误判；新增 `assert_contains_todo_tag` 辅助断言。
- **`tests/l1_golden_tests.rs`**：新增 `todo_tag_test!` 宏和 4 个降级标记专项断言（031/039/040/047），直接验证 `cpp2rust-todo[FP]` 注释是否正确生成。
- **`tests/l2_compile_tests.rs`**：`compile_test!` 宏加入 `#[cfg_attr(not(feature = "full-test"), ignore)]` 保护，在无 C++ 工具链的 CI 环境中自动忽略，与 L1 保护策略一致。
- **`tests/l4_merge_integration_tests.rs`**（新增）：新增 merge 集成测试，覆盖 `merge_in_place` 备份/rename、重复运行幂等性、`merge_units` 去重与类提取、降级签名收集以及 `collect_unit_rs_files` 目录扫描，无需 C++ 工具链。

### 优化（错误路径测试）

- **`src/error.rs`**：新增 5 个 `Cpp2RustError::Display` 格式测试，覆盖所有错误变体。
- **`src/layout/io.rs`**：新增 7 个单元测试，覆盖 `save_init_report`、`save_merge_report` 的正常路径和边界情况。

### 优化（文档）

- **`README.md`**：补充 `--output-dir` 多 feature 输出目录结构说明及 CI/CD 集成典型使用场景示例（CMake 构建后导出、交叉编译多平台、GitHub Actions 工件上传）。
- **`docs/INTRODUCTION.md`**：补充 Phase 6（merger）技术细节：`merge_in_place` 原子性 rename 机制、跨翻译单元 `cpp_lines` 去重策略、模板特化分组、冲突检测与报告生成。
- **`DEVELOPMENT.md`**：目录树补全 `extractor/lib_spec.rs`、`extractor/class_spec.rs`、`extractor/cpp_block.rs` 三个子模块；六阶段描述与 CHANGELOG 统一。

---

## [0.1.0] — 2026-06-01

### 新增

- **五层测试体系**：L1 黄金文件测试、L2 编译测试、L3 运行测试、L4 端到端 E2E 测试（rapidjson / tinyxml2 / pugixml / sqlite3 / nlohmann_json / fmtlib）、L5 `nm` 符号验证测试。
- **完整 hicc 三段式代码生成**：从 C++ 源文件生成 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式 FFI 脚手架。
- **五阶段处理流水线**：
  - Phase 1：编译拦截（`hook.cpp` / `capture.rs`）
  - Phase 2：AST 解析（`ast_parser.rs`，基于 libclang）
  - Phase 3：IR 提取（`extractor/`，输出 `FfiSpec`）
  - Phase 4：后处理（`postprocessor/`，菱形继承 + 运算符重载处理）
  - Phase 5：代码生成（`generator/`，输出 `lib.rs`）
  - Phase 6：多 feature 合并（`merger/`，输出可独立编译的 Rust 项目）
- **多 feature 合并支持**：`merge --feature a --feature b` 将多个 `.cpp2rust` feature 合并为统一 crate。
- **类型映射**：支持 C++ 原始类型、指针、引用、C 函数指针 → Rust FFI 类型自动映射（遵循 LP64 约定）。
- **关联函数归属**：ctor/dtor/factory 函数自动归属对应 `ClassSpec::associated_fns`。
- **菱形继承处理**：自动去重菱形继承场景下的重复方法绑定。
- **运算符重载处理**：自动识别并标注比较运算符、赋值运算符等绑定类别。
- **API manifest 输出**：`merge` 后生成 `meta/api-manifest.md`，汇总所有导出接口（Markdown 格式）。
- **完整 README / INTRODUCTION 文档**：包含快速入门、类型映射规则、流水线架构说明和 10+ 个示例。
