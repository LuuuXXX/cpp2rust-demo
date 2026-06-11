# v7 开发进展记录 — 总览

> 本文档跟踪 v7 方案（见 `automated-cpp2rust-ffi-v7.md`）的**目标、阶段、进展与后续计划**。
> 全程使用简体中文。每个阶段落地后可追加独立的 `development-progress-phase-*.md` 详述。

---

## 1. 开发目标（一句话）

把 v6 中由环境变量开关（`CPP2RUST_GEN_TEMPLATES` / `CPP2RUST_GEN_PROXY` /
`CPP2RUST_GEN_DYNAMIC_CAST` / `CPP2RUST_GEN_SMOKE`）控制、默认关闭的高级能力
**毕业为默认行为**，删除全部开关，并补齐覆盖全部特性的行为级冒烟测试——
在不改变 `init` + `merge` 使用方式的前提下，让默认产物即覆盖全特性、符合 hicc 与 Rust 最佳实践。

---

## 2. 硬约束

1. `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变。
2. 不新增、不依赖任何环境变量开关来影响生成结果。
3. 代码库中不再保留 `CPP2RUST_GEN_*` 字样。
4. 冒烟测试默认生成、覆盖全部特性、行为级断言、符合 Rust 测试最佳实践。

---

## 3. 阶段进展表

| 阶段 | 内容 | 状态 | 备注 |
|------|------|------|------|
| Phase 1 | 移除 `CPP2RUST_GEN_SMOKE`；冒烟默认幂等生成 + 表驱动行为级断言 | ⬜ 待开始 | |
| Phase 2 | 移除 `CPP2RUST_GEN_DYNAMIC_CAST`；`@dynamic_cast` 默认输出 + 重做黄金 | ⬜ 待开始 | |
| Phase 3 | 移除 `CPP2RUST_GEN_PROXY`；`@make_proxy` 默认输出 + 重做黄金 | ⬜ 待开始 | |
| Phase 4 | 移除 `CPP2RUST_GEN_TEMPLATES`；模板骨架/别名/工厂默认输出 + 重做黄金 | ⬜ 待开始 | |
| Phase 5 | 冒烟全特性覆盖：48/48 示例具备 `tests/smoke.rs` | ⬜ 待开始 | |
| Phase 6 | 代码去冗余：删除 `env_switch_enabled` 等遗留、emit_* 去 `enabled` 形参、清理注释 | ⬜ 待开始 | |
| Phase 7 | CI：l-smoke 扩全量、gen-verify 覆盖三类高级能力、门禁校验无 `CPP2RUST_GEN_*` | ⬜ 待开始 | |
| Phase 8 | 文档：INTRODUCTION / hicc.md / README / CHANGELOG / DEVELOPMENT 对齐 v7 | ⬜ 待开始 | |

状态图例：⬜ 待开始　🚧 进行中　✅ 已完成

---

## 4. 受影响的关键文件（预估）

- 生成器：`src/generator/hicc_codegen.rs`、`src/generator/smoke_test_gen.rs`、`src/generator/project_generator.rs`
- 命令：`src/commands/init.rs`
- IR / 提取器：`src/ffi_model.rs`、`src/extractor/{template_spec,proxy_spec,dynamic_cast_spec,mod}.rs`、`src/ast_parser/{mod,collector}.rs`
- 测试：`tests/{template_gen_tests,proxy_gen_tests,dynamic_cast_gen_tests,l1_golden_tests,gen_verify_e2e_test}.rs`、各 `examples/*/rust_hicc/tests/smoke.rs`
- CI：`.github/workflows/ci.yml`
- 文档：`docs/INTRODUCTION.md`、`docs/references/hicc.md`、`README.md`、`CHANGELOG.md`、`DEVELOPMENT.md`

---

## 5. 后续计划

- 按 §3 阶段顺序推进，优先完成风险最低、可独立验证的 Phase 1。
- 移除开关的 Phase 2/3/4 各自为「删开关 → 默认输出 → 重做该特性黄金 → 更新 gen 测试」垂直切片，独立 PR。
- 每完成一个阶段，更新本表状态并追加对应 `development-progress-phase-*.md`。
