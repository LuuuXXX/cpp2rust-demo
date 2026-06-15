# Delta: CI Workflow Configuration

**Change ID:** `optimize-ci-fix-golden-mismatch`
**Affects:** .github/workflows/ci.yml, L5 nm symbols, L_smoke, macOS CI, MSVC CI, gen-verify

---

## ADDED

### Requirement: Submodule 初始化步骤

L5 nm symbol validation 和 L_smoke Smoke tests job 必须在 checkout 后初始化 third-party library submodule。

#### Scenario: L5 nm symbol validation submodule init
- GIVEN L5 job 执行 `actions/checkout@v4`
- WHEN 步骤包含 `git submodule update --init references/rapidjson references/pugixml references/nlohmann-json`
- THEN examples 050-052 的 C++ 编译路径正确解析 `rapidjson/document.h`、`pugixml.hpp`、`nlohmann/json.hpp`

#### Scenario: L_smoke submodule init
- GIVEN L_smoke job 执行 `actions/checkout@v4`
- WHEN 步骤包含 `git submodule update --init references/rapidjson references/pugixml references/nlohmann-json`
- THEN smoke tests 可编译依赖这些 submodule 的 example

---

### Requirement: macOS/MSVC 自动触发

macOS CI 和 Windows MSVC job 在 push/PR 时自动触发，与 Linux/MinGW 行为一致。

#### Scenario: macOS CI push 触发
- GIVEN `github.event_name == 'push'` 且目标分支为 main/master
- WHEN macOS CI job 条件为 `always()` 或无 `if` 限制
- THEN macOS CI job 自动运行 build + unit + L1

#### Scenario: macOS CI PR 触发
- GIVEN `github.event_name == 'pull_request'` 且目标分支为 main/master
- WHEN macOS CI job 无 `workflow_dispatch` 独占限制
- THEN macOS CI job 自动运行

#### Scenario: MSVC push/PR 触发
- GIVEN push/PR 事件触发 CI
- WHEN MSVC job 条件不再限制为 `workflow_dispatch`
- THEN MSVC build job 自动运行

---

### Requirement: gen-verify 独立执行

gen-verify 和 gen-verify (MinGW) job 在上游部分失败时仍可执行，提供独立回归验证。

#### Scenario: L1 golden 失败时 gen-verify 仍执行
- GIVEN L1 golden tests job 失败（exit code 1）
- WHEN gen-verify `needs` 条件使用 `always()` 而非隐式 `success()`
- THEN gen-verify job 正常执行（不受上游失败阻塞）

#### Scenario: gen-verify 自身失败时标记为 failure
- GIVEN gen-verify job 执行完毕且有测试失败
- THEN CI 整体状态为 failure（不因上游 skip 被掩盖）

---

### Requirement: 本地 CI 回归脚本

新增 `scripts/ci_local.sh`，允许开发者在提交前本地执行完整 CI 流程验证。

#### Scenario: 本地全量验证
- GIVEN 开发者在项目根目录执行 `bash scripts/ci_local.sh`
- WHEN 脚本依次执行 build → lint → unit → L1 → L2 → tinyxml2 E2E
- THEN 所有步骤通过则返回 exit code 0，任一失败则返回非零 exit code

#### Scenario: 快速验证模式
- GIVEN 开发者执行 `bash scripts/ci_local.sh --quick`
- WHEN 仅执行 build + lint + unit tests（跳过 L1/L2/E2E）
- THEN 快速获得基本质量反馈

---

## MODIFIED

### Requirement: CI 矩阵裁剪（原 spec: repo-slim.md）

原 spec 要求 macOS/MSVC 改为 `workflow_dispatch`，现改为 push/PR 也自动触发。

#### Scenario: macOS / MSVC 自动触发
- THEN macOS 与 MSVC job 在 push/PR 时自动触发（不再仅限 `workflow_dispatch`）
- AND `workflow_dispatch` 仍作为手动触发备选保留
- AND `.github/workflows/ci.yml` 总 job 数仍 ≤ 15

---

## REMOVED

（None — 无需求删除，仅修改触发条件）
