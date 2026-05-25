# Feature 级测试设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-002` |
| feature 名称 | `encoding-unicode` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`encoding-unicode-dev-design.md` 开发设计文档](./encoding-unicode-dev-design.md) |
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
| crates.io 使用策略 | 本 feature 的测试不引入任何 crates.io Rust 依赖；编码相关特性验证全部依赖 legacy gtest、Rust 自测与必要的 Python 工具。 |
| 对当前 feature 技术选型的影响 | 所有编码与 Unicode 测试仅使用 gtest 与 `cargo test`，不使用第三方 fuzz/Unicode 框架；如需补充测试数据，优先采用 JSON 向量或自研脚本生成。 |

**约束说明:**
- 与 component 级策略一致，本 feature 在测试层面也不得引入第三方 Rust 编码测试库，仅可使用 C/C++ 和 Python 生态工具。

### 1.1 测试目标

**本 feature 测试重点:**
- 验证 `rapidjson-rs` 实现的编码与 Unicode 支持（UTF-8/UTF-16/UTF-32/ASCII）在行为上与 C++ 实现保持等价。
- 覆盖需求文档 2.7 中的各项需求：编码集合、自动检测、转码、验证、代理对、自定义编码扩展点等。
- 在 perftest 中与 C++ 实现进行性能对比，确保不出现数量级退化，特别是在 UTF-8 验证和转码场景。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | 编码检测、转码、验证、Unicode 工具函数 | 不涉及具体 JSON 语法规则（由其他 feature 测试） |
| 排除范围 | DOM/SAX/Schema 对编码的综合使用行为 | 在对应 feature 测试中覆盖 |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest 框架 | 执行与 encodings/encoded streams 相关的 legacy 测试用例 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy RapidJSON 与其测试二进制 | 与 legacy 项目配置一致 |
| `cargo test` | 执行 Rust 侧镜像与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 编写必要的辅助脚本（如编码向量生成、diff 报告） | Python 3 稳定版 |

### 1.3 测试环境

- C/C++ 编译器：`与 legacy RapidJSON 项目一致的编译器版本（如 gcc/clang/MSVC）`。
- 目标平台：`Linux/macOS/Windows 三平台优先，由 CI 保证一致性`。
- 构建与执行方式：沿用 `rapidjson_legacy` 中的 CMake 配置与 gtest 执行脚本；Rust 侧使用标准 `cargo test`。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 使用 `encodingstest.cpp`、`encodedstreamtest.cpp` 以及 perftest 中编码相关用例冻结 C++ 行为与性能特性。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，通过 FFI 调用 C++ 实现，验证 Rust 侧测试控制面。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 基于 `rapidjson-rs` 的编码实现编写纯 Rust 测试，与镜像测试并跑，验证行为与性能等价性。 | [4](#4-孪生测试层) |

测试断言来源遵循 component 级约束：`legacy_test`、`c_output`、`spec`、`invariant`、`metamorphic`。

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/
├── rapidjson_legacy/
│   └── test/
│       ├── unittest/
│       │   ├── encodingstest.cpp                # 编码单元测试
│       │   └── encodedstreamtest.cpp            # 编码流相关测试
│       └── perftest/
│           └── rapidjsontest.cpp                # 包含 UTF8_Validate 等性能测试
│
├── inventory/
│   └── encoding-unicode.legacy_tests.json       # 与编码相关的 Legacy 测试资产清单
│
├── baseline/
│   └── encoding-unicode.golden_samples.jsonl    # 编码相关黄金样本（输入/输出向量）
│
└── reports/
    ├── encoding-unicode.legacy.junit.xml        # Legacy 编码测试执行结果
    └── encoding-unicode.legacy.coverage.xml     # Legacy 编码测试覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| EncodedStreamTest.AutoUTFInputStream | test/unittest/encodedstreamtest.cpp:267 | legacy_test |
| EncodedStreamTest.AutoUTFOutputStream | test/unittest/encodedstreamtest.cpp:302 | legacy_test |
| EncodedStreamTest.EncodedInputStream | test/unittest/encodedstreamtest.cpp:254 | legacy_test |
| EncodedStreamTest.EncodedOutputStream | test/unittest/encodedstreamtest.cpp:289 | legacy_test |
| EncodingsTest.ASCII | test/unittest/encodingstest.cpp:428 | legacy_test |
| EncodingsTest.UTF16 | test/unittest/encodingstest.cpp:337 | legacy_test |
| EncodingsTest.UTF32 | test/unittest/encodingstest.cpp:397 | legacy_test |
| EncodingsTest.UTF8 | test/unittest/encodingstest.cpp:285 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/
│   └── encoding_ffi/                             # 编码相关 FFI 适配代码（规划）
│
├── migrations/
│   └── encoding-unicode.legacy_to_mirror.json    # gtest -> 镜像测试映射表
│
├── reports/
│   └── encoding-unicode.mirror.junit.xml         # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── encoding_unicode_mirror.rs        # 本 feature 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| EncodedStreamTest.AutoUTFInputStream | test/unittest/encodedstreamtest.cpp:267 | encoding_unicode_mirror::EncodedStreamTest_AutoUTFInputStream | |
| EncodedStreamTest.AutoUTFOutputStream | test/unittest/encodedstreamtest.cpp:302 | encoding_unicode_mirror::EncodedStreamTest_AutoUTFOutputStream | |
| EncodedStreamTest.EncodedInputStream | test/unittest/encodedstreamtest.cpp:254 | encoding_unicode_mirror::EncodedStreamTest_EncodedInputStream | |
| EncodedStreamTest.EncodedOutputStream | test/unittest/encodedstreamtest.cpp:289 | encoding_unicode_mirror::EncodedStreamTest_EncodedOutputStream | |
| EncodingsTest.ASCII | test/unittest/encodingstest.cpp:428 | encoding_unicode_mirror::EncodingsTest_ASCII | |
| EncodingsTest.UTF16 | test/unittest/encodingstest.cpp:337 | encoding_unicode_mirror::EncodingsTest_UTF16 | |
| EncodingsTest.UTF32 | test/unittest/encodingstest.cpp:397 | encoding_unicode_mirror::EncodingsTest_UTF32 | |
| EncodingsTest.UTF8 | test/unittest/encodingstest.cpp:285 | encoding_unicode_mirror::EncodingsTest_UTF8 | |#### 3.3 迁移策略建议

| 测试 ID | 进入 L1 优先级建议 | 原因 |
|---------|--------------------|------|
| `EncodingsTest.UTF8` | high | UTF-8 是最主要编码，解析与生成路径高度依赖。 |
| `EncodingsTest.UTF16`/`UTF32` | high | 影响跨平台与 DOM 内部表示，错误后果严重。 |
| `EncodingsTest.ASCII` | medium | 辅助场景，重要程度略低于 UTF-8。 |
| `EncodedStreamTest.*` | medium | 用于验证编码与流结合行为，可在编码核心稳定后迁移。 |
| `RapidJson.UTF8_Validate` | high | 直接关联性能与合法性检测，应尽早构建镜像测试以锁定基线。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/encoding-unicode.mirror.junit.xml` | 镜像层执行结果（Rust 控制、C++ 实现）。 |
| `migrations/encoding-unicode.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/encoding_unicode_mirror.rs` | 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── encoding-unicode.rust.junit.xml           # 孪生层执行结果（Rust 实现）
│   └── encoding-unicode.parity.json              # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── encoding-unicode.mirror_to_rust.json      # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── encoding_unicode.rs               # 本 feature 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| EncodedStreamTest.AutoUTFInputStream | encoding_unicode_mirror::EncodedStreamTest_AutoUTFInputStream | encoding_unicode | encoding_unicode::EncodedStreamTest_AutoUTFInputStream_rust | | |
| EncodedStreamTest.AutoUTFOutputStream | encoding_unicode_mirror::EncodedStreamTest_AutoUTFOutputStream | encoding_unicode | encoding_unicode::EncodedStreamTest_AutoUTFOutputStream_rust | | |
| EncodedStreamTest.EncodedInputStream | encoding_unicode_mirror::EncodedStreamTest_EncodedInputStream | encoding_unicode | encoding_unicode::EncodedStreamTest_EncodedInputStream_rust | | |
| EncodedStreamTest.EncodedOutputStream | encoding_unicode_mirror::EncodedStreamTest_EncodedOutputStream | encoding_unicode | encoding_unicode::EncodedStreamTest_EncodedOutputStream_rust | | |
| EncodingsTest.ASCII | encoding_unicode_mirror::EncodingsTest_ASCII | encoding_unicode | encoding_unicode::EncodingsTest_ASCII_rust | | |
| EncodingsTest.UTF16 | encoding_unicode_mirror::EncodingsTest_UTF16 | encoding_unicode | encoding_unicode::EncodingsTest_UTF16_rust | | |
| EncodingsTest.UTF32 | encoding_unicode_mirror::EncodingsTest_UTF32 | encoding_unicode | encoding_unicode::EncodingsTest_UTF32_rust | | |
| EncodingsTest.UTF8 | encoding_unicode_mirror::EncodingsTest_UTF8 | encoding_unicode | encoding_unicode::EncodingsTest_UTF8_rust | | |#### 4.3 迁移策略建议

| 测试 ID | 进入 L2 优先级建议 | 原因 |
|---------|--------------------|------|
| `EncodingsTest.UTF8` | high | 核心路径，须优先孪生以保证行为一致。 |
| `EncodingsTest.UTF16`/`UTF32` | high | 对内部表示和跨平台兼容性影响大。 |
| `EncodingsTest.ASCII` | medium | 辅助编码，优先级略低。 |
| `EncodedStreamTest.*` | medium | 依赖流与编码组合，适合在基础行为稳定后进行。 |
| `RapidJson.UTF8_Validate` | high | 性能与合法性关键指标，应成为孪生测试重点。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/encoding-unicode.rust.junit.xml` | 孪生层执行结果（Rust 实现编码与 Unicode 行为）。 |
| `reports/encoding-unicode.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/encoding-unicode.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/encoding_unicode.rs` | 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-002 `encoding-unicode` feature 级测试设计文档。 | `TBD` |
