# Feature 级开发设计文档

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
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.3、4.1 encoding 模块](../rapidjson-rs-dev-design.md#2-系统架构) |
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
| crates.io 使用策略 | 本 feature 不得使用任何第三方编码/Unicode 库（如 ICU、unicode-x 系列），所有编码探测、转码与验证逻辑必须依赖 `core`/`std` 与自研实现。 |
| 对当前 feature 技术选型的影响 | UTF-8/UTF-16/UTF-32 处理、BOM 检测、代理对处理、编码验证和转码 pipeline 需完整在 `rapidjson-rs` 内实现，不能通过 crates.io 扩展；高阶 Unicode 功能以满足需求文档为限，不扩展额外特性。 |

**约束说明:**
- 由于为商用代码且全局策略为 `core/std-only`，本 feature 不得引入任何外部 Unicode 库；必要的表驱动数据（如码点范围）应通过生成或手工维护的常量完成，并注意版权问题。

### 1.1 feature 职责

**一句话描述:**
- `encoding-unicode` 提供 `rapidjson-rs` 处理 JSON 文本时所需的编码与 Unicode 支持，包括 UTF-8/UTF-16/UTF-32 探测、转码、验证与基本字符属性判断，为 DOM、SAX、Schema、Pointer 等功能提供稳定、标准兼容的字符处理基础。

**详细职责:**
- 支持 UTF-8、UTF-16（LE/BE）、UTF-32（LE/BE）和 ASCII 编码的输入文本处理，与需求文档 2.7 一致。
- 提供 BOM 与内容特征结合的编码自动探测接口，并在探测失败时返回明确错误。
- 实现编码间内部转码逻辑（如 UTF-8 输入 → UTF-16 DOM 表示），隐藏不同编码下字符宽度差异。
- 提供编码验证接口：检测非法序列、非法代理对、超出允许范围的码点等。
- 提供基本 Unicode 支持：
  - 代理对处理；
  - 码点计数（如 `CountStringCodePoint` 行为）；
  - 必要的分类判断（视需求文档而定，如空白字符）。

**不在职责范围内:**
- 不实现完整 Unicode 分类/归一化/大小写折叠等高级功能，除非在需求文档中明确要求。
- 不直接处理 JSON 语法相关逻辑（如转义序列解析），但需为上层解析器提供必要的解码原语。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`。
- 预期 module 路径：`rapidjson_rs::encoding`，内部可细分若干子模块（示意）：
  - `encoding::utf8`
  - `encoding::utf16`
  - `encoding::utf32`
  - `encoding::ascii`
  - `encoding::detect`
  - `encoding::unicode`（与代理对/码点工具相关）

### 1.3 重构策略

**重构策略:** `完全 Rust 化 + 渐进收敛`。

- 目标是在 Rust 中重写编码与 Unicode 逻辑，使其行为与 C++ `encodings.h`/`unicode` 等价，并通过现有 gtest 用例验证正确性。
- 编码探测/转换/验证等算法以 C++ 实现为基线，逐步用 idiomatic Rust 实现，并以 `encodingstest.cpp`、`encodedstreamtest.cpp`、perftest 中 UTF8 验证场景为准进行收敛。

**技术选型:**
- 利用 `u8`/`char`/`u16`/`u32` 等基础类型表示不同编码单元；
- 使用 `core`/`std` 提供的最小必要 API（如 `char::from_u32`），避免直接依赖平台特定 API；
- 在必要时与 `core-infra` feature 的流与内存设施配合（如编码流包装）。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/encodings.h` | UTF-8/UTF-16/UTF-32/ASCII 编码 traits、转码模板、验证辅助 | 高 |
| `rapidjson_legacy/include/rapidjson/unicode.h`（如存在） | Unicode 相关工具（代理对、码点判断） | 中/高 |

**相关测试（主要引用）：**
- `rapidjson_legacy/test/unittest/encodingstest.cpp`
- `rapidjson_legacy/test/unittest/encodedstreamtest.cpp`
- `rapidjson_legacy/test/perftest/rapidjsontest.cpp` 中编码/UTF8 验证相关性能测试（如 `RapidJson.UTF8_Validate`）。

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `ENC-REQ-ENCSET-001` | 支持 UTF-8/UTF-16/UTF-32/ASCII 编码集合 | P0 | 中 | [需求文档 2.7 REQ-E-01, E-07](../../requirements/requirements.md#2-7-编码与-unicode) | 待开发 |
| `ENC-REQ-DETECT-001` | 自动检测输入流编码（BOM+内容特征） | P0 | 中 | [REQ-E-02](../../requirements/requirements.md#2-7-编码与-unicode) | 待开发 |
| `ENC-REQ-TRANSCODE-001` | 编码间内部转码 | P1 | 高 | [REQ-E-03](../../requirements/requirements.md#2-7-编码与-unicode) | 待开发 |
| `ENC-REQ-VALIDATE-001` | 编码验证（非法序列/代理对等） | P0 | 高 | [REQ-E-04, REQ-E-05](../../requirements/requirements.md#2-7-编码与-unicode) | 待开发 |
| `ENC-REQ-CUSTOM-001` | 自定义编码与字符类型扩展点 | P2 | 中 | [REQ-E-06](../../requirements/requirements.md#2-7-编码与-unicode) | 待开发 |

### 2.2 功能详细规格

#### ENC-REQ-ENCSET-001：支持 UTF-8/UTF-16/UTF-32/ASCII

**输入:**
- 字节序列（通常来自 `core-infra` 的流抽象），可能含 BOM。

**输出:**
- 对应编码的迭代接口或转换函数，使上层解析器可以按码点消费输入。

**处理逻辑:**
1. 根据 REQ-E-01 与 REQ-E-07，保证最少覆盖 UTF-8/UTF-16/UTF-32/ASCII。
2. 为每种编码提供统一 trait（例如 `Encoding`），定义常见操作：从字节读取码点、写入码点等。
3. 对于 UTF-16/UTF-32，显式处理大小端问题；调用方需提供端序或从 BOM/上文推断。

**异常场景:**
- 遇到非法字节序列时，应通过编码验证功能返回错误，而不是产生 UB 或错误码点。

#### ENC-REQ-DETECT-001：自动检测输入流编码

**输入:**
- 字节流起始部分，可能包含 BOM 或内容特征（例如 UTF-8 特征）。

**输出:**
- 一个编码枚举值（如 `EncodingKind::Utf8`），或错误（未知/冲突）。

**处理逻辑:**
1. 优先根据 BOM 识别编码。
2. 无 BOM 时，根据 UTF-8 的合法性和常见编码特征进行推断。
3. 尽量与 C++ 实现保持行为一致，以保证同一输入在两侧的编码选择相同。

**异常场景:**
- 无法确定编码时返回明确错误，交由上层决策（如假定 UTF-8 或报错中止）。

#### ENC-REQ-TRANSCODE-001：编码间内部转码

**输入:**
- 源编码标识、源字节序列。

**输出:**
- 目标编码表示（如内部统一编码或 DOM 所选编码）。

**处理逻辑:**
1. 提供从 UTF-8 等常见外部编码到内部统一编码（例如 UTF-16）的一步转码接口；
2. 转码过程中复用编码验证逻辑，确保不会产生非法码点；
3. 支持按需将内部编码转换为输出编码（用于 Writer）。

**异常场景:**
- 对于不能表示的码点，遵循 C++ 实现策略（如返回错误，或使用替代符号）。

#### ENC-REQ-VALIDATE-001：编码验证

**输入:**
- 字节序列与编码种类。

**输出:**
- 验证结果（成功/错误），并在错误时给出位置与错误类型。

**处理逻辑:**
1. 对 UTF-8，检测无效起始字节、续字节数错误、超范围码点等。
2. 对 UTF-16，检测无效代理对组合；对 UTF-32，检测无效码点范围。
3. 与 `encodingstest.cpp` 和性能测试中 UTF8_Validate 用例保持一致。

**异常场景:**
- 遇到非法序列时，不应继续解析而产出错误码点，应中断并返回错误。

#### ENC-REQ-CUSTOM-001：自定义编码与字符类型扩展点

**输入:**
- 用户提供的编码/字符类型实现。

**输出:**
- 通过 trait 或泛型参数扩展支持的编码集合，不修改核心逻辑。

**处理逻辑:**
1. 为编码相关 trait 预留泛型参数，支持用户自定义实现；
2. 不为自定义编码提供内建验证逻辑，仅提供挂接点。

**异常场景:**
- 自定义编码行为不在本 feature 测试范围内，需通过文档约束用户责任。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 也主要服务于 `rapidjson-rs` 内部，不直接暴露 FFI 接口。与编码相关的 FFI API 将由 FFI component 统一设计。本节**无特殊设计**。

### 3.2 内部接口（示意）

```rust
// 编码种类枚举
pub enum EncodingKind {
    Utf8,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
    Ascii,
}

// 编码 trait（示意）
pub trait Encoding {
    type Unit; // u8/u16/u32

    fn decode_next(input: &[Self::Unit]) -> Result<(char, usize), crate::error::Error>;
    fn encode_char(ch: char, output: &mut [Self::Unit]) -> Result<usize, crate::error::Error>;
}

// 编码探测
pub fn detect_encoding(bytes: &[u8]) -> Result<EncodingKind, crate::error::Error> {
    // 仅示意，不给出实现
    unimplemented!()
}
```

> 说明：以上仅为接口方向示例，具体关联 `core-infra` 错误类型与流接口的细节将在实现阶段补全。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| `FEAT-001` (`core-infra`) | 错误类型 `Error`、输入流 trait | 在解码/编码过程中返回统一错误类型，从流中读取字节 | [core-infra-dev-design.md 3.2 小节](./core-infra-dev-design.md) |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 函数签名（示意） |
|----------|----------|------------------|------|-------------------|
| `detect_encoding` | public | `sax-io`, `dom-core` | 解析前自动识别输入编码 | `pub fn detect_encoding(bytes: &[u8]) -> Result<EncodingKind, Error>` |
| 编码 trait `Encoding` | public | `sax-io`, `dom-core`, `schema-validate` | 统一处理不同编码的字符流 | `pub trait Encoding { .. }` |
| Unicode 工具（如 `count_code_points`） | public/internal | `dom-core`, `schema-validate` | 用于处理字符串长度/码点相关逻辑 | `pub fn count_code_points(s: &str) -> usize` |

**接口一致性说明:**
- 编码类型与工具必须与 component 级设计的 encoding 模块职责一致，不引入 JSON 语法知识。
- 上层 feature 通过 `EncodingKind` 与 `Encoding` trait 组合使用本 feature 功能，接口变更需评估兼容性。

---

## 4. 数据结构

### 4.1 类型定义（概念级）

**公共类型示意：**

```rust
pub enum EncodingKind {
    Utf8,
    Utf16Le,
    Utf16Be,
    Utf32Le,
    Utf32Be,
    Ascii,
}
```

```rust
pub struct Utf8Encoding;
pub struct Utf16LeEncoding;
pub struct Utf16BeEncoding;
// 等等
```

### 4.2 数据结构设计

本 feature 主要通过 trait 与函数组合实现，不设计复杂状态结构。内部可能使用少量辅助结构体（例如缓存 BOM 检测结果、状态机节点等），这些将在实现阶段细化。本节**无特殊设计**。

### 4.3 内存布局

不定义与 C 对齐相关的数据结构，本 feature 不直接参与 FFI 对齐问题。涉及 FFI 的编码结构将在 FFI feature 中单独建模。本节**无特殊设计**。

---

## 5. 实现要点

### 5.1 关键算法

**UTF-8 解码与验证:**
- 算法应能线性扫描字节序列，识别多字节序列的合法性，并在错误时返回位置与错误类型。
- 行为须与 `encodingstest.cpp`、`rapidjsontest.cpp` 中 UTF8 相关测试一致。

**UTF-16/UTF-32 代理对与码点计算:**
- 需正确处理高低代理对组合与非法代理范围。

**编码探测:**
- 按 BOM 优先，其次结合 UTF-8 特征；避免过度猜测其他编码。

实现阶段可参考 C++ `encodings.h` 但需用安全 Rust 重写，避免未定义行为。

### 5.2 错误处理

**错误处理策略:**
- 所有编码相关错误通过 `Error` 类型返回，不使用 panic；
- 将错误分类为“无效字节序列”“无效代理对”“未知编码”等，便于诊断。

### 5.3 性能优化

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| UTF-8 验证吞吐 | 接近 C++ `UTF8_Validate` 实现，同量级 | 使用 perftest 中 UTF8 相关场景比较 |
| 转码效率 | 对常见 JSON 文本负载保持与 C++ 同数量级 | 基于自定义基准或 perftest 扩展 |

**优化策略:**
- 在不违反安全前提下使用切片迭代与局部循环展开；
- 后续 `simd-accel` feature 可对热点路径做 SIMD 优化，本 feature 不直接引入。

### 5.4 内存管理

本 feature 主要使用调用方提供的缓冲区或 `core-infra` 的内存设施，不独立管理长期内存。本节**无特殊设计**。

### 5.5 并发安全

所有函数应为纯函数或仅依赖输入参数，不维护全局可变状态，天然适用于多线程环境。本节**无特殊设计**。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-002 `encoding-unicode` feature 级开发设计文档。 | `TBD` |
