# C++ 到 Rust Safe FFI 自动化工具 - v7 开发进度跟踪

> 本文件是 `docs/plans/v7/automated-cpp2rust-ffi-v7.md`（v7 方案）的配套进度跟踪文档。
> 它把 **整体目标、详细计划、已完成部分、未完成部分、后续详细实施方案** 全部文档化，
> 以便下一个 PR 一次性推进剩余工作。
>
> **快照时间**：截至当前分支 `copilot/optimize-cpp2rust-demo-yet-again`。
> **状态图例**：✅ 已完成 ｜ 🟡 部分完成 ｜ ⬜ 未开始

---

## 1. 整体目标（回顾）

在 v6（模板类 / 模板函数 / `@make_proxy` / `@dynamic_cast` / 冒烟测试，全部由 `CPP2RUST_GEN_*`
环境变量开关控制、默认关闭）的基础上，v7 的主线是 **「把高级能力从灰度开关毕业为默认行为」**：

- **使用方式零变更**：`init` / `merge` 两命令、参数、`.cpp2rust/<feature>/` 目录结构均不变。
- **零环境变量开关**：删除 `CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` /
  `CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE`，相关能力改为「默认生成」。
- **行为可预期**：同样的 C++ 输入，`init` 的产物确定、覆盖全特性、不因环境而异。
- **生成即验证**：冒烟测试默认生成、覆盖全部特性、做行为级断言、符合 Rust 测试最佳实践。

完整目标、约束与特性核对见 `automated-cpp2rust-ffi-v7.md`（§1–§2、§10）。

---

## 2. 阶段进度总览

| 阶段 | 内容 | 状态 | 说明 |
|------|------|------|------|
| **Phase 1** | 移除 `CPP2RUST_GEN_SMOKE`，冒烟测试默认幂等生成；冒烟生成器表驱动重构 + 行为级断言 | 🟡 部分完成 | 开关已移除、默认幂等生成已落地；**生成器仍是「类型断言 + 零参调用 + 占位」策略，尚未做表驱动 + 行为级断言重构** |
| **Phase 2** | 移除 `CPP2RUST_GEN_DYNAMIC_CAST`，`@dynamic_cast` 默认输出；重做黄金 + gen 测试 | ✅ 已完成 | 默认活动绑定输出，黄金与 gen 测试已重做 |
| **Phase 3** | 移除 `CPP2RUST_GEN_PROXY`，`@make_proxy` 默认输出；重做黄金 + gen 测试 | ✅ 已完成 | 默认活动绑定输出，黄金与 gen 测试已重做 |
| **Phase 4** | 移除 `CPP2RUST_GEN_TEMPLATES`，模板骨架/别名/工厂默认输出；重做黄金 + gen 测试 | ✅ 已完成 | 默认以注释骨架输出（保证可编译），黄金与 gen 测试已重做 |
| **Phase 5** | 冒烟测试全特性覆盖：为剩余示例补 `tests/smoke.rs`，48/48 行为级覆盖 | ✅ 已完成 | 48/48 示例均有**手写**行为级 `tests/smoke.rs`（CI `l-smoke` 自动发现全量） |
| **Phase 6** | 代码去冗余：删除 `env_switch_enabled` 等遗留、清理注释、`emit_*` 去 `enabled` 形参 | 🟡 大部分完成 | 开关基础设施与双路径分支已删除；**§6 可选项「`impl std::ops::*` 运算符骨架增强」未做**；emit_* 仍残留少量 `active`/历史形参（见 §4.6） |
| **Phase 7** | CI：l-smoke 全量、gen-verify 覆盖三类高级能力；门禁校验无 `CPP2RUST_GEN_*` | ✅ 已完成 | `l-smoke` 自动发现 48 例，gen-verify 覆盖模板/proxy/dynamic_cast，门禁已加 |
| **Phase 8** | 文档对齐：INTRODUCTION / hicc.md / README / CHANGELOG / DEVELOPMENT | 🟡 大部分完成 | 主文档均已对齐「默认生成、无开关」；本进度文档（development-progress.md）此前缺失，现补齐 |

> **一句话结论**：v7 的「移开关 + 默认输出 + 重做黄金 + CI 门禁 + 文档对齐」主体已完成；
> **真正剩余的核心工作只有一项** —— 把冒烟测试**生成器**升级为方案 §4.2 的「表驱动 + 行为级断言」，
> 让 `init` 对**新项目**生成的 `tests/smoke.rs` 也具备示例库已有的行为级断言质量。

---

## 3. 已完成部分（含代码证据）

### 3.1 环境变量开关全部移除（Phase 1/2/3/4/6）✅

- `src/` 与 `tests/` 中 **0 处** `CPP2RUST_GEN_*`、`*_enabled()`、`env_switch_enabled` 字样。
- 生成路径由「开/关双路径」收敛为「IR 非空即输出」单路径：
  - `src/generator/hicc_codegen.rs`：模板/proxy/dynamic_cast 直接遍历 IR 集合输出，
    `emit_proxy_factory` / `emit_template_class` 等签名不再含 `enabled` 形参。
  - `src/commands/init.rs`：无条件调用 `smoke_test_gen::generate_smoke_test` +
    `project_generator::write_smoke_test`。
- CHANGELOG `[Unreleased]` 已记录破坏性行为变更。

### 3.2 高级能力默认输出（Phase 2/3/4）✅

- `@dynamic_cast` 下行转换（裸指针 + 引用形式）：默认活动绑定输出。
- `@make_proxy` 代理工厂：默认活动绑定输出（抽象类 → `#[interface]`）。
- 模板类 / 函数 / 实例化别名 / 工厂：默认以**注释骨架**（带 `cpp2rust-todo[TMPL]` 指引）输出，
  因「未实例化模板无可链接符号、泛型 `<T>` 不可直接编译」，注释化以保证 L6 gen-verify 默认产物可编译。

### 3.3 黄金基线与 gen 测试重做（Phase 2/3/4）✅

- `tests/{template,proxy,dynamic_cast}_gen_tests.rs` 已去掉 `set_var`/`remove_var` 串行化，
  改为直接断言默认产物；新增「模板骨架须为注释行」契约断言。
- `tests/l1_golden_tests.rs` 新增 `golden_test_scaffold!` 宏；024 模板函数黄金
  `lib_scaffold.rs` 更新为注释骨架形式。

### 3.4 示例冒烟测试 48/48 覆盖（Phase 5）✅

- `examples/001–048/rust_hicc/tests/smoke.rs` 全部存在，且为**手写行为级断言**
  （`assert_eq!` / `assert!` 验证返回值、状态往返、多态、模板实例化、dynamic_cast 等）。
- 抽查：019 运算符、023 RTTI、024 模板函数、016 纯虚/proxy 均为行为级断言，无 `cpp2rust-todo[SMOKE]` 占位。

### 3.5 CI 扩展与门禁（Phase 7）✅

- `.github/workflows/ci.yml`：
  - `l-smoke` job 自动发现 `examples/*/rust_hicc/tests/smoke.rs` 并逐个 `cargo test --test smoke`，
    汇总 `passed/total`（48/48）。
  - `gen-verify`（L6）覆盖模板函数/模板类/虚函数/纯虚 proxy/RTTI dynamic_cast 等，全 48 示例。
  - 新增门禁 step：`grep -r "CPP2RUST_GEN_"` 命中即 `exit 1`。
  - 保留 `cargo fmt --check` / `cargo clippy` / `cargo test` 三道门禁与多平台矩阵。

### 3.6 文档对齐（Phase 8）✅（除本进度文档外）

- `docs/INTRODUCTION.md`、`README.md`、`DEVELOPMENT.md`、`docs/references/hicc.md`、`CHANGELOG.md`
  均已改述为「默认生成、无环境变量开关」。
- 本次补齐 `docs/plans/v7/development-progress.md`（即本文件）。

---

## 4. 未完成部分（下一个 PR 的工作清单）

### 4.1 【核心】冒烟测试生成器升级为表驱动 + 行为级断言（Phase 1 剩余）🟡

**现状**：`src/generator/smoke_test_gen.rs` 当前策略（§4.1 现状，非 §4.2 目标）：

- A. 对所有 `pub class` 生成编译期类型可用性断言 `assert_type_available::<T>()`；
- B. 仅对**零参**工厂函数生成构造调用；
- C. 仅对**零参且类有零参构造**的方法生成 `let _result = obj.method();`（不做 `assert_eq!`）；
- D. 仅对**零参**全局函数生成调用；
- F. 其余「含非平凡参数」的函数统一以 `// cpp2rust-todo[SMOKE]` 列名占位。

也就是说：**工具对新项目生成的 `tests/smoke.rs` 只验证「可编译/可链接/可调用」，不做行为级 `assert_eq!`**。
示例库里的行为级断言全部是**手写**的，并非工具产物。这与方案 §4.2「表驱动 + 行为级断言、覆盖全部可安全调用接口」尚有差距。

### 4.2 【可选】运算符 `impl std::ops::*` 骨架增强（Phase 6 §6 可选项）⬜

- `src/postprocessor/operator_handler.rs` 目前只产出命名 shim（如 `Vec2_add`），
  未追加 `impl std::ops::Add for ...` 骨架（带 `cpp2rust-todo[OP]`）。方案 §6 标注为「可选」。

### 4.3 残留 `active`/历史形参清理（Phase 6 收尾）🟡

- `emit_dynamic_cast(out, dc, active)` 仍带 `active` 形参（用于「类型是否在 Rust 作用域内」裁决，
  与开关无关，但属可评估收敛项）。需确认是否保留语义或进一步收敛。

### 4.4 文档「冒烟测试」章节随生成器升级二次更新（Phase 8 收尾）⬜

- 一旦 §4.1 生成器升级完成，需把 `INTRODUCTION.md` / `README.md` 的冒烟测试小节，
  从「工具生成类型/链接级冒烟」改述为「工具默认生成行为级冒烟」，并同步 CHANGELOG。

---

## 5. 后续详细实施方案（建议在「一个 PR」中完成剩余全部工作）

> **强烈建议：把 §4 的剩余工作集中在一个 PR 内完成并合并**，避免「生成器升级」与
> 「黄金/文档/CI 断言」跨多个 PR 产生反复回归。该 PR 的垂直切片如下，按顺序推进、单 PR 内自洽。

### 步骤 1 — 重构冒烟生成器为表驱动（对应 §4.1，核心）

1. 在 `src/generator/smoke_test_gen.rs` 引入「FFI 元素类别 → 断言模板」表驱动结构，
   覆盖方案 §4.2 的全部类别：独立/友元/命名空间函数、类构造+getter、setter/getter 往返、
   静态/全局/constexpr、运算符命名 shim、模板函数实例化、模板类别名、虚函数/proxy、
   dynamic_cast、异常、智能指针/RAII。
2. 对「可断言已知结果」者生成 `assert_eq!`（带可读上下文消息）；对「仅能调用不可断言」者
   退化为「构造 + 调用不 panic」并附最小化 `// cpp2rust-todo[SMOKE]`，**最大限度降低占位比例**。
3. 遵循 Rust 测试最佳实践：`#[test] fn smoke_<feature>_<behavior>()`，`unsafe` 块最小化集中在
   FFI 调用点，无全局可变状态；平台差异用 `#[cfg]` / `#[ignore]` 标注并注释说明（对标 v6 L3 先例）。
4. 保持文件级幂等：`tests/smoke.rs` 不存在才生成、已被用户手改则跳过（沿用现 `write_smoke_test` 逻辑）。

### 步骤 2 — 更新生成器单元测试

- 重写 `smoke_test_gen` 单测：去掉「仅类型断言」预期，新增「按特性类别生成正确行为断言骨架」用例。
- 确保表驱动分支均有对应断言覆盖。

### 步骤 3 — 同步黄金与 L6 gen-verify

- 受影响的 L1 黄金（如含模板/类/全局函数的示例 `tests/smoke.rs` 黄金）随生成器输出变化更新。
- `tests/gen_verify_e2e_test.rs`：确认工具对新项目生成的行为级冒烟仍可编译/可链接/可运行
  （必要时扩展断言）。

### 步骤 4 —（可选）运算符 `impl ops` 骨架（对应 §4.2 / 方案 §6）

- 在 `operator_handler.rs` 为 `[OP]` 特性追加 `impl std::ops::*` 注释骨架；同步 019 示例黄金与单测。
- 若评估收益有限，可在该 PR 描述中显式声明「本轮不做、留待后续」，保持范围清晰。

### 步骤 5 —（可评估）`emit_dynamic_cast` 的 `active` 形参收敛

- 确认 `active` 是否仍有必要（类型不在 Rust 作用域时注释化绑定的开关），决定保留或内联收敛。

### 步骤 6 — 文档二次对齐与 CHANGELOG

- 更新 `INTRODUCTION.md` / `README.md` 冒烟测试小节为「工具默认生成行为级冒烟」。
- 在 `CHANGELOG.md` `[Unreleased]` 追加「冒烟生成器表驱动 + 行为级断言」条目。
- 在本文件（`development-progress.md`）把 Phase 1 / 6 / 8 状态更新为 ✅。

### 步骤 7 — 全量验证（该 PR 的 Definition of Done）

1. 不设置任何环境变量，`init` 对**新项目**默认产物即包含模板/proxy/dynamic_cast 骨架
   与**行为级**全特性冒烟测试。
2. 代码库中仍 0 处 `CPP2RUST_GEN_*`（CI 门禁绿）。
3. L1–L6 + L_smoke 全绿；48 示例均可 `cargo test --test smoke`（平台跳过项除外）。
4. `init` / `merge` 命令、参数、输出目录结构与 v6 完全一致。

---

## 6. 风险与缓解（针对剩余工作）

| 风险 | 影响 | 缓解 |
|------|------|------|
| 生成器升级改变多个示例 `tests/smoke.rs` 黄金 | L1 回归面扩大 | 在同一 PR 内逐示例核对 diff；优先用「构造+调用不 panic」兜底，仅对确定性结果加 `assert_eq!` |
| 行为级断言对某些平台不稳定（虚函数/异常） | L_smoke / L3 偶发失败 | `#[cfg]` / `#[ignore]` 标注并在 PR 描述记录，沿用 v6 先例 |
| 表驱动覆盖不全导致占位仍较多 | 「生成即验证」闭环不完整 | 以方案 §4.2 类别表为验收清单，逐类落地，统计占位比例并在 PR 描述披露 |
| 跨多个 PR 推进生成器与黄金 | 反复回归、基线漂移 | **本方案核心建议：在一个 PR 内完成 §5 步骤 1–7** |

---

## 7. 参考

- v7 方案：`docs/plans/v7/automated-cpp2rust-ffi-v7.md`（§4 冒烟测试、§6 去冗余、§8 阶段划分）。
- 生成器：`src/generator/smoke_test_gen.rs`、`src/generator/hicc_codegen.rs`。
- 测试：`tests/{template,proxy,dynamic_cast}_gen_tests.rs`、`tests/l1_golden_tests.rs`、
  `tests/gen_verify_e2e_test.rs`、`examples/*/rust_hicc/tests/smoke.rs`。
- CI：`.github/workflows/ci.yml`（`l-smoke` / `gen-verify` / 门禁 step）。
