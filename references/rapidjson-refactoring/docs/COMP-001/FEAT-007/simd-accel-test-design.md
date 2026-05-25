# Feature 级测试设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-007` |
| feature 名称 | `simd-accel` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`simd-accel-dev-design.md` 开发设计文档](./simd-accel-dev-design.md) |
| 是否商用代码 | `是` |
| 依赖使用策略 | `core/std-only`（禁止 crates.io 第三方依赖） |

---

## 目录

- [1. 测试概述](#1-测试概述)
- [2. 基线层](#2-基线层)
- [3. 镜像测试层](#3-镜像测试层)
- [4. 孪生测试层](#4-孪生测试层)

---

## 1. 测试概述

### 1.0 商用代码与测试依赖约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | SIMD 测试不引入第三方基准/性能分析框架，仅使用 gtest/perftest 和 `cargo bench`/`cargo test`。 |
| 对当前 feature 技术选型的影响 | 性能验证与行为验证全部依赖现有 C++ perftest 与 Rust 自有测试，不引入额外性能工具链。 |

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 SIMD 路径与标量路径的行为完全一致；
- 在支持 SIMD 的平台上，验证 SIMD 路径对空白跳过、字符串扫描等关键场景有预期的性能提升；
- 验证在不支持 SIMD 或禁用 SIMD 时，系统正确退化到标量路径且行为不变。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | 空白跳过/未转义字符串扫描等热点路径行为与性能验证 | 仅针对内部优化点 |
| 排除范围 | 功能正确性（由 encoding/sax/dom 测试覆盖），SIMD 测试仅关注“同一功能不同实现路径”的等价性与性能 |  |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest | 执行 `simdtest.cpp` 中 SIMDish 行为测试 | 与 legacy RapidJSON 项目一致 |
| C/C++ perftest harness | 执行 `rapidjsontest.cpp` 中 SkipWhitespace/Scan* 性能测试 | 同 legacy 项目 |
| `cargo test` | 执行 Rust SIMD 行为镜像与孪生测试 | 与 workspace Rust 版本一致 |
| `cargo bench` 或自定义基准 | 在 Rust 实现上进行性能比较 | 可选 |

### 1.3 测试环境

SIMD 相关测试对硬件与编译器特性敏感，需要指定：
- 支持 SIMD 指令集的平台配置（如 x86_64 SSE2/SSE4.2、ARM NEON）；
- 非 SIMD 平台配置，用于验证回退路径；

具体平台矩阵在实现阶段细化，本节**无特殊设计**。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 使用 `simdtest.cpp` 与 `rapidjsontest.cpp` 中相关用例冻结 C++ 行为与性能特征。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 C++ 测试编写 Rust 镜像测试，在同一硬件/编译器配置下对比行为与性能。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 Rust SIMD 实现编写原生测试与基准，验证与镜像测试行为一致并比较性能。 | [4](#4-孪生测试层) |

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/
│       ├── unittest/
│       │   └── simdtest.cpp                      # SIMD 相关行为测试
│       └── perftest/
│           └── rapidjsontest.cpp                 # 包含 SkipWhitespace/Scan* 性能测试
│
├── inventory/
│   └── simd-accel.legacy_tests.json              # SIMD 相关 Legacy 测试资产清单
│
├── baseline/
│   └── simd-accel.golden_samples.jsonl           # 性能与行为黄金样本（输入大小/形态与 C++ 基线）
│
└── reports/
    ├── simd-accel.legacy.junit.xml               # Legacy SIMD 行为测试执行结果
    └── simd-accel.legacy.perf.json               # Legacy 性能基线（可自定义格式）
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| SIMD.ScanCopyUnescapedString_SSE42 | test/unittest/simdtest.cpp:164 | legacy_test |
| SIMD.ScanWriteUnescapedString_SSE42 | test/unittest/simdtest.cpp:169 | legacy_test |
| SIMD.SkipWhitespace_EncodedMemoryStream_SSE42 | test/unittest/simdtest.cpp:82 | legacy_test |
| SIMD.SkipWhitespace_SSE42 | test/unittest/simdtest.cpp:77 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── simd_ffi/                                  # SIMD 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── simd-accel.legacy_to_mirror.json           # gtest/perftest -> Rust 镜像测试映射表
│
├── reports/
│   └── simd-accel.mirror.junit.xml                # 镜像层行为测试执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── simd_accel_mirror.rs               # SIMD 行为镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| SIMD.ScanCopyUnescapedString_SSE42 | test/unittest/simdtest.cpp:164 | simd_accel_mirror::SIMD_ScanCopyUnescapedString_SSE42 | |
| SIMD.ScanWriteUnescapedString_SSE42 | test/unittest/simdtest.cpp:169 | simd_accel_mirror::SIMD_ScanWriteUnescapedString_SSE42 | |
| SIMD.SkipWhitespace_EncodedMemoryStream_SSE42 | test/unittest/simdtest.cpp:82 | simd_accel_mirror::SIMD_SkipWhitespace_EncodedMemoryStream_SSE42 | |
| SIMD.SkipWhitespace_SSE42 | test/unittest/simdtest.cpp:77 | simd_accel_mirror::SIMD_SkipWhitespace_SSE42 | |#### 3.3 迁移策略建议

| 测试类别 | 进入 L1 优先级建议 | 原因 |
|----------|--------------------|------|
| SIMD 行为测试（unittest） | high | 确保行为对齐是前提。 |
| perftest 性能场景 | medium | 在行为对齐后可以逐步迁移，重点用于性能回归比较。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/simd-accel.mirror.junit.xml` | 镜像层行为测试执行结果。 |
| `migrations/simd-accel.legacy_to_mirror.json` | gtest/perftest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/simd_accel_mirror.rs` | SIMD 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── simd-accel.rust.junit.xml                  # 孪生层行为测试执行结果
│   └── simd-accel.parity.json                     # 镜像 vs 孪生 行为与性能对比报告
│
├── migrations/
│   └── simd-accel.mirror_to_rust.json             # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── simd_accel.rs                      # SIMD 孪生测试与基准源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| SIMD.ScanCopyUnescapedString_SSE42 | simd_accel_mirror::SIMD_ScanCopyUnescapedString_SSE42 | simd_accel | simd_accel::SIMD_ScanCopyUnescapedString_SSE42_rust | | |
| SIMD.ScanWriteUnescapedString_SSE42 | simd_accel_mirror::SIMD_ScanWriteUnescapedString_SSE42 | simd_accel | simd_accel::SIMD_ScanWriteUnescapedString_SSE42_rust | | |
| SIMD.SkipWhitespace_EncodedMemoryStream_SSE42 | simd_accel_mirror::SIMD_SkipWhitespace_EncodedMemoryStream_SSE42 | simd_accel | simd_accel::SIMD_SkipWhitespace_EncodedMemoryStream_SSE42_rust | | |
| SIMD.SkipWhitespace_SSE42 | simd_accel_mirror::SIMD_SkipWhitespace_SSE42 | simd_accel | simd_accel::SIMD_SkipWhitespace_SSE42_rust | | |#### 4.3 迁移策略建议

| 测试类别 | 进入 L2 优先级建议 | 原因 |
|----------|--------------------|------|
| 行为测试 | high | 行为一致是前提。 |
| 性能测试 | medium | 在行为验证完成后再逐步对齐性能。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/simd-accel.rust.junit.xml` | 孪生层行为测试执行结果。 |
| `reports/simd-accel.parity.json` | 镜像 vs 孪生 行为与性能对比报告。 |
| `migrations/simd-accel.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/simd_accel.rs` | SIMD 孪生测试与基准源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-007 `simd-accel` feature 级测试设计文档。 | `TBD` |
