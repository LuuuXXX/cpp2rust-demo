# C++ 到 Rust Safe FFI 自动化工具 - 方案 v7

> 在 v6（模板类 / 模板函数 / `@make_proxy` / `@dynamic_cast` / 冒烟测试，全部由环境变量开关
> 控制、默认关闭）的基础上，v7 的主线是 **「把高级能力从灰度开关毕业为默认行为」**：
> 让 `init` 在不引入任何环境变量开关的前提下，默认就生成全部 C++ 特性对应的、
> 符合 hicc 最佳实践的 Rust FFI，并默认生成覆盖全部特性、符合 Rust 最佳实践的冒烟测试。
>
> **硬约束**：
> 1. 不改变现有使用方式（`init` + `merge` 两个命令、参数、`.cpp2rust/<feature>/` 目录结构）。
> 2. **不新增、不依赖任何环境变量开关**来影响用户的生成结果——直接改为默认处理。
> 3. 全程简体中文。

---

## 1. 背景与问题定位

### 1.1 v6 已经完成的工作

通过通读 `docs/plans/v6/automated-cpp2rust-ffi-v6.md` 及其 14 篇 `development-progress-*.md`、
当前 `src/` 源码与 `examples/001–048`，确认 v6 已交付：

- **Phase A**：AST 层结构化提取模板类（`ClassTemplate`）与模板函数（`FunctionTemplate`），
  落地 `CppAst.template_classes / template_functions`（`src/ast_parser/`）。
- **Phase B**：泛型 `import_class!` / `import_lib!` 骨架 + 5 种来源的实例化别名与构造工厂
  （`src/extractor/template_spec.rs`、`src/generator/hicc_codegen.rs`）。
- **Phase C**：`@make_proxy` 代理工厂（`src/extractor/proxy_spec.rs`）、跨层 `@dynamic_cast`
  下行转换（裸指针 + 引用形式，`src/extractor/dynamic_cast_spec.rs`）。
- **Phase D**：冒烟测试生成器 `src/generator/smoke_test_gen.rs`，在 `init` 阶段幂等生成
  `.cpp2rust/<feature>/rust/tests/smoke.rs`。
- **Phase E**：14 个示例（015–018、023–027、034–038）迁移为 `lib.rs + main.rs + smoke.rs`。
- **Phase F**：CI 新增 `l-smoke`（冒烟）与 `gen-verify`（工具实际输出可编译）两个 job。
- **Phase G**：`docs/INTRODUCTION.md` Part 3.6、`docs/references/hicc.md`、各示例 README 对齐。

### 1.2 当前的核心缺口（本次优化要解决的）

| 缺口 | 当前代码现状 | 后果 |
|------|------------|------|
| **高级能力被环境变量「锁死」在默认关闭** | `src/generator/hicc_codegen.rs` 定义 `GEN_TEMPLATES_ENV` / `GEN_PROXY_ENV` / `GEN_DYNAMIC_CAST_ENV` 三个开关，`templates_enabled()` / `proxy_enabled()` / `dynamic_cast_enabled()` 默认返回 `false` | 用户执行 `init` 时，模板泛型骨架、`@make_proxy`、`@dynamic_cast` **默认不输出**，必须自行设置环境变量才能获得 v6 的全部能力——与用户「不要额外开关、直接默认处理」的诉求冲突 |
| **冒烟测试也受开关影响、且只生成类型可用性断言** | `src/generator/smoke_test_gen.rs` 由 `GEN_SMOKE_ENV` 控制（默认开但可被关），生成的 `smoke.rs` 仅做编译期类型可用性断言（`pub class` 占位），私有工厂以 `cpp2rust-todo[SMOKE]` 占位 | 冒烟测试是「能编译」而非「能验证行为」；且仍是一个可被关闭的开关 |
| **冒烟测试未覆盖全部特性** | 仅 14/48 示例迁移出 `tests/smoke.rs`；`init` 生成的冒烟测试也只对 `pub class` 做类型断言，运算符 / 模板函数 / 异常 / 智能指针等多数特性无行为级冒烟覆盖 | 「生成即验证」闭环不完整，无法证明每类特性的 FFI 真的可用 |
| **开关相关代码冗余** | `ffi_model.rs`、`extractor/*`、`hicc_codegen.rs`、`init.rs`、各 `*_gen_tests.rs` 中遍布「始终构建 IR，但仅在开关开启时输出」的注释与分支；测试需在单一 `#[test]` 内串行设置/清理进程级环境变量 | 双路径（开/关）增加维护成本、测试脆弱、注释噪声大 |
| **黄金基线仍以「开关关闭的默认产物」为准** | L1 黄金（52 项）与 lib 单测以「默认产物逐字节不变」为约束 | 一旦默认输出高级能力，所有相关黄金需重做；需要分批迁移避免回归 |

> **结论**：v6 出于「默认产物逐字节不变」的安全考量，用环境变量做灰度通道，能力是齐的但默认不开。
> v7 的任务不是「再加功能」，而是 **「让既有能力默认生效、删除开关、补齐冒烟覆盖、重做黄金基线」**，
> 同时借机做一轮代码与测试的去冗余。

### 1.3 v7 的定位

- **使用方式零变更**：`init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构均不变。
- **零环境变量开关**：删除 `CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` /
  `CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE`，相关能力改为「默认生成」。
- **行为可预期**：同样的 C++ 输入，`init` 的产物是确定的、覆盖全特性的；不再因环境而异。
- **生成即验证**：冒烟测试默认生成、覆盖全部特性、做行为级断言、符合 Rust 测试最佳实践。

---

## 2. C++ 特性 → hicc 最佳实践覆盖核对（对应需求 1）

下表把 `examples/001–048` 覆盖的 C++ 特性与「v7 默认生成策略」「是否符合 hicc 最佳实践」逐项核对。
✅ = 默认全自动；⚙️ = v6 已具备能力但默认关闭，v7 改为默认开启；⚠️ = 保留降级 + `cpp2rust-todo[TAG]` 兜底。

| 分类 | 特性（示例） | v6 默认 | v7 默认策略 | hicc 最佳实践对标 |
|------|------|------|------|------|
| 基础函数 | 001–004 普通/重载/默认参/inline | ✅ | ✅ 不变 | `import_lib!` / `hicc::cpp!` 内联 |
| 基础函数 | 005 C 可变参数 `...` | ⚠️`[CV]` | ⚠️`[CV]` 保留（FFI 无法表达 va_list） | 头文件固定参 wrapper → `import_lib!` |
| 类与对象 | 006–011 类/构造/拷贝/移动/静态/const | ✅ | ✅ 不变 | opaque ptr + `import_class!` + 必要 shim |
| 类与对象 | 012 volatile 成员 | ⚠️`[VM]` | ⚠️`[VM]` 保留 | `volatile T*` C shim → `import_lib!` |
| 面向对象 | 013–014 单/多继承 | ✅ | ✅ 不变 | 基类方法提升进子类 `import_class!` |
| 面向对象 | 015–018 虚函数/纯虚/override/菱形 | ✅(+⚙️proxy) | ✅ + **默认输出 `@make_proxy`**（抽象类→`#[interface]`，Rust 可实现 C++ 接口） | `#[interface]` + `@make_proxy` + `hicc::Interface<T>` |
| 运算符/类型 | 019 运算符重载 | ⚠️`[OP]` | ⚠️`[OP]` 保留命名 shim + 默认补 `impl ops::*` 骨架（见 §6） | 命名 shim → `hicc::cpp!`/`import_lib!` |
| 运算符/类型 | 020–022 友元/explicit/mutable | ✅ | ✅ 不变 | 透传 / 普通函数 |
| 运算符/类型 | 023 typeid/RTTI | ✅(+⚙️dynamic_cast) | ✅ + **默认输出 `@dynamic_cast`** 下行转换（替代整数枚举绕过） | `@dynamic_cast`（裸指针 + 引用形式） |
| 模板实例化 | 024 函数模板 | ⚙️ | ✅ **默认输出**模板函数 `#[cpp(func="ret name<T>(...)")]` 骨架 + 实例化绑定 | 模板函数 → `import_lib!` |
| 模板实例化 | 025 类模板 | ⚙️ | ✅ **默认输出**泛型 `class Name<T>` + 实例化别名/工厂 | 模板类 → `import_class!` 泛型 |
| 模板实例化 | 026/027 偏特化/显式实例化 | ⚙️ | ✅ **默认输出**实例化别名 + 工厂 | 泛型 class + 具体别名 |
| 模板实例化 | 028 可变参数模板 | ⚠️`[VA]` | ⚠️`[VA]` 保留按元数展开 wrapper | wrapper 静态方法 |
| 智能指针/内存 | 029–033 unique/shared/deleter/placement/RAII | ✅ | ✅ 不变 | opaque ptr + 计数/删除 shim |
| STL 容器 | 034–038 vector/map/string/array/tuple | ✅ | ✅ 不变（薄 wrapper → `import_class!`；后续可对标 `hicc-std`） | wrapper class → `import_class!` |
| 函数对象 | 039/040 lambda/std::function | ⚠️`[LM]`/`[FP]` | ⚠️ 保留 class wrapper + 函数指针映射 | class wrapper + `unsafe extern "C" fn` |
| 函数对象 | 041 std::bind | ✅ | ✅ 不变 | 函数对象 → `import_class!` |
| 函数对象 | 042 异常 | ✅ | ✅ 不变（shim try/catch → 错误码）；评估 `hicc::Exception<T>` 直通 | `hicc::Exception<T>` / shim |
| 高级特性 | 043–048 命名空间/enum class/union/constexpr/noexcept/综合 | ✅ | ✅ 不变 | 扁平化前缀 / `const` / opaque ptr |

**v7 覆盖结论**：除 5 个本质无法被 C ABI / FFI 表达的降级特性（`[CV]` `[VM]` `[OP]` `[VA]` `[LM]/[FP]`，
均已有命名 shim + `cpp2rust-todo` 兜底），其余特性 v7 **默认全自动生成符合 hicc 最佳实践的绑定**；
v6 中三类被开关锁住的高级能力（模板、`@make_proxy`、`@dynamic_cast`）在 v7 **默认开启**。

> 降级特性的「不可自动化」属于 C ABI / FFI 的固有边界（详见 README「降级特性详解」），
> v7 不承诺消除，但要求：(a) 命名 shim 默认生成；(b) `cpp2rust-todo[TAG]` 注释精确定位；
> (c) 冒烟测试对其可调用部分仍做行为断言（见 §4）。

---

## 3. 移除环境变量开关，改为默认处理（对应需求 3，v7 主线）

### 3.1 移除清单

| 环境变量 / 函数 | 位置 | v7 处理 |
|------|------|------|
| `GEN_TEMPLATES_ENV` / `templates_enabled()` | `src/generator/hicc_codegen.rs` | 删除；模板骨架与实例化别名/工厂 **无条件输出** |
| `GEN_PROXY_ENV` / `proxy_enabled()` | `src/generator/hicc_codegen.rs` | 删除；`@make_proxy` 工厂 **无条件输出** |
| `GEN_DYNAMIC_CAST_ENV` / `dynamic_cast_enabled()` | `src/generator/hicc_codegen.rs` | 删除；`@dynamic_cast` 下行转换 **无条件输出** |
| `GEN_SMOKE_ENV` / `smoke_enabled()` | `src/generator/smoke_test_gen.rs`、`src/commands/init.rs` | 删除；冒烟测试 **无条件生成**（保留幂等：已存在用户改动则不覆盖） |
| `env_switch_enabled()` 辅助 | `src/generator/hicc_codegen.rs` | 删除（无其他调用者后） |

### 3.2 改造后的生成路径

- `extractor` 侧已「始终构建」模板/proxy/dynamic_cast 的 IR（开销极小），v7 **删除生成器侧的
  开关裁决分支**，IR 一旦非空即输出对应骨架。代码从「双路径（开/关）」简化为「单路径」。
- `init.rs` 删除 `if smoke_test_gen::smoke_enabled()` 判断，改为始终调用
  `project_generator::write_smoke_test`（幂等：目标文件已被用户修改则跳过，避免覆盖）。
- 删除 `ffi_model.rs` 中所有「仅在 `CPP2RUST_GEN_*` 开启时输出」的注释，替换为「默认输出」语义说明。

### 3.3 幂等与可重入（替代「开关」的安全网）

用户诉求是「不要用开关影响使用」，但仍需避免 `init` 覆盖用户手改的冒烟测试：

- **幂等生成**：`tests/smoke.rs` 若不存在则生成；若已存在则比对「是否仍为工具生成的原样」，
  仅在未被手改时才更新（沿用 v6 幂等策略，但去掉环境变量入口）。
- 这是「文件级幂等」而非「环境变量开关」，不改变命令签名，符合硬约束。

---

## 4. 冒烟测试：默认生成、全特性覆盖、Rust 最佳实践（对应需求 2）

### 4.1 现状与目标差距

- 现状：`smoke_test_gen.rs` 只为 `pub class` 生成编译期类型可用性断言，私有工厂以
  `cpp2rust-todo[SMOKE]` 占位；只有 14 个迁移示例有「手写」的行为级 `tests/smoke.rs`。
- 目标：`init` 默认生成的 `tests/smoke.rs` 要 **覆盖 FfiSpec 中全部可安全调用的接口**，
  并尽量做 **行为级断言**，而非仅类型可用。

### 4.2 按特性类别的冒烟生成策略

| FFI 元素 | 冒烟断言策略（Rust 侧最佳实践） |
|------|------|
| 独立函数 / 友元函数 / 命名空间函数 | 以确定性入参调用，`assert_eq!` 返回值（数值/布尔/字符串可断言者） |
| 类：构造 + getter | 工厂构造 → 调用 const getter → `assert_eq!` 已知初值 |
| 类：构造 + setter/getter 往返 | set 已知值 → get → `assert_eq!`，验证状态可读写 |
| 静态成员 / 全局变量 / constexpr | 读取 → `assert_eq!` 编译期已知值 |
| 运算符命名 shim（`[OP]`） | 构造两实例 → 调用 `{class}_add` 等 → 断言结果 |
| 模板函数实例化（024） | 对具体实例（如 `do_swap<int>`）构造入参 → 调用 → 断言交换生效 |
| 模板类实例化别名（025–027） | 用别名/工厂构造 → 调用成员 → 断言 |
| 虚函数 / `@make_proxy`（015–018） | 通过基类指针调用虚方法断言多态；proxy 工厂构造 → 调用 |
| `@dynamic_cast`（023） | 上行构造派生 → 下行 `dynamic_cast` → 断言非空/类型正确 |
| 异常（042） | 触发异常路径 → 断言错误码/消息；正常路径断言成功 |
| 智能指针 / RAII（029–033） | 构造 → 使用 → 作用域结束（`Drop`）不 panic |
| 无法自动断言者 | 退化为「构造 + 调用不 panic」+ `// cpp2rust-todo[SMOKE]` 提示补断言（最小化占位比例） |

### 4.3 Rust 测试最佳实践要求

- 使用 Cargo 集成测试约定：`tests/smoke.rs`，每个特性一个 `#[test] fn smoke_*()`。
- 测试名表意（`smoke_<feature>_<behavior>`），失败信息可读（`assert_eq!` 带上下文）。
- 不使用全局可变状态；`unsafe` 块最小化并集中在 FFI 调用点。
- 默认 `cargo test` 可运行（链接 C++ 由生成的 `build.rs` 负责），无需额外环境变量。
- 平台差异（如某些虚函数示例在 macOS）用 `#[cfg]` / `#[ignore]` 标注并在注释说明，对标 v6 L3 先例。

### 4.4 覆盖全部特性

- `init` 生成的冒烟测试覆盖该 feature 内 FfiSpec 的全部可调用接口。
- examples 侧：把剩余未迁移示例（除 14 个已迁移）按需补 `tests/smoke.rs`，使
  **48 个示例全部具备行为级冒烟测试**（分批，先做高价值/此前为开关锁住的特性）。

---

## 5. 单元测试 / 集成测试 / CI（对应需求 4）

### 5.1 单元测试

- 删除开关后，重写 `tests/template_gen_tests.rs` / `tests/proxy_gen_tests.rs` /
  `tests/dynamic_cast_gen_tests.rs`：去掉「设置/清理环境变量」的串行约束，直接断言
  **默认产物即包含**对应骨架（测试更简单、可并行）。
- `smoke_test_gen` 单测：去掉 `smoke_enabled()` 相关用例，新增「按特性类别生成正确断言骨架」的用例。
- 既有 lib 单测（约 265）随 IR 注释/分支精简而更新。

### 5.2 集成测试

- L1 黄金（`tests/l1_golden_tests.rs`）：**重做黄金基线**，使其包含默认输出的模板/proxy/dynamic_cast
  骨架与新冒烟测试；这是 v7 最大的一次性回归面，需分批、逐示例核对。
- L2 编译 / L3 运行 / L4 merge / L5 nm：保持层次，更新受默认产物变化影响的断言。
- `tests/gen_verify_e2e_test.rs`：扩展覆盖到模板/proxy/dynamic_cast 默认产物可编译。
- L_smoke：从 14 个迁移示例扩展到全部具备 `tests/smoke.rs` 的示例。

### 5.3 CI（`.github/workflows/ci.yml`）

- 现有 jobs（build / unit-tests / l1-golden / l2-compile / l3-run / l-smoke / gen-verify，
  含 Windows MinGW / MSVC 矩阵）保留。
- `l-smoke` job 的示例清单从 14 扩到全量（或自动发现含 `tests/smoke.rs` 的示例）。
- `gen-verify` job 覆盖三类默认高级能力（模板/proxy/dynamic_cast）各 ≥1 示例。
- 保持 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 三道门禁。
- 平台限制项用 `#[cfg]` / `#[ignore]` 跳过并在 PR 描述记录。

### 5.4 验收标准

1. 不设置任何环境变量，`init` 默认产物即包含模板/proxy/dynamic_cast 骨架与全特性冒烟测试。
2. 代码库中不再存在 `CPP2RUST_GEN_*` 字样（注释与代码均清理）。
3. L1–L6 + L_smoke 全绿；48 示例均可 `cargo test --test smoke`（平台跳过项除外）。
4. `init` / `merge` 命令、参数、输出目录结构与 v6 完全一致。

---

## 6. 代码优化与去冗余（对应需求 6）

| 优化点 | 说明 |
|------|------|
| 删除开关基础设施 | 移除 `env_switch_enabled` 及 4 个 `*_enabled()`、4 个 `*_ENV` 常量，消除「开/关」双路径分支 |
| 注释去噪 | 清理 `ffi_model.rs` / `extractor/*` / `ast_parser/*` 中数十处「仅在 `CPP2RUST_GEN_*` 开启时…」注释 |
| 生成器收敛 | `hicc_codegen.rs` 中模板/proxy/dynamic_cast 输出统一为「IR 非空即输出」，emit_* 函数签名去掉 `enabled` 形参 |
| 测试简化 | gen 测试去掉进程级环境变量串行化，改为直接断言默认产物，提升可并行性与稳定性 |
| 运算符骨架增强（可选） | 评估为 `[OP]` 默认追加 `impl std::ops::*` 骨架（带 `cpp2rust-todo`），减少用户手工量 |
| 冒烟生成器重构 | 将「按 FfiSpec 元素类别 → 断言模板」抽象为表驱动，降低分支复杂度并便于扩展 |

> 去冗余以「行为等价 + 测试护航」为前提，分阶段小步提交，避免一次性大改导致回归。

---

## 7. 文档（对应需求 5）

| 文档 | v7 更新 |
|------|------|
| `docs/plans/v7/automated-cpp2rust-ffi-v7.md` | 本方案（目标 / 方案 / 计划） |
| `docs/plans/v7/development-progress.md` | 进展跟踪骨架（阶段表 + 状态），随开发持续更新；每阶段可追加 `development-progress-phase-*.md` |
| `docs/INTRODUCTION.md` | 移除「环境变量开关」相关说明（Part 3.6 的开关表）；改述为「默认生成的高级映射能力」；冒烟测试章节更新为「默认全特性行为级覆盖」 |
| `docs/references/hicc.md` | 「v6 新增能力速查」更新为「默认能力」，去掉启用方式中的环境变量 |
| `README.md` | 特性矩阵：模板/虚函数/RTTI 标注更新为「默认原生 hicc 映射」；删除环境变量开关文档；冒烟测试小节更新 |
| `CHANGELOG.md` | 记录「移除 `CPP2RUST_GEN_*` 开关、默认生成全特性 + 冒烟测试」的破坏性/行为变更 |
| `DEVELOPMENT.md` | 移除环境变量开关说明，补充 v7 默认行为与黄金基线重做指引 |

> 全部文档使用简体中文，风格与现有 `INTRODUCTION.md` 一致。

---

## 8. 实现阶段划分

> 测试驱动：每个 Phase 完成的标准是「相关测试全绿且默认产物符合预期」。
> 风险最高的是「黄金基线重做」，因此把开关移除与黄金更新绑定、分批推进。

| 阶段 | 内容 | 依赖 | 产出 |
|------|------|------|------|
| **Phase 1** | 移除 `CPP2RUST_GEN_SMOKE`，冒烟测试默认幂等生成；冒烟生成器表驱动重构 + 行为级断言（先覆盖已迁移特性） | 无 | 默认冒烟测试 + 单测 |
| **Phase 2** | 移除 `CPP2RUST_GEN_DYNAMIC_CAST`，`@dynamic_cast` 默认输出；重做相关黄金 + gen 测试 | 1 | 默认 dynamic_cast + 黄金 |
| **Phase 3** | 移除 `CPP2RUST_GEN_PROXY`，`@make_proxy` 默认输出；重做相关黄金 + gen 测试 | 1 | 默认 proxy + 黄金 |
| **Phase 4** | 移除 `CPP2RUST_GEN_TEMPLATES`，模板骨架/实例化别名/工厂默认输出；重做相关黄金 + gen 测试 | 1 | 默认模板 + 黄金 |
| **Phase 5** | 冒烟测试全特性覆盖：为剩余示例补 `tests/smoke.rs`，达成 48/48 行为级覆盖 | 1 | 全特性冒烟 + L_smoke 扩展 |
| **Phase 6** | 代码去冗余收尾：删除 `env_switch_enabled` 等遗留、清理注释、emit_* 去 `enabled` 形参 | 2–4 | 单路径生成器 |
| **Phase 7** | CI：扩展 l-smoke 全量、gen-verify 覆盖三类高级能力；门禁校验「无 `CPP2RUST_GEN_*`」 | 2–6 | 绿色 CI |
| **Phase 8** | 文档：INTRODUCTION / hicc.md / README / CHANGELOG / DEVELOPMENT 全面对齐 v7 | 1–7 | 文档对齐 |

> 每个移除开关的阶段（2/3/4）都是「删开关 → 默认输出 → 重做该特性黄金 → 更新 gen 测试」的垂直切片，
> 可独立 PR、独立验证，最大限度降低黄金回归风险。

---

## 9. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 黄金基线一次性重做面大 | L1 大量回归 | 按特性垂直切片（Phase 2/3/4 各管一类），逐示例核对 diff；保留旧黄金作对照 |
| 默认输出高级骨架后，部分骨架含 `cpp2rust-todo[TMPL]` 等占位 | 用户误以为「未完成」 | 占位即 v6 既有降级语义，文档明确「骨架默认生成、复杂依赖类型需补全」；冒烟测试对可调用部分仍断言 |
| 删除冒烟开关后 `init` 总是写 `tests/smoke.rs` | 覆盖用户手改 | 文件级幂等：仅当文件不存在或仍为工具原样时才生成/更新 |
| `@make_proxy` / `@dynamic_cast` 平台差异 | 某平台运行崩溃 | 沿用 L3/L_smoke 的 `#[cfg]` / `#[ignore]` 平台跳过策略并记录 |
| 多 feature 集成测试（`multi_feature_e2e`）默认产物变化 | 回归 | 同步更新断言；在 Phase 2–4 各自的 PR 内修正 |
| 移除开关属行为变更（对依赖开关的现有用户） | 兼容性 | `CHANGELOG.md` 明确标注；由于命令/参数不变，仅产物更全，影响面可控 |

---

## 10. 不做的事（范围边界）

- 不改变 `init` / `merge` 两命令的使用方式与参数。
- 不引入任何新的环境变量开关或命令行开关来影响生成结果（与需求 3 一致）。
- 不追求 100% 消除降级特性（`[OP]` `[VA]` `[LM]` `[CV]` `[VM]` `[FP]` 仍保留命名 shim + 兜底注释）。
- 不生成业务语义等价的 Rust 实现，只生成 FFI 绑定与行为级冒烟验证。
- 不引入与 hicc 无关的新依赖。
