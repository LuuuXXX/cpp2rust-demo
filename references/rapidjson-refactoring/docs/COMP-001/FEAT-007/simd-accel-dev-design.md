# Feature 级开发设计文档

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
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.1、2.4、4.1 simd 模块](../rapidjson-rs-dev-design.md#2-系统架构) |
| 是否商用代码 | `是` |
| 依赖使用策略 | `core/std-only`（禁止 crates.io 第三方依赖） |

---

## 快速导航

- [1. feature 概述](#1-feature-概述)
- [2. 功能需求](#2-功能需求)
- [3. 接口设计](#3-接口设计)
- [4. 数据结构](#4-数据结构)
- [5. 实现要点](#5-实现要点)
- [变更历史](#变更历史)

---

## 1. feature 概述

### 1.0 商用代码与依赖使用约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | SIMD 加速实现不得引入第三方 SIMD/向量计算库（如 `packed_simd` 等），仅可使用编译器内建、平台 intrinsics 或手写位运算实现。 |
| 对当前 feature 技术选型的影响 | 所有 SIMD 操作必须在 Rust 标准编译器支持范围内实现，注意平台特性检测与退化路径；禁止依赖 crates.io 提供的便捷封装。 |

### 1.1 feature 职责

**一句话描述:**
- `simd-accel` 为 `rapidjson-rs` 的关键路径提供可选的 SIMD 加速函数（如空白跳过、未转义字符串扫描等），在保持行为完全一致的前提下提升解析与生成性能。

**详细职责:**
- 为 `encoding-unicode` 和 `sax-io` 提供可选的 SIMD 优化路径，用于：
  - 空白字符跳过（SkipWhitespace）；
  - 未转义字符串扫描与写出（ScanCopy/ScanWriteUnescapedString 等）；
- 为数字处理等场景提供必要的位运算辅助（参考 `ieee754` 相关逻辑）；
- 在不支持 SIMD 或未启用加速时提供安全的标量回退路径；
- 提供统一的功能检测与选择机制，便于上层在运行时或编译期决定是否使用 SIMD 路径。

**不在职责范围内:**
- 不改变 JSON 解析/生成的语义与错误行为，只改变实现路径的性能特征；
- 不对 DOM 结构或 Schema 行为做任何额外修改；
- 不实现完整的向量数学库，仅覆盖 JSON 处理链路中已知热点。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::simd`，内部可按平台或功能细分子模块（如 `simd::x86`, `simd::arm`, `simd::whitespace` 等）。

### 1.3 重构策略

**重构策略:** `后置优化，渐进启用`。

- 仅在核心功能（encoding/sax/dom）行为稳定后逐步引入 SIMD；
- 每个 SIMD 优化点都必须有对应的标量实现作为基线，实现前后必须通过同一组测试，包括 perftest；
- 平台检测逻辑必须谨慎设计，保证在非目标平台或禁用 SIMD 时退化为标量路径，不影响正确性。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/internal/ieee754.h` | 浮点相关辅助，部分与 SIMD 协同 | 中 |
| `rapidjson_legacy/test/unittest/simdtest.cpp` | SIMD 相关行为测试 | 中 |
| `rapidjson_legacy/test/perftest/rapidjsontest.cpp` | SkipWhitespace/Scan* 等性能用例 | 高 |

> 注意：C++ 版本的 SIMD 逻辑可能分散在多个内部头文件或宏中，Rust 版本将按功能和平台重新组织。

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `SIMD-REQ-WHITESPACE-001` | SIMD 加速空白跳过 | P2 | 中 | [REQ-P-16, REQ-ST-09](../../requirements/requirements.md#2-1-json-解析parsing) | 待开发 |
| `SIMD-REQ-STRSCAN-001` | SIMD 加速未转义字符串扫描/写出 | P2 | 高 | [perftest Reader/Writer 用例](../../requirements/gtests.csv) | 待开发 |
| `SIMD-REQ-DETECT-001` | 平台特性检测与回退路径 | P1 | 中 | component 级性能目标 | 待开发 |

> SIMD 相关需求在原需求文档中主要以“加速白字符跳过”等性能要求出现，本 feature 将这些要求集中实现为可选优化路径。

### 2.2 功能详细规格

#### SIMD-REQ-WHITESPACE-001：SIMD 加速空白跳过

**输入:**
- 字节序列与当前解析位置；

**输出:**
- 第一个非空白字符的位置；

**处理逻辑要点:**
1. 在支持 SIMD 的平台上，使用向量指令一次检查多个字节是否为空白；
2. 在不支持 SIMD 时，使用标量循环作为回退；
3. 行为必须与标量实现完全一致，不允许漏判或错判。

#### SIMD-REQ-STRSCAN-001：SIMD 加速未转义字符串扫描/写出

**输入:**
- 字符串起始位置与长度；

**输出:**
- 第一次遇到需要特殊处理的位置（如引号、反斜杠、控制字符）；

**处理逻辑要点:**
1. 使用 SIMD 扫描字符串，快速定位潜在的需要特殊处理的字符；
2. 将扫描结果反馈给上层解析/生成逻辑，后者根据结果执行转义或结束字符串；
3. 需与 C++ perftest 中相关用例保持一致。

#### SIMD-REQ-DETECT-001：平台特性检测与回退路径

**输入/输出:**
- 在 crate 初始化或函数调用时检测当前平台是否支持特定 SIMD 指令集；

**处理逻辑要点:**
1. 优先使用 Rust 编译器内建的 cfg/target_feature 检测机制；
2. 在运行时不可用时退化为标量路径；
3. 不提供编译时强制要求某 SIMD 指令集，以保持跨平台兼容性。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 不提供外部 FFI 接口，仅作为内部优化模块。本节**无特殊设计**。

### 3.2 内部接口（Rust API 概要）

```rust
pub mod whitespace {
    pub fn skip(bytes: &[u8]) -> usize {
        // 返回首个非空白字符的索引；内部可根据平台选择 SIMD 或标量路径
        unimplemented!()
    }
}

pub mod strscan {
    pub fn scan_unescaped(bytes: &[u8]) -> usize {
        // 返回首个需要特殊处理的字符索引
        unimplemented!()
    }
}
```

> 以上仅为接口形态示意，实际实现细节由实现阶段的性能与安全分析决定。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-001` (`core-infra`) | 内部工具/平台检测（如 clzll 等） | 在某些 SIMD 优化中重用已有位运算工具 | [core-infra-dev-design.md](../FEAT-001/core-infra-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 说明 |
|----------|----------|------------------|------|------|
| `whitespace::skip` | internal | `encoding-unicode`, `sax-io` | 加速空白跳过 | 上层选择性调用 |
| `strscan::scan_unescaped` | internal | `sax-io` | 加速未转义字符串扫描 | 上层选择性调用 |

---

## 4. 数据结构

SIMD feature 更偏向算法与平台特性，不定义复杂持久化数据结构。本节**无特殊设计**。

---

## 5. 实现要点

实现要点包括：

- 仔细对齐 C++ SIMD 路径与标量路径行为，确保结果一致；
- 使用 cfg/target_feature 控制编译与运行路径，保持跨平台兼容；
- 在 perftest 场景中验证 SIMD 优化对性能的提升，并保证无明显退化。

详细算法与平台相关实现将在实现阶段展开，本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-007 `simd-accel` feature 级开发设计文档。 | `TBD` |
