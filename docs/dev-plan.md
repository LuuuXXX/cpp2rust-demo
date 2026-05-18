# cpp2rust-demo 可落地开发计划

> 本文档基于 V1 已实现能力全景，结合工具现有局限与 hicc 上游约束，制定 V2、V3 两个阶段的可落地开发计划。

---

## 一、V1 已实现能力总结（基线）

V1 已全部落地的内容，是后续版本的能力基础，不在 V2/V3 重复实现。

### 1.1 核心流程

| 能力 | 状态 |
|------|:----:|
| `init` 子命令：LD_PRELOAD 捕获构建命令、生成 `.cpp2rust` 中间件 | ✅ |
| `merge` 子命令：将平铺 `<stem>.rs` 合并为 `lib.rs` | ✅ |
| `suggest-aliases` 子命令：从 AST JSON 提取模板别名建议 | ✅ |
| `--dry-run` 模式：执行 AST dump 但不写 `rust/src/` | ✅ |
| `--no-link` / `--header-only` 模式 | ✅ |
| 接口报告自动生成（`init-interface-report.md`） | ✅ |
| `merge --output` 导出自包含 Rust 项目 | ✅ |

### 1.2 已自动提取的 C++ 特性（零人工介入）

自由函数（含重载/命名空间/static 方法）、类实例方法（const/非const/virtual/纯虚/混合类）、构造函数（主构造 + 工厂函数）、运算符重载（全自动 shim + 激活绑定）、枚举、typedef/using 别名（含链式传递性解析）、模板类特化（有别名）、public 单继承 + dynamic_cast 骨架、全局变量/静态成员/实例字段、placement new 骨架、RustAny 建议、多翻译单元 merge（含跨 TU 去重）。

### 1.3 工具引导支持（🔧 骨架已生成，需用户填写业务逻辑）

`std::string`、`std::function`/lambda、函数指针——三类参数均自动生成接口骨架/shim 原型，shim 函数体由用户填写。

### 1.4 hicc 硬限制（无论工具如何改进都无法消除，V1 已做标记和报告）

析构函数显式绑定、多重继承运行时语义、虚继承、方法模板、`auto`/`decltype` 返回类型、友元函数。

---

## 二、V2 开发计划：体验提升 & 场景扩展

**目标**：在不改动 hicc 的前提下，消除用户与工具之间最大的摩擦点，扩大可自动化覆盖范围，提升工程可集成性。

---

### V2.1 构建输入多样化——支持 `compile_commands.json`

**背景**：V1 依赖 `LD_PRELOAD` 拦截编译器，仅在 Linux 上可用，且对 CMake Ninja 后端、Bazel 等构建系统兼容性有限。

**目标**：新增 `--compile-commands <path>` 参数，从 `compile_commands.json` 直接读取编译单元列表和编译参数，绕过 `LD_PRELOAD` 捕获阶段。

**实现范围**：

- `capture.rs`：新增 `from_compile_commands()` 入口，解析 JSON，提取每个编译单元的 `file` 和 `command`/`arguments`，将 include 路径与宏定义转化为 `.opts` 文件
- `main.rs`：`InitArgs` 增加 `--compile-commands` flag；当该 flag 存在时跳过 hook 注入阶段，直接进入 AST dump 步骤
- `layout.rs`：兼容从 compile_commands 模式生成与 LD_PRELOAD 模式等价的 `.cpp2rust` 中间件（内容为 `#include "原始文件"` 形式）
- 输入验证：路径不存在时给出清晰错误提示，含有不支持语言（`.c` + `-x c++` 组合）时给出警告

**用户感知**：

```bash
cpp2rust-demo init --link mylib --compile-commands build/compile_commands.json
```

**验收标准**：现有 RapidJSON 示例可通过 `cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON` 生成 `compile_commands.json` 后用此模式完成绑定，结果与 LD_PRELOAD 模式一致。

---

### V2.2 增量重处理——仅重跑变更翻译单元

**背景**：大型项目有数十上百个翻译单元，每次重跑 `init` 会对所有文件重新执行 clang AST dump，耗时显著。

**目标**：`init` 增量模式下，对未发生变化的翻译单元跳过 AST dump 和代码生成，仅重处理有变化的文件。

**实现范围**：

- `layout.rs`：在 `ast/<stem>.ast.json` 旁记录 `<stem>.checksum`（对输入 `.cpp2rust` + `.opts` 的内容哈希）
- `main.rs`：init 主循环在执行 clang 前比较当前 checksum 与已保存值；若相同且 `<stem>.rs` 已存在，则跳过该文件并在输出中标注 `(cached)`
- `--force` flag：强制全量重跑，忽略 checksum
- `merge` 阶段无需改动（已基于平铺文件扫描，增量后仍然正确）

**验收标准**：对 RapidJSON 8-multi-tu 示例，第二次运行 `init` 在无文件变更时，输出显示所有 TU 均被跳过，耗时接近零。

---

### V2.3 配置文件支持——`.cpp2rust.toml`

**背景**：当前所有参数均通过 CLI flag 传入，重复运行时需手动指定 `--link`、`--clang`、`--extra-clang-args` 等，不便于 CI 集成和团队共享配置。

**目标**：支持项目根目录下的 `.cpp2rust.toml` 配置文件，CLI flag 优先级高于配置文件。

**配置项覆盖范围**：

```toml
[init]
link = "mylib"
clang = "clang-17"
extra_clang_args = "-std=c++17 -Iinclude"
no_link = false

[features.rapidjson]
link = "rapidjson"
no_link = true
extra_clang_args = "-std=c++17 -Iinclude -Ithird_party/rapidjson/include"
```

**实现范围**：

- `main.rs`：在解析 CLI 参数前，尝试从当前目录加载 `.cpp2rust.toml`；用 `serde` 反序列化为配置结构体，与 CLI `Args` merge（CLI 优先）
- 错误处理：配置文件格式错误时给出具体行号提示；未找到配置文件时静默忽略

**验收标准**：在 examples/rapidjson 目录下放置 `.cpp2rust.toml`，运行 `cpp2rust-demo init` 无需指定任何 flag。

---

### V2.4 `validate` 子命令——生成代码可编译性验证

**背景**：`init` 后用户不确定生成的 `<stem>.rs` 是否能通过 `cargo build`，需手动切换到 Rust 项目目录验证。

**目标**：`validate` 子命令在工具侧自动运行 `cargo build`（或 `cargo check`）并将构建输出与接口报告关联展示。

**实现范围**：

- `main.rs`：新增 `validate` 子命令，参数与 `merge` 对齐（`--feature`）
- 执行 `cargo check` in `.cpp2rust/<feature>/rust/`，捕获 stderr
- 将编译错误中的 `src/<stem>.rs:<line>` 与 `<stem>.meta.json` 中的声明列表关联，在输出中定位到具体的 C++ 声明名称
- 生成 `meta/validate-report.md`：包含构建状态（✅ 成功 / ❌ N 个错误）和错误摘要

**验收标准**：在完整 RapidJSON 示例上运行 `validate` 显示 "Build succeeded"；人工破坏一个 `<stem>.rs` 后重跑 `validate` 显示关联到 C++ 声明的错误摘要。

---

### V2.5 接口报告增强——HTML 可视化版本

**背景**：现有接口报告为 Markdown 格式，在大型项目中（如 RapidJSON）表格和 Shim Suggestions 节较长，浏览体验受限。

**目标**：`init` 同时生成 `meta/init-interface-report.html`，提供可折叠/搜索的可视化界面。

**实现范围**：

- `codegen.rs`：新增 `render_interface_report_html()` 函数，与现有 Markdown 渲染并行，内嵌 CSS + 少量 JavaScript（单文件，无外部依赖）
- HTML 结构：顶部统计卡片（✅ 数量、⚠️ 数量、🔧 数量、❌ 数量）→ 可搜索的完整声明表 → 可展开的 Shim Suggestions → 各类别详细节
- `main.rs`：`init` 流程末尾同时写入 `.html` 文件；`--dry-run` 时跳过

**验收标准**：在 RapidJSON 示例上生成的 HTML 文件可在浏览器中打开，搜索框输入 `std::string` 时能过滤出相关跳过项。

---

### V2.6 `std::string` 参数自动 shim 实现生成

**背景**：`std::string` 参数跳过后，工具已生成 `const char*` shim 原型，但函数体需用户手写。对于简单的 getter（返回 `std::string`）和 setter（接受 `std::string`），函数体模式高度固定，可自动生成。

**目标**：对满足以下两种模式的 `std::string` 相关函数，自动补全 shim 函数体：

1. **Getter**：`std::string FooClass::get_bar() const` → 生成 `const char* get_bar_shim(FooClass* self) { static thread_local std::string s; s = self->get_bar(); return s.c_str(); }`
2. **Setter**：`void FooClass::set_bar(const std::string& s)` → 生成 `void set_bar_shim(FooClass* self, const char* s) { self->set_bar(s); }`

**实现范围**：

- `ast.rs`：在 `generate_unsupported_type_shim()` 中增加函数体填充逻辑，识别上述两种模式
- `codegen.rs`：对已有函数体的 shim，在 `operator_shims.hpp` 中直接写出完整函数（不再只是原型 + `// TODO`）
- 不满足模式的函数（参数为 `std::string` 且有业务逻辑的）仍只输出原型 + `// TODO`

**验收标准**：对含有 `std::string` getter/setter 的测试类，生成的 `operator_shims.hpp` 包含可编译的完整 shim 函数体，无需用户修改即可通过 `cargo build`。

---

## 三、V3 开发计划：能力边界突破 & 生态集成

**目标**：突破 V1/V2 的工具边界，向更高自动化程度和更广应用场景延伸。

---

### V3.1 安全包装层自动生成（Safe API Layer）

**背景**：现有工具生成的 hicc 绑定全部为 `unsafe` 代码（FFI 边界），用户直接使用时需自行保证内存安全。对 Rust 新手和安全要求高的项目，通常需要在 `unsafe` FFI 之上手写一层 safe 包装。

**目标**：在 `merge` 阶段可选输出一个 `safe/` 目录，包含基于 FFI 层自动推断的 safe Rust API。

**推断规则**（初始版本覆盖最常见模式）：

| C++ 模式 | 安全包装策略 |
|---------|------------|
| 构造函数 + 析构自动管理 | 生成 `struct Foo(hicc::Class<FooRaw>)` + `impl Drop` |
| `const &self` 方法 | 生成接受 `&self` 的 safe 包装，不加 `unsafe` |
| 枚举参数 | 替换为 Rust `enum` 参数，加范围校验 |
| `*const i8` 返回（字符串） | 包装为 `Option<&str>` 并做空指针检查 |
| `*mut T` / `*const T` 返回 | 包装为 `Option<&'_ T>` / `Option<&'_ mut T>` |

**实现范围**：

- 新建 `src/safegen.rs`：基于 `ExtractedDecls` 推断并生成 safe 包装代码
- `main.rs`：`merge` 增加 `--safe` flag，触发 safe 层生成；输出到 `src.2/safe/`
- 生成的代码遵循 Rust API Guidelines（方法命名、错误类型、`Option` vs `Result`）

**验收标准**：对 RapidJSON 的 `Document`、`Value`、`StringBuffer` 等核心类，`--safe` 生成的包装层通过 `cargo build`，且不暴露任何 `unsafe` 给调用方。

---

### V3.2 libclang 直接集成（替代 clang AST JSON dump）

**背景**：V1 通过 `clang -ast-dump=json` 生成 JSON 再解析，存在以下问题：
- JSON 体积大（RapidJSON 单 TU 约 30 MB+），解析慢
- 部分 AST 信息在 JSON 序列化中丢失（如 implicit instantiation 缺少 `inner`）
- 无法做流式处理，必须全量加载

**目标**：新增 `--use-libclang` 模式，通过 `libclang` C API（`clang_sys` crate）直接遍历 AST，替代 JSON 中间格式。

**实现范围**：

- 新增可选 feature `libclang`（`Cargo.toml`），引入 `clang-sys` crate
- `src/libclang_ast.rs`：实现与 `ast.rs` 相同的 `ExtractedDecls` 输出接口，通过 `CXCursor` 遍历替代 JSON 解析
- `main.rs`：根据 `--use-libclang` flag 选择 AST 提取后端，两条路径共享 `codegen.rs` 不变
- 保持 `ast.rs`（JSON 路径）作为默认路径，`libclang` 路径作为可选加速路径

**验收标准**：`--use-libclang` 模式对所有现有示例（含 RapidJSON 全集）生成结果与 JSON 模式完全一致，AST 处理时间减少 ≥ 50%。

---

### V3.3 双向 FFI 辅助——Rust → C++ 回调场景

**背景**：V1 覆盖的是 "从 Rust 调用 C++ 代码" 方向。实际项目中另一个常见需求是 "C++ 代码调用 Rust 实现的逻辑"（如插件接口、事件回调、自定义 allocator）。hicc 已通过 `@make_proxy` 支持这一方向，但需用户手写 `impl XxxInterface for MyStruct`。

**目标**：对全纯虚类（即 `#[interface]` trait），自动生成 Rust 侧的 impl 骨架文件，减少样板代码。

**实现范围**：

- `codegen.rs`：新增 `render_proxy_impl_skeleton()` 函数，为每个 `#[interface]` trait 生成对应的 `impl XxxInterface for Todo` 骨架（方法签名正确，函数体为 `todo!()`）
- 输出到 `meta/proxy_impls/<interface_name>_impl.rs.template`（扩展名为 `.template` 避免误入编译）
- 接口报告新增 "Proxy Implementation Skeletons" 节，列出生成路径与使用说明
- 文档更新：在 `hicc-usage.md` 中补充 proxy impl 骨架使用流程

**验收标准**：对 RapidJSON `IAllocator` 接口，生成的骨架文件包含正确方法签名，用户只需复制到业务目录并填写方法体即可完成 Rust 实现 C++ allocator 接口。

---

### V3.4 多库合并 & 跨 feature 统一视图

**背景**：实际项目往往依赖多个 C++ 库（如 `libfoo` + `libbar` + `libcommon`），V1 的 feature 概念将它们分隔在不同目录，无法统一引用。

**目标**：新增 `merge --features foo,bar,common --output unified/` 命令，将多个 feature 的 FFI 内容合并为一个 Rust crate。

**实现范围**：

- `merge.rs`：扩展 `merge_grouped_modules()` 支持多 feature 输入列表；去重逻辑覆盖跨 feature 的同名类型（以第一次出现为准，并在报告中标注冲突）
- `layout.rs`：`merge --features` 模式下，从多个 `.cpp2rust/<feature>/` 读取 `src.1/`（或 `src/`），汇聚到统一输出目录
- `build.rs` 生成：统一视图的 `build.rs` 合并所有 feature 的 include 路径和链接名称
- 冲突检测：同名类出现在多个 feature 时，在 merge 报告中标注，让用户决定保留哪个

**验收标准**：创建两个示例 feature（`foo` 和 `bar`），各含一个类和若干函数，`merge --features foo,bar` 生成的 `lib.rs` 包含两个 feature 的全部内容且 `cargo build` 通过。

---

### V3.5 IDE 集成——VSCode 扩展（MVP）

**背景**：`cpp2rust-demo` 目前是纯 CLI 工具，对 IDE 用户（尤其是团队新成员）入门门槛偏高。

**目标**：发布 VSCode 扩展（独立仓库），提供以下 MVP 功能：

1. **一键 init**：右键工作区目录 → "Run cpp2rust-demo init"，弹出 feature 名称输入框，后台执行并实时展示日志
2. **接口报告预览**：检测 `meta/init-interface-report.md` 并在侧边栏提供 WebView 预览，自动刷新
3. **跳过声明悬停提示**：在 `.rs` 文件中，对 `// [SKIPPED]` 注释行提供 Hover 解释（跳过原因 + 对应 shim 建议）

**实现范围**（cpp2rust-demo 工具侧配合项）：

- `codegen.rs`：在生成的 `<stem>.rs` 中，跳过声明旁的注释统一格式化为机器可读的 `// [SKIPPED reason=hicc_limitation decl="FooClass::bar()"]`，供扩展解析
- 接口报告格式版本化：在 `init-interface-report.md` 第一行写入 `<!-- cpp2rust-report-version: 2 -->`，便于扩展检测格式兼容性
- 接口报告同时生成 `meta/init-interface-report.json`（结构化数据），供扩展直接消费，不依赖 Markdown 解析

**验收标准**：VSCode 扩展在 RapidJSON 示例目录可成功触发 init 并展示接口报告 WebView，跳过声明悬停提示内容正确。

---

## 四、优先级与依赖关系

```
V2.1 compile_commands 支持
V2.2 增量重处理          ← 依赖 V2.1（compile_commands 模式更易做增量）
V2.3 配置文件            ← 独立，可先行
V2.4 validate 子命令     ← 独立，可先行
V2.5 HTML 接口报告       ← 依赖 V2.4（validate 状态写入报告）
V2.6 std::string 自动 shim ← 独立

V3.1 safe 包装层         ← 依赖 V2 整体完成，输出结构稳定后再做
V3.2 libclang 集成       ← 独立，但较大，建议在 V2 完成后启动
V3.3 双向 FFI 骨架       ← 依赖 V1 proxy 机制（已完成），可并行于 V2
V3.4 多库合并            ← 依赖 V2.1 + V2.3（多 feature 配置）
V3.5 VSCode 扩展         ← 依赖 V3 工具侧接口报告 JSON（V3.5 工具侧改动先做）
```

---

## 五、不在计划范围内的内容

以下内容因 hicc 上游硬限制或超出工具定位，**不**纳入 V2/V3 计划：

| 内容 | 原因 |
|------|------|
| 多重继承运行时语义 | hicc 本身不支持，需上游改动 |
| 虚继承（菱形继承）运行时语义 | 同上 |
| 方法模板 Rust 泛型映射 | hicc 无法表达泛型方法，需上游改动 |
| `auto`/`decltype` 返回类型自动推断 | hicc 签名无法表达，需手写包装函数 |
| 完整 C++ 语义翻译（如 SFINAE、constexpr 计算） | 超出工具定位（工具是脚手架生成器，不是语义翻译器） |
| Windows 原生支持（非 WSL） | 依赖 `LD_PRELOAD`，V2.1 的 compile_commands 模式部分缓解 |

---

## 六、验收与质量门禁（全版本通用）

每个 V2/V3 子项完成后，必须满足：

1. **现有测试不回归**：`cargo test` 全量通过
2. **现有示例不破坏**：`examples/` 下所有示例（simple、class、features、rapidjson、semi-auto、conditional、guided）的生成结果与 V1 一致（或有明确 changelog 说明变化）
3. **接口报告正确性**：生成的 `init-interface-report.md` 中特性统计数字（✅/⚠️/🔧/❌ 数量）与实际代码行为匹配
4. **文档同步更新**：影响到用户可见行为的改动，同步更新 `docs/design.md`、`docs/cpp-features.md` 或 `docs/特性支持全景图.md`
