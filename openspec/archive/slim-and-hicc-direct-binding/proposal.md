# Proposal: 项目瘦身 + 直接 hicc 绑定（去除 C++ extern-C shim 层）

**Change ID:** `slim-and-hicc-direct-binding`
**Created:** 2026-06-15
**Status:** Archived
**Completed:** 2026-06-15

---

## Problem Statement

当前 `cpp2rust-demo` 存在三类与"轻量化"目标相悖的问题：

### 1. 仓库膨胀，新增了非必要的体积
- `references/` 目录合计约 **17 MB**，其中 `rapidjson-refactoring/`（12 MB）和 `c2rust-demo/`（4 MB）属于历史快照，不是核心 init+merge 流程必需。
- `examples/006_class_basic/cpp/class_basic.cpp` 等示例的 C++ 端存在**冗余的 extern-C 访问器**（`counter_new` / `counter_get` / `counter_increment` / ...），这些"shim 函数"只是把 C++ 类的方法包成 C ABI。
- `usage/`、`DEVELOPMENT.md`（28 KB）、`CHANGELOG.md`（8 KB）等文档体量偏大，许多内容与主线流程无关。

### 2. C++ 项目接入门槛高 —— 必须先手写 extern-C shim
现有工作流要求 C++ 项目同时提供：
- 真正的 C++ 类（如 `class Counter { int get() const; ... }`）
- **额外的 extern-C shim 函数**（如 `int counter_get(struct Counter* self) { return self->get(); }`）

工具实际绑定的是 shim 函数。但 `hicc` 框架本身支持通过 `#[cpp(method = "int get() const")]` **直接绑定 C++ 类方法**（参考 `hicc-usages/examples/006_class_basic`），不需要这一层 C 包装。shim 层对用户而言是额外的、非必要的工作量。

### 3. 测试体系庞大，与瘦身目标不匹配
- `tests/` 下 16 个测试文件，其中 9 个 `_e2e_test.rs` 依赖 `references/*` 子模块（rapidjson / tinyxml2 / pugixml / sqlite3 / nlohmann-json / fmtlib），CI 中跨 4 个平台 × 多个 E2E 测试组合，单次运行时间很长。
- `gen_verify_e2e_test.rs` 跑全部 48 个示例的端到端生成，在 CI 上常因内存/超时失败。
- 用户明确反馈"机器的内存不够"，需要减少一次性并发。

### 谁受影响
- **使用者**：希望接入任意 C++ 项目到 hicc，不希望被迫先写一遍 C 包装。
- **维护者**：CI 跑得久、本地内存吃紧；`references/*` 子模块膨胀拖慢 clone。
- **hicc-usages 仓库**：与 cpp2rust-demo 的 examples 重叠，但模式更先进（无 shim），是本提案的参考基线。

---

## Proposed Solution

按 **5 个 Phase 串行推进**（不并发，避免 OOM），每阶段独立可验收，每阶段产物落盘后再进入下一阶段。

| Phase | 内容 | 主要产出 |
|---|---|---|
| **P1** | 核心生成器支持「无 shim 直接绑」模式 | 新增 `extractor/direct_binding.rs`；hicc_codegen 输出 `#[cpp(method = "...")]` 直绑 |
| **P2** | examples 改造（**分 6 批**，每批 ≤ 8 项）| 48 个示例的 cpp/ 去掉 extern-C shim，rust_hicc/ 改为直接绑 C++ 类 |
| **P3** | references 瘦身 | 删除 rapidjson-refactoring、c2rust-demo 历史快照；保留 5 个 git submodule（已在 .gitmodules） |
| **P4** | 测试体系精简 | L1（golden）、L2（compile）、L_smoke（48 cargo test smoke）保留；删除/瘦身 9 个 e2e |
| **P5** | 文档与 CI 适配 | README 瘦身、DEVELOPMENT 精简；CI 矩阵裁剪 |

### 关键设计决策

1. **保留 init+merge 命令行接口与 .cpp2rust/ 目录结构不变**。用户使用方法零变化（`cpp2rust-demo init -- make`、`cpp2rust-demo merge --feature X`）。

2. **「直接绑」作为新的提取模式**，与现有 shim 模式**共存**而非替换：
   - 当 C++ 项目已提供 extern-C shim（如 references/rapidjson）→ 继续按现有路径提取。
   - 当 C++ 项目只有原生 C++ 类（如 examples）→ 走新的 direct_binding 路径，由 hicc 直接绑方法。
   - 由 `extractor` 自动识别：若 AST 中找到 `class Counter` 与同名 `counter_*` 自由函数配对，按"直接绑"处理。

3. **Windows hook_shim 保留**：`hook/hook_shim.rs` 是 Windows 平台编译器拦截机制（PATH 注入替代 LD_PRELOAD），与"C++ 端 extern-C shim 函数"是**两件不同的事**。前者是工具运行机制，必须保留。

4. **examples 改造分批**：参考 `hicc-usages/openspec/changes/cpp-feature-matrix` 的批次方案（A-F，每批 8 项），跑完一批才进下一批，每批结束清理 `target/`、`build/`。

5. **测试体系裁剪原则**：
   - 保留：`cargo test --lib`（单元）、`cargo test --bin`（CLI）、L1 golden、L2 compile、L_smoke。
   - 瘦身：保留 **1 个** 端到端测试（tinyxml2，体积最小、构建最快），作为 init+merge 全流程的回归。
   - 删除：rapidjson_e2e、pugixml_e2e、sqlite3_e2e、nlohmann_json_e2e、fmtlib_e2e、multi_feature_e2e（合并入 L_smoke 或转为可选 `#[ignore]`）、gen_verify_e2e（48 项串行耗时太长，改为只跑 8 个代表性样本）。

6. **冒烟测试增强**：保留现有 `smoke_test_gen.rs` 的「生成即验证」思路，同时参考 hicc-usages 的 `tests/smoke.rs`，对每个能导出符号的接口生成断言式测试（不只是"能编译"，而是 assert 返回值）。

7. **文档瘦身原则**：
   - README 保留：工作原理、命令参考、快速开始（Linux/macOS/Windows）、最小示例。
   - README 删除：长篇踩坑总览、shim 工作流（不再是推荐路径）、L3-L5 测试说明（改为 DEVELOPMENT.md 简短引用）。
   - 删除 `usage/` 下的脚本（已被 CI 覆盖）。

---

## Scope

### In Scope
- 核心生成器新增 `direct_binding` 提取模式，与 shim 模式共存。
- 48 个 examples 的 cpp/ + rust_hicc/ 重做（去掉 extern-C shim，直接绑 C++ 类）。
- `references/` 删除 `rapidjson-refactoring/`、`c2rust-demo/`（共约 16 MB）。
- 测试体系精简：保留单元测试 + L1 + L2 + L_smoke + 1 个 e2e（tinyxml2）。
- README、DEVELOPMENT、CI 配置同步适配。
- 冒烟测试增强：参考 hicc-usages，对零参可调用接口生成 assert 断言。

### Out of Scope
- **不动** `init`/`merge` 的命令行参数与 `.cpp2rust/<feature>/` 输出目录结构。
- **不动** `hook/` 目录（hook.cpp、Makefile、hook_shim.rs 都是平台机制，必须保留）。
- **不修改** `hicc` 框架本身（cpp2rust-demo 依赖 hicc，但 hicc 是只读依赖）。
- **不做** Windows MSVC 平台的新一轮覆盖验证（瘦身后只在 Linux + MinGW 上验证，MSVC 视 CI 时间余量选做）。
- **不做** 性能基准测试。
- **不替换** AST 解析后端（继续用 libclang）。

---

## Impact Analysis

| Component | Change Required | Details |
|---|---|---|
| `src/extractor/` | **Yes** | 新增 `direct_binding.rs`：识别"原生 C++ 类 + 无配对 shim" 模式，输出 direct 风格的 ClassSpec |
| `src/generator/hicc_codegen.rs` | **Yes** | 增加 direct 模式输出：`import_class!` 内直接 `#[cpp(method = "...")]` 而非依赖 import_lib 的 shim 函数 |
| `src/generator/smoke_test_gen.rs` | **Yes** | 增强：对零参工厂/方法生成 assert（如 `assert_eq!(c.count(), 0)`），参考 hicc-usages |
| `src/commands/{init,merge}.rs` | **No** | 命令接口完全不变 |
| `src/capture.rs` / `hook/` | **No** | 编译拦截机制保持原样 |
| `src/ast_parser/` | **No** | AST 提取逻辑不变，只是下游 extractor 解释方式改变 |
| `examples/` | **Yes** | 48 项分 6 批改造（cpp 去 shim，rust_hicc 改直绑）|
| `references/` | **Yes** | 删除 `rapidjson-refactoring/`、`c2rust-demo/`；保留 5 个 submodule |
| `tests/` | **Yes** | 删除 5 个 e2e、瘦身 2 个；保留 L1/L2/L_smoke + tinyxml2_e2e |
| `Cargo.toml` | **No**（可能） | 依赖列表不变 |
| `.gitmodules` | **No** | 保留 5 个 submodule 条目 |
| `.github/workflows/ci.yml` | **Yes** | 删除已移除测试对应的 job；macOS job 简化或选做 |
| `README.md` | **Yes** | 去掉 shim 工作流章节、瘦身快速开始 |
| `DEVELOPMENT.md` / `usage/` / `CHANGELOG.md` | **Yes** | 大幅精简 |

---

## Architecture Considerations

### 共存而非替换：shim 模式 vs direct 模式

```
C++ 项目（含 extern-C shim 函数）         C++ 项目（仅原生 C++ 类）
            ↓                                       ↓
       extractor::lib_spec                  extractor::direct_binding
            ↓                                       ↓
   FnBinding (绑 shim 函数)              ClassSpec (直接含方法签名)
            ↓                                       ↓
   hicc_codegen::import_lib!            hicc_codegen::import_class!
   （旧路径，保留以兼容）                （新路径，默认推荐）
```

**自动判定**：extractor 收集完 AST 后，对每个 `class Foo`：
- 若发现 `foo_*` 形式的自由函数且参数含 `Foo*` → 判定为 shim 模式，沿用旧路径。
- 否则 → 判定为 direct 模式，class 的 methods 直接进入 ClassSpec。

### 测试目录瘦身后的结构

```
tests/
├── l1_golden_tests.rs        # 保留：代码生成准确性（含 direct 模式新 golden）
├── l2_compile_tests.rs       # 保留：示例可编译
├── l_smoke_*（合入 l2）       # L_smoke 由 CI shell 循环跑 48 个 cargo test --test smoke
├── tinyxml2_e2e_test.rs      # 保留：唯一端到端回归
├── dynamic_cast_gen_tests.rs # 保留：单元测试
├── proxy_gen_tests.rs        # 保留：单元测试
└── template_gen_tests.rs     # 保留：单元测试
```

### references/ 瘦身后

```
references/
├── tinyxml2/        # submodule
├── pugixml/         # submodule（保留供可选 e2e 扩展，可改为 opt-in）
├── sqlite/          # submodule
├── nlohmann-json/   # submodule
├── fmtlib/          # submodule
└── hicc/            # 保留（非 submodule，工具自身依赖参考）
```

> 注：`rapidjson-refactoring/` 与 `c2rust-demo/` 是历史快照，非 submodule，删除后无 `.gitmodules` 影响。

### 内存友好的批次执行

参考 `hicc-usages/README.md` 的批次方案：
- 批 A: 001-008
- 批 B: 009-016
- 批 C: 017-024
- 批 D: 025-032
- 批 E: 033-040
- 批 F: 041-048

每批结束 `find examples/ -name target -type d -exec rm -rf {} +`。

---

## Success Criteria

### P1（核心生成器）
- [ ] 新增 `src/extractor/direct_binding.rs`，含自动判定逻辑（shim vs direct）。
- [ ] `hicc_codegen::generate` 支持 direct 模式输出，与 shim 模式共存。
- [ ] 新增单元测试 ≥ 5 个，覆盖：纯 C++ 类、含模板、含继承、含 const 方法、混合（部分 shim + 部分直绑）。
- [ ] L1 golden 测试为 direct 模式新增 ≥ 2 个 fixture。

### P2（examples 改造）
- [ ] 48 个 examples 的 cpp/ 全部去除 extern-C shim 函数，仅保留 C++ 类。
- [ ] 48 个 examples 的 rust_hicc/ 使用 `#[cpp(method = "...")]` 直接绑 C++ 类方法（参考 hicc-usages）。
- [ ] 48 个 examples 的 `tests/smoke.rs` 至少 1 个 assert 断言测试通过。
- [ ] 批次执行无 OOM（每批 ≤ 8 项）。

### P3（references 瘦身）
- [ ] `references/rapidjson-refactoring/` 删除（约 12 MB）。
- [ ] `references/c2rust-demo/` 删除（约 4 MB）。
- [ ] 5 个 submodule 在 `.gitmodules` 中保留。

### P4（测试体系精简）
- [ ] `tests/rapidjson_e2e_test.rs` 删除或转为 `#[ignore]`。
- [ ] `tests/pugixml_e2e_test.rs`、`sqlite3_e2e_test.rs`、`nlohmann_json_e2e_test.rs`、`fmtlib_e2e_test.rs`、`multi_feature_e2e_test.rs` 删除。
- [ ] `gen_verify_e2e_test.rs` 缩减为 8 个代表性样本。
- [ ] `cargo test --lib` 通过；`cargo test --test l1_golden_tests -- --test-threads=1` 通过。
- [ ] CI 总运行时间下降 ≥ 30%。

### P5（文档与 CI）
- [ ] README.md 从 50 KB 瘦身至 ≤ 20 KB，去除 shim 工作流章节。
- [ ] DEVELOPMENT.md 精简至 ≤ 15 KB。
- [ ] `usage/` 目录评估：保留必要的 verify 脚本，删除冗余。
- [ ] `.github/workflows/ci.yml` 删除已移除测试对应的 job；保留 Linux + Windows MinGW 双平台覆盖。

### 全局
- [ ] `cpp2rust-demo init -- make` / `cpp2rust-demo merge` 使用方法零变化。
- [ ] 全程串行执行，无一次性高并发任务。

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| direct 模式与 shim 模式判定边界模糊（混合项目）| Med | High | extractor 先收集所有 AST 再做配对判定；混合项目优先 shim 模式（保守）；单元测试覆盖混合场景 |
| 48 个 examples 改造工作量大、易出错 | High | Med | 严格按 6 批推进；每批跑 L_smoke 验证；出错回滚单批 |
| 删除 references/rapidjson 后 `rapidjson_e2e_test.rs` 无法运行 | High | Low | 同步删除该测试（已在 P4 中）|
| hicc 框架在某些场景对直绑支持不完整（如虚继承、RTTI）| Med | Med | 参考 hicc-usages 的"部分支持"标记；这些示例保留 cpp! 包装；README 标注限制 |
| CI 矩阵裁剪导致回归漏检 | Med | Med | 保留 Linux + Windows MinGW 双平台；MSVC/macOS 改为手动触发 |
| 旧用户依赖 shim 工作流（references/rapidjson）| Low | Low | shim 模式保留兼容；文档说明两种模式差异 |
| 内存不够（用户明确反馈）| High | High | 每批 ≤ 8 项；阶段间清理 target/；CI 单线程跑 golden 测试 |
| 文档瘦身误删关键信息 | Low | Low | 删除前 git diff 自审；保留 init/merge/快速开始核心章节 |

---

## 实施约束

- **语言**：所有产出（代码注释、文档、commit message）使用简体中文。
- **执行节奏**：每完成一个 Phase 主动停下，等待用户确认后再进入下一 Phase。**方案本身只需一次确认**，不在每个子任务都问"继续"。
- **并发上限**：构建/测试任务最多 `--test-threads=1` 或 `xargs -P1`，单批内不超过 8 项并行。
- **可逆性**：所有变更通过 git 提交分阶段保留；references 删除前确认无未推送修改。

---

## Archive Information

**Archived:** 2026-06-15 16:00
**Duration:** <1 day (same-day completion)
**Outcome:** Successfully implemented

### Files Modified
- `src/extractor/direct_binding.rs` — new file: classify(), build_direct_class_specs(), build_direct_lib_spec()
- `src/extractor/mod.rs` — Direct mode branch in extract()
- `src/ffi_model.rs` — BindingMode enum, is_copy_ctor field
- `src/generator/hicc_codegen.rs` — Direct mode generation
- `src/generator/smoke_test_gen.rs` — default_value_literal() + assert generation
- `examples/*/cpp/*.h, *.cpp` — extern-C shim removed from all 48 examples
- `examples/*/rust_hicc/src/lib.rs, lib_scaffold.rs, main.rs` — Direct mode bindings
- `references/rapidjson-refactoring/, references/c2rust-demo/` — deleted (git rm)
- `tests/` — 6 e2e tests deleted, gen_verify reduced to 8 active + 40 #[ignore]
- `README.md, DEVELOPMENT.md, CHANGELOG.md` — slimmed
- `.github/workflows/ci.yml` — CI reduced to 14 jobs
- `docs/direct-vs-shim-binding.md, docs/feature-matrix.md` — new docs
- `Makefile` — simplified

### Specs Updated
- `openspec/specs/extractor.md` — Direct binding mode, BindingMode, factory rules, deleted ctor filtering
- `openspec/specs/repo-slim.md` — examples, references, tests, docs, CI slimming
