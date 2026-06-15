# Proposal: Optimize CI Pipeline & Fix Golden Test Mismatch

**Change ID:** `optimize-ci-fix-golden-mismatch`
**Created:** 2026-06-15
**Status:** Archived

---

## Archive Information

**Archived:** 2026-06-15
**Duration:** 0 days (same-day completion)
**Outcome:** Successfully implemented

### Files Modified
- `src/extractor/direct_binding.rs` — shim naming `_with_<param>` + copy ctor filter
- `examples/*/rust_hicc/src/lib_scaffold.rs` (31 files) — regenerated golden files
- `examples/*/rust_hicc/src/lib.rs` (~15 files) — updated shim references
- `.github/workflows/ci.yml` — submodule init, trigger unification, always(), concurrency
- `scripts/ci_local.sh` (new) — local CI script
- `src/bin/regolden.rs` (new) — golden file regeneration tool
- `DEVELOPMENT.md` — local CI documentation
**Completed:** 2026-06-15

---

## Problem Statement

CI 再次大面积失败（4 个 job 失败、2 个 job 被跳过），且缺乏回归验证机制导致反复修复又反复劣化。具体有三大类问题：

### P1: L1 Golden Tests 命名不匹配（26/51 Linux 失败、25/46 MinGW 失败）

FFI scaffold `make_unique` 工厂函数命名约定不一致，当前生成代码与 golden 文件期望输出在以下方面存在偏差：

| 差异点 | 当前生成（left） | Golden 期望（right） | 影响范围 |
|--------|----------------|---------------------|---------|
| 单参数 ctor shim 名 | `_cpp2rust_make_unique_<snake>_<param>` (如 `_buffer_sz`) | `_cpp2rust_make_unique_<snake>_with_<param>` (如 `_buffer_with_sz`) | 所有带 1 参数 ctor 的 class |
| 默认 ctor 处理 | `hicc::make_unique<T>()`（不生成 shim） | 部分 golden 期望 `_cpp2rust_make_unique_<snake>_0()` shim | test_008 等有默认 ctor 的 class |
| 复制 ctor 过滤 | 生成 `buffer_new_with_other`（复制 ctor 工厂） | Golden 不包含复制 ctor 工厂 | test_008 等含 `= delete` 或 `const T&` 的 class |
| 旧 golden 模式 | — | 部分 golden 仍用 `std::make_unique<T>(params)` 直接引用（R4 已证明不可行） | test_017-046 等约 18 个测试 |

**根本原因**：`fix-direct-binding-ci` 修改了代码生成逻辑（增加 shim wrapper），但 golden 文件与 examples 未同步更新，且 shim 命名规则 `_with_<param>` 与代码实现的 `_<param>` 不对齐。

### P2: L5 nm Symbol Validation & L_smoke 失败（submodule 未初始化）

- **L5 nm symbol validation**: 3 个 real-lib example (050/051/052) 编译失败，因为 `references/rapidjson`、`references/pugixml`、`references/nlohmann-json` submodule 未初始化
- **L_smoke Smoke tests**: 24/48 examples 失败，部分因 factory 函数名不匹配（与 P1 同源），部分因 submodule 缺失

**根本原因**：CI 的 `checkout@v4` 不自动初始化 submodule，L5 和 L_smoke job 缺少 `git submodule update --init` 步骤。

### P3: CI 触发不一致 & 缺乏回归验证

| 平台 | 当前触发方式 | 问题 |
|------|-------------|------|
| Linux (ubuntu-latest) | push/PR 自动触发 | 正常 |
| MinGW (windows-latest) | push/PR 自动触发（needs: build） | 正常 |
| macOS (macos-latest) | **仅 workflow_dispatch** | PR 不触发，无法验证 macOS |
| MSVC (windows-latest) | **仅 workflow_dispatch** | PR 不触发，无法验证 MSVC |
| gen-verify | needs: [unit-tests, l1-golden] | L1 失败 → gen-verify 被跳过 → 无法独立验证 |

**问题**：
1. macOS 和 MSVC 在 PR/push 时完全不触发，导致这些平台的问题只在手动 dispatch 后才发现
2. gen-verify 依赖 `success()` 条件，上游失败时级联跳过，无法独立执行回归验证
3. 没有回归测试门控：修复后无法保证不再劣化

---

## Proposed Solution

### S1: 统一 make_unique 工厂命名规则

修改 `src/extractor/direct_binding.rs:634-641` 的 shim 命名逻辑，与文档注释和 golden 期望对齐：

```rust
// 当前代码（需修改）:
if ctor.params.len() == 1 {
    format!("_{}", rust_params[0].0)  // 生成 _sz, _n
} else {
    format!("_{}", ctor.params.len())  // 生成 _2, _3
}

// 修改后:
if ctor.params.len() == 1 {
    format!("_with_{}", rust_params[0].0)  // 生成 _with_sz, _with_n
} else {
    format!("_{}", ctor.params.len())  // 生成 _2, _3（不变）
}
```

保持默认 ctor 使用 `hicc::make_unique<T>()`（可正确工作），不改为 `_0()` shim wrapper。

### S2: 更新所有 L1 golden 文件

- 将约 26 个 golden 文件从旧模式（直接引用 `std::make_unique<T>(params)`）更新为新模式（shim wrapper + `_with_<param>` / `_<N>` 命名）
- 默认 ctor 统一使用 `hicc::make_unique<T>()`，golden 中不再出现 `_0()` shim
- 确保复制 ctor（`const T&` 参数）工厂被正确过滤（不生成）

### S3: 更新 examples 的 lib.rs 文件

约 19-24 个 example 的 `rust_hicc/src/lib.rs` 中引用的 shim 名需从 `_<param>` 更新为 `_with_<param>`，确保与代码生成一致。

### S4: CI 添加 submodule 初始化

在 `l5-nm-symbols` 和 `l-smoke` job 中添加：

```yaml
- name: Init required submodules
  run: git submodule update --init references/rapidjson references/pugixml references/nlohmann-json
```

同时在 `l1-golden` 和 `l2-compile` job 中，如果新增的 050-052 example 需要 golden 测试，也需初始化对应 submodule。

### S5: 统一 CI 触发机制

修改 macOS 和 MSVC job 的触发条件，使其在 push/PR 时也能运行（与 Linux/MinGW 一致）：

```yaml
macos-ci:
  if: github.event_name == 'push' || github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'
  # 删除原来仅 workflow_dispatch 的限制

build-msvc:
  if: github.event_name == 'push' || github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'
```

### S6: 增加回归验证门控

1. **CI job 独立化**：将 `gen-verify` 的 `needs` 条件从 `success()` 改为 `always()`，允许在上游部分失败时仍执行
2. **Golden test 回归 CI**：在 lint 或 unit-tests job 后新增一个 `golden-regression` job，仅检查 L1 golden 是否全量通过，如果任何 golden 失败则标记为 `failure` 并阻止 merge
3. **本地回归脚本**：新增 `scripts/ci-local.sh`，本地模拟完整 CI 流程（build + lint + unit + L1 + L2 + smoke + L5），方便开发者在提交前验证

---

## Scope

### In Scope
- S1: 统一 shim 命名规则（`_with_<param>` for single-param）
- S2: 更新 L1 golden 文件（26 个）
- S3: 更新 examples 的 lib.rs shim 引用（约 19 个）
- S4: CI submodule 初始化（L5、L_smoke）
- S5: macOS/MSVC 触发条件修改
- S6: 回归验证门控（gen-verify always、本地脚本）

### Out of Scope
- 复制 ctor 过滤逻辑修改（如已有过滤但特定 case 未生效，需在 S2 中一并验证）
- hicc framework 本身修改（ClassMutPtr、String 类型碰撞等问题）
- 新增 050-052 example 的 L1/L2/E2E CI job（deferred from real-lib-direct-examples）

---

## Impact Analysis

| Component | Change Required | Details |
|-----------|-----------------|---------|
| src/extractor/direct_binding.rs | Yes | shim 命名从 `_<param>` 改为 `_with_<param>` |
| tests/l1_golden_tests.rs | Possible | golden 期望值更新（约 26 个 inline golden string） |
| examples/*/rust_hicc/src/lib.rs | Yes | shim 引用名更新 |
| .github/workflows/ci.yml | Yes | submodule init、触发条件、回归门控 |
| scripts/ci-local.sh | New | 本地 CI 模拟脚本 |

---

## Architecture Considerations

- `make_unique` 工厂命名是 extractor → hicc_codegen → example 的贯穿约定，一处修改需三方同步
- `hicc::make_unique<T>()` 对默认 ctor 可正确工作（hicc 纯 C++ 函数指针解析），无需改为 shim wrapper
- `gen-verify` 改为 `always()` 后，即使 L1 失败也可执行，提供独立回归验证
- macOS/MSVC 触发 push/PR 后会增加 CI 时间，但可通过 `concurrency` group 控制资源

---

## Success Criteria

- [ ] L1 golden tests: 51/51 Linux 全通过、46/46 MinGW 全通过
- [ ] L_smoke Smoke tests: 48/48 全通过
- [ ] L5 nm symbol validation: 所有 example 编译成功
- [ ] gen-verify: 不再因上游失败被跳过
- [ ] macOS CI: push/PR 时自动触发
- [ ] MSVC CI: push/PR 时自动触发
- [ ] 本地回归脚本 `scripts/ci-local.sh` 可执行完整验证流程
- [ ] `cargo test --lib` + `cargo clippy` + `cargo fmt --check` 通过

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| 26 个 golden 文件更新遗漏 | Medium | High | 用脚本批量对比生成输出 vs golden，逐个确认 |
| macOS/MSVC 触发 push/PR 后 CI 时间翻倍 | Low | Medium | macOS/MSVC 仅跑 build + unit + L1，不跑全量 |
| `_with_<param>` 命名与旧 example lib.rs 不一致 | Medium | High | 同步更新所有 example lib.rs 中的 shim 引用 |
| gen-verify always() 在上游失败时产生额外噪音 | Low | Low | 仅作为 warning，不阻止 merge |
