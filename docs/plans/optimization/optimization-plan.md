# cpp2rust-demo 优化方案

> 本文档是 cpp2rust-demo 仓库的**结构性优化方案**（区别于 v6/v7 的 FFI 能力演进方案）。
> 目标是在不改变 `init` + `merge` 使用方式、不破坏现有 CI 行为的前提下，
> 分阶段完成**仓库瘦身、生成产物对齐 hicc 最佳实践、测试与 CI 增强、文档收敛**。
> 全程使用简体中文。落地进展见同目录 `development-progress.md`。

---

## 1. 背景

v7 方案完成后（详见 `docs/plans/v7/`），工具已实现「`init` + `merge` 默认产出覆盖全部 48
特性、符合 hicc 与 Rust 最佳实践」的目标。在此基础上，本优化方案聚焦**工程质量与可维护性**：

- 仓库内 vendored 的第三方参考副本（含大体积二进制）拖慢克隆、模糊归属边界。
- 生成产物的「shim 策略」需要与 [`references/hicc-usages`](../../../references/hicc-usages)
  人工标准答案集显式对齐，明确「哪些特性零 shim（L0）、哪些必须最小 shim（L1）」。
- 导出符号级别的行为冒烟、每库独立 CI、真实项目矩阵扩充等仍有提升空间。
- 文档分散，缺少统一的优化方案与进展跟踪入口。

---

## 2. 硬约束

1. `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变。
2. 不改变现有 CI 各 job 的既有行为：`actions/checkout` 不自动拉取子模块，
   仅按需 `git submodule update --init <指定路径>`。
3. 涉及默认生成产物变化的阶段（B）必须**跨平台**重生成并校验 golden（Linux/macOS/Windows），
   不得仅凭单平台结果合入。
4. 测试依赖（如 `references/rapidjson-refactoring` 的 E2E 角色）在替换前保持可用。

---

## 3. 阶段总览

| 阶段 | 内容 | 风险 | 可在沙箱本地验证 | 状态 |
|------|------|------|------------------|------|
| A | 仓库瘦身与子模块化 | 低 | 是（仅文件/子模块布局） | ✅ 已完成（PR #157） |
| E | 文档与导航（L0/L1 分级、hicc-usages 对账） | 低 | 是（仅 Markdown） | ✅ 已完成（PR #157） |
| B | `cpp_block` 落地 L0/L1 shim 分级 + 重生成 L1 golden | 高 | 否（需跨平台 golden 回归） | ⬜ 待开始 |
| C | 导出符号冒烟测试全覆盖 | 中 | 部分（需各库可链接产物） | ⬜ 待开始 |
| D | 每库独立 CI + 扩充真实项目矩阵 | 中 | 否（依赖 CI 基础设施） | ⬜ 待开始 |
| A 收尾 | `references/hicc` 子模块化、`examples/` 与 hicc-usages 去重 | 中 | 否（gitcode 网络受限） | ⬜ 阻塞中 |

状态图例：⬜ 待开始　🚧 进行中　✅ 已完成

> **排序原则**：先做「低风险 + 可本地验证」的 A/E，再推进需要跨平台回归或外部基础设施的 B/C/D。
> 这样每个增量都能独立验证、独立合入，避免把不可在当前沙箱验证的改动与可验证改动耦合。

---

## 4. 阶段细节

### 阶段 A — 仓库瘦身与子模块化（✅ 已完成）

- `references/c2rust-demo`：移除约 4M vendored 副本及 2 个 `.rar` 二进制，改为 GitHub 子模块
  （`https://github.com/LuuuXXX/c2rust-demo`）。
- 新增 `references/hicc-usages` GitHub 子模块（48 特性 × hicc 人工标准答案集，
  作为本工具自动生成结果的对齐基准）。
- `references/hicc`：gitcode 源（`https://gitcode.com/xuanwu/hicc`）在 CI/沙箱网络被屏蔽
  （`Could not resolve host: gitcode.com`），**暂保留 vendored 副本**，已在 `.gitmodules`
  注释记录预期来源，待网络条件具备后正式子模块化（见「阶段 A 收尾」）。
- 历史方案归档：`docs/plans/v5` → `docs/archive/v5`（v6 被 DEVELOPMENT.md 活跃引用、
  v7 为当前实现，均保留），新增 `docs/archive/README.md`。

### 阶段 E — 文档与导航（✅ 已完成）

- README「生成代码格式」将原「最小 shim 策略」强化为明确的
  **L0 无 shim（默认，约 34/48）/ L1 最小 shim（约 14/48）分级**，并引用 hicc-usages 表1/表2。
- README「特性矩阵」增补 hicc-usages 标准答案对账说明；「仓库结构」更新为子模块布局。
- DEVELOPMENT.md「示例矩阵」关联 hicc-usages 标准答案集。

### 阶段 B — `cpp_block` 落地 L0/L1 shim 分级 + 重生成 L1 golden（⬜ 待开始）

- 在生成器侧把「L0 无 shim / L1 最小 shim」从**文档约定**落到**代码行为**：
  对 L0 特性，`cpp!` 块仅保留必要 `#include`，不重新内联类定义与方法体；
  仅对 L1 特性（ctor/dtor、静态成员 getter/setter、运算符重载、placement new、
  多继承/菱形虚继承、`volatile`、RTTI、模板具现、`tuple`/`enum class`/`union`、
  STL 容器 wrapper 的 ctor/dtor）生成最小命名空间级 inline 包装。
- **必须**跨平台（Linux/macOS/Windows）重生成并校验受影响示例的 L1 golden，逐字节回归。
- 受影响关键文件（预估）：`src/generator/`（`cpp_block` 相关）、`tests/l1_golden_tests.rs`、
  各 `examples/*/rust_hicc/`、`.github/workflows/ci.yml`（gen-verify）。

### 阶段 C — 导出符号冒烟测试全覆盖（⬜ 待开始）

- 在现有行为级 `tests/smoke.rs`（48/48 覆盖）基础上，补齐**导出符号级**冒烟：
  对每个示例的被转换库产物，断言期望的可链接符号存在并可调用。
- 依赖各库可链接产物，需在 CI 中先编译被转换库再运行符号断言。

### 阶段 D — 每库独立 CI + 扩充真实项目矩阵（⬜ 待开始）

- 把当前集中式 CI 拆为「每个真实库一个独立 job」，缩短反馈回路、隔离失败。
- 扩充真实项目矩阵（候选见决策点 4），覆盖更多真实 C++ 工程形态。

### 阶段 A 收尾（⬜ 阻塞中）

- `references/hicc`：待 gitcode.com 在 CI/沙箱可达后，从 vendored 副本切换为正式子模块。
- `examples/` 与 hicc-usages 去重：需先重构 L1/L2/L3 取数路径，再把全量 48 标准答案
  下沉到 hicc-usages 子模块，本仓库只留回归所需最小样例。

---

## 5. 待确认决策点

1. 「无 shim」目标确认为**默认 L0、仅 ⚠️ 特性保留最小 shim**（约 14/48 无法完全去除）？
2. `references/rapidjson-refactoring`（约 12M）：先抽独立仓库再子模块化，
   还是用其它公开库替换其 E2E 角色？
3. 是否同意将全量 48 标准答案交由 hicc-usages 子模块承载，本仓库只留回归所需最小样例？
4. 是否需要推荐一批可子模块化的新增真实项目（用于阶段 D 矩阵扩充）？

---

## 6. 相关文档

- FFI 能力演进：`docs/plans/v6/`、`docs/plans/v7/`
- 历史归档：`docs/archive/`
- 技术细节：`docs/INTRODUCTION.md`
- 对齐基准：`references/hicc-usages`（hicc FFI 人工标准答案集，表1/表2）
