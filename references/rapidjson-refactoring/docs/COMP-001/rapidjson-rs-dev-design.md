# component 级系统架构设计报告

## 报告基本信息

| 字段 | 内容 |
|------|------|
| component ID | `COMP-001` |
| component 名称 | `rapidjson-rs` |
| 所属系统/产品 | `RapidJSON Rust 重构（rapidjson-refactoring）` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| 对应需求文档 | [`docs/requirements/requirements.md`](../../docs/requirements/requirements.md) |
| 是否商用代码 | `是` |

---

## 快速导航

- [1. component 概述](#1-component-概述)
- [2. 系统架构](#2-系统架构)
- [3. crate 间交互与数据流设计](#3-crate-间交互与数据流设计)
- [4. module 划分](#4-module-划分)
- [变更历史](#变更历史)
- [附录 A：术语表](#附录-a术语表)

---

## 1. component 概述

### 1.1 项目背景

#### 1.1.1 项目起源

`rapidjson-rs` 是对 C++ 版本 RapidJSON 的 Rust 重构，实现等价的 JSON 解析/生成/DOM/SAX/Schema/Pointer 等能力，以满足在 Rust 生态中对高性能、低内存占用、跨平台 JSON 引擎的需求。项目的功能与非功能需求来源于 C++ 版 RapidJSON 的官方文档与行为，以及本仓库中的需求规格文档 `docs/requirements/requirements.md`。

#### 1.1.2 原项目概况

| 属性 | 值 |
|------|-----|
| **项目名称** | RapidJSON (C++) |
| **语言/技术栈** | C++17 及以下，模板元编程 + 手写内存管理，gtest 测试框架 |
| **代码规模** | 数万行级别，包含核心库、测试、性能基准等 |
| **核心功能** | JSON 解析/生成、DOM 操作、SAX 流式接口、JSON Pointer、JSON Schema 校验、Unicode/编码处理、流封装、内存池分配器、可选 SIMD 加速 |
| **构建系统** | CMake / 自定义构建脚本（依原 RapidJSON 项目） |

#### 1.1.3 重构目的

1. `purpose_1`：提供与 RapidJSON C++ 行为尽可能等价的 Rust 实现，在保证高性能和低内存占用的前提下，利用 Rust 的所有权和类型系统提升整体安全性。
2. `purpose_2`：在不引入 crates.io 第三方依赖的约束下，为后续 FFI crate（如 `rapidjson-ffi`）和上层业务 crate 提供稳定、清晰的 Rust API 基座。
3. `purpose_3`：为未来的测试与迁移策略提供可预期的接口和内部结构划分，使得 legacy gtest 能够通过 FFI 或镜像测试与 Rust 实现进行一一对比。

#### 1.1.4 重构范围

**包含在重构范围内:**
- `rapidjson-rs` crate 内的核心 JSON 功能：解析（SAX/DOM）、生成、DOM 值模型、内存管理、编码/Unicode 处理、流抽象、JSON Pointer、JSON Schema 校验等。
- 与上述能力直接相关的错误类型、配置类型、内部辅助工具（如内部 big integer、正则/模式匹配实现，但不得依赖 crates.io 正则库）。
- 支撑这些能力的公共抽象：如 `Reader`/`Writer` 风格 API、基于内存池的分配接口等。

**不包含在重构范围内:**
- FFI 层实现（如 `rapidjson-ffi` crate）、C 头文件封装及 C++ 适配层，仅在本文件中作为未来外部依赖方向进行说明，不做 module 级细节设计。
- 上层业务应用逻辑和特定产品场景，仅假定其作为 `rapidjson-rs` 的调用方。
- 用于性能基准、测试编排的独立工具或 crate，仅在 crate 间交互章节中作为潜在消费者提及。

---

### 1.2 架构目标

#### 1.2.1 核心目标

1. **`CORE-JSON-ENGINE`**: 提供一个与 C++ RapidJSON 行为等价、面向库型使用的 Rust JSON 引擎，覆盖 Parsing/Generation/DOM/SAX/Pointer/Schema/Encoding/Stream/Memory 管理等主要能力。

#### 1.2.2 质量目标

1. **`PERFORMANCE-PARITY`**: 在相同输入规模和场景下，解析与生成性能接近原 C++ 实现，满足需求文档 NFR-01 “解析速度可与 `strlen()` 相比”的目标，不引入明显的数量级性能退化。
2. **`MEMORY-EFFICIENCY`**: DOM 每个 Value 保持紧凑内存占用，支持内存池分配与预分配缓冲，满足 NFR-02 低内存目标。
3. **`SPEC-CONFORMANCE`**: 严格遵循 JSON 相关标准（RFC 7159/ECMA-404）和需求文档中的宽松语法扩展，满足 NFR-06 标准合规性要求。

#### 1.2.3 工程目标

1. **`WORKSPACE-INTEGRATION`**: 在已有 `rapidjson-refactoring` workspace 中，以 `rapidjson-rs` crate 的形式集成，遵循统一的目录结构和构建规范，为后续新增 crate 提供统一模板。
2. **`MIGRATION-FRIENDLY`**: 通过清晰的公共 API 和内部模块边界，降低后续引入 FFI crate 以及镜像测试/孪生测试编码成本，使 legacy gtest 能够稳定映射到 Rust 行为。

---

### 1.3 关键需求

#### 1.3.1 功能需求

| 需求 ID | 描述 | 优先级 | 来源 |
|---------|------|--------|------|
| **FR-001** | 支持与 RapidJSON C++ 等价的 JSON 解析与生成能力，包括 DOM/SAX API、In-Situ 解析、宽松语法（注释/尾逗号/NaN/Infinity/数字字符串）、多文档流、错误码与偏移信息。 | 高 | `docs/requirements/requirements.md` 第 2.1/2.2/2.4 节 |
| **FR-002** | 提供完整的 DOM 值模型和操作，包括 7 种 JSON 类型、类型查询、强类型数值接口、对象/数组增删改查、零拷贝字符串引用、深拷贝与值比较。 | 高 | 需求文档第 2.3 节 |
| **FR-003** | 支持 SAX 事件驱动解析与生成（Reader/Writer + Handler），包含各类事件回调及可选中间过滤层。 | 高 | 需求文档第 2.4 节 |
| **FR-004** | 提供 JSON Pointer 功能（获取/设置/创建/删除/交换），支持 URI Fragment 形式及错误报告。 | 中 | 需求文档第 2.5 节 |
| **FR-005** | 提供 JSON Schema Draft v4 校验能力，包括 Schema 编译、DOM/SAX 校验、错误报告、远程 Schema 引用等。 | 中 | 需求文档第 2.6 节 |
| **FR-006** | 支持 UTF-8/UTF-16/UTF-32 编码及自动探测、内部转码、编码验证、Unicode 代理对处理、自定义编码。 | 高 | 需求文档第 2.7 节 |
| **FR-007** | 提供内存与 IO 流抽象，包括字符串流、文件流、编码流、带游标的流包装器和自定义流接口。 | 高 | 需求文档第 2.8 节 |
| **FR-008** | 提供内存管理策略，包括内存池分配器、系统堆分配器、用户预分配缓冲区和自定义分配器扩展点。 | 高 | 需求文档第 2.9 节 |

**FR-001 详细规格（示例，用于代表同类需求的结构）：**
- **输入**: UTF-8/UTF-16/UTF-32 或 ASCII 编码的 JSON 文本（字符串或流），可选解析配置（是否启用宽松语法、数字字符串模式、全精度模式等）。
- **输出**: DOM 文档（Document）或 SAX 事件序列；在错误场景中返回错误码与偏移信息（不触发 panic）。
- **处理逻辑**: 按需求文档定义的语法和语义解析 JSON，支持 In-Situ 模式、迭代式解析、多文档流等特性；遇到错误时立即终止或按配置继续收集错误。
- **边界条件**:
  - 极大嵌套深度的 JSON（需防止栈溢出，提供迭代式解析路径）。
  - 含多种编码和 BOM 的输入（需遵守编码探测与错误报告要求）。
  - 宽松语法相关的多种错误组合（尾逗号、注释、不支持的数字格式等）。

#### 1.3.2 非功能需求

| 需求 ID | 指标 | 目标值 | 优先级 |
|---------|------|--------|--------|
| **NFR-001** | 解析性能 | 同平台同编译选项下，解析吞吐接近 C++ RapidJSON，满足“接近 `strlen()`”级别的时间复杂度 | 高 |
| **NFR-002** | 内存效率 | DOM 每 Value 的内存占用不显著高于 C++ 实现，支持内存池与预分配策略 | 高 |
| **NFR-003** | 跨平台支持 | 支持 Windows/Linux/macOS/iOS/Android 等主流平台，Rust MSRV 向后兼容 | 中 |
| **NFR-004** | 标准合规性 | 完全符合 RFC 7159/ECMA-404，兼容需求文档定义的宽松模式 | 高 |
| **NFR-005** | 线程安全 | 独立 Document/Reader/Writer 实例可在各自线程使用，只读结构可跨线程共享（Send+Sync） | 中 |

#### 1.3.3 约束条件

| 约束 ID | 类型 | 要求 | 理由 |
|---------|------|------|------|
| **TC-001** | 技术/依赖 | Rust 实现不得依赖 crates.io 第三方库，核心功能仅允许使用 `core`/`std`，测试和构建阶段也优先不引入第三方 Rust crate。 | 项目为商用代码，结合需求文档 NFR-05 “无外部依赖：核心功能不依赖第三方 crate”。 |
| **TC-002** | 接口/兼容 | 对外公开 API 行为需与 C++ RapidJSON 尽可能保持一致，便于未来 FFI 和业务迁移。 | 减少迁移成本，使现有 C++ 使用场景可以平滑转向 Rust/FFI。 |

#### 1.3.4 商用代码与依赖使用约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | 核心 crate `rapidjson-rs` 不得引入任何 crates.io 依赖；如需 SIMD/测试相关能力，应优先使用 Rust 标准工具链或 C/C++ 侧能力，而不是第三方 Rust crate。 |
| 对技术栈的影响 | 禁止使用常见的 JSON/序列化/正则第三方库（如 `serde_json`、`regex` 等），所有关键功能需手写或复用 C/C++ 侧逻辑；SIMD 优化需通过编译器内建或手写 intrinsics 完成。 |
| 对测试工具链的影响 | Rust 侧测试优先使用 `cargo test` + 标准库及内建宏，不新增第三方测试框架；需要的结构化对比、基准测试可复用 C++ gtest 输出或使用自研轻量逻辑。 |

**约束说明:**
- 由于 `{is_commercial_code} = 是`，本 component 设计在技术栈与依赖清单上必须遵守 `core/std-only` 策略，后续如需引入任何 crates.io 依赖，必须由人工明确评审与批准后，更新本章节。

---

### 1.4 component 设计范围说明

#### 1.4.1 系统类型

本 component 设计为**`通用软件库（跨平台 JSON 解析/生成引擎）`**，包含以下功能：
- 提供库型 API（Rust crate），供上层业务、工具或 FFI crate 调用，无独立长生命周期进程假设。
- 支持多种使用风格：以 DOM 为中心的对象操作，以及以 SAX 为中心的流式处理。
- 支持高性能场景（大 JSON、流式处理、内存受限环境）和常规应用场景。

**component 划分策略:**
- 本 component 对应 workspace 中的单一 crate：`rapidjson-rs`，采用“一对一” 映射。
- FFI crate（如 `rapidjson-ffi`）被视为单独 component，当前文档仅说明其与 `rapidjson-rs` 的预期交互边界，不展开其内部结构设计。
- 细粒度 Feature 与子模块拆分（例如针对 Pointer/Schema 等功能的 feature flag 与实现细节）将由后续 feature 级设计文档和实现阶段进一步细化。

#### 1.4.2 简化或省略的章节

根据项目特点，本 component 设计文档**不省略模板中的任何章节**，所有章节均给出高层设计或显式声明“无特殊设计”。本小节**无特殊设计**。

#### 1.4.3 设计重点

本项目的 component 设计重点为：
1. **`FOCUS-MEMORY-MODEL`**: 设计适配 Rust 所有权模型的 DOM/内存池/字符串引用策略，保证零拷贝友好与低内存占用，同时保持与 C++ 行为尽量一致。
2. **`FOCUS-ERROR-HANDLING`**: 设计统一的错误处理模式（错误码 + 结构化错误信息），避免 panic 作为跨边界错误机制，为未来 FFI 和测试提供稳定的错误语义。
3. **`FOCUS-MODULE-BOUNDARIES`**: 通过清晰的 module 划分与 DAG 依赖，限制内部耦合，便于后续按 feature 维度进行扩展与裁剪（如仅编译解析子集）。

---

## 2. 系统架构

### 2.1 整体架构图

#### 2.1.1 系统边界

```text
+-----------------------------------------------------------+
|                    调用方（外部系统）                     |
|  - 业务应用（Rust）                                      |
|  - 未来 FFI 层（rapidjson-ffi，C/C++ 调用）              |
|  - 基准测试/测试框架（Rust 或 C++）                      |
+----------------------------+------------------------------+
                             |
                             v
+-----------------------------------------------------------+
|                     rapidjson-rs component                |
|  - dom：DOM 值模型与 Document                            |
|  - sax：事件驱动解析/生成接口                            |
|  - pointer：JSON Pointer 操作                            |
|  - schema：JSON Schema 校验                              |
|  - encoding：编码与 Unicode 处理                         |
|  - stream：输入/输出流抽象                               |
|  - memory：内存池与分配策略                              |
|  - error：错误码与错误信息                               |
|  - simd（可选）：SIMD 加速点（不依赖 crates.io）         |
+-----------------------------------------------------------+
                             |
                             v
+-----------------------------------------------------------+
|                    底层平台与运行时                       |
|  - Rust core/std                                          |
|  - 操作系统文件/IO 接口                                  |
|  - CPU 指令集（可选 SIMD）                               |
+-----------------------------------------------------------+
```

#### 2.1.2 crate 级调用关系图

```text
+-----------------+        +-----------------+        +-----------------+
|  外部应用 crate |  -->   |   rapidjson-rs  |  -->   |   core / std    |
+-----------------+        +-----------------+        +-----------------+

（未来）
+-----------------+
|  rapidjson-ffi  |  -->  仅通过公开 API 使用 rapidjson-rs
+-----------------+
```

#### 2.1.3 Rust crate 结构

**本文档设计范围:**
- 本文档针对 Workspace 中的 **`rapidjson-rs`** crate 进行设计（对应下方目录结构中标注 `← 本文档设计对象` 的子目录）。
- 本次设计覆盖整个 `rapidjson-rs` crate，而非其中部分模块。

**component 与 crate 的映射关系:**
- 映射关系：`COMP-001` ↔ `rapidjson-rs`（一对一）。
- **说明**: 每个 component 代表一个可独立演化的 Rust crate。本项目中 `rapidjson-rs` 即核心 JSON 引擎 component，未来的 FFI 或工具 crate 将采用新的 component ID 与文档。

**Workspace 完整目录结构（规划）：**

```text
rapidjson-refactoring/                           # Workspace 根目录
├── Cargo.toml                                   # Workspace 级 Cargo.toml（[workspace] 配置）
├── README.md                                    # 项目总体说明
│
├── rapidjson_refactoring_legacy/               # 旧 C/C++ 工程；重构依赖；非 Rust 产物
│   └── ...
│
├── rapidjson_refactoring_sys/                  # 预留：C++ 工程 bindgen 胶水层目录
│   └── ...
│
├── inventory/                                  # 预留：测试资产清单目录
│   └── ...
│
├── baseline/                                   # 预留：Legacy 行为基线样本目录
│   └── ...
│
├── ci/                                         # 预留：CI 配置片段目录
│   └── ...
│
├── reports/                                    # 预留：测试报告汇总目录
│   └── ...
│
├── shared/                                     # 预留：跨 crate 共享测试资源目录
│   └── ...
│
├── schemas/                                    # 预留：测试输入/输出 schema 目录
│   └── ...
│
├── migrations/                                 # 预留：legacy -> Rust mirror 映射表目录
│   └── ...
│
├── rapidjson-rs/                               # ← 本文档设计对象
│   ├── Cargo.toml                              # crate 配置文件
│   ├── README.md                               # crate 说明文档
│   ├── src/
│   │   ├── lib.rs                              # lib crate 入口
│   │   └── ...                                 # 模块文件（dom/sax/schema/…）
│   ├── tests/
│   │   ├── mirrors/                            # 预留：镜像测试目录（与 legacy 行为对齐）
│   │   └── rust/                               # 预留：孪生/原生测试目录
│   ├── backends/                               # 预留：测试 backend 选择与说明
│   │   └── ...
│   └── logs/                                   # 运行时日志文件目录（测试/基准采集）
│
└── ...                                         # 其他 crate（由用户后续指定，本文件不展开）
```

**crate 列表（当前已知）：**

| crate ID | crate 名称 | 类型 | 入口文件 | Workspace 内路径 | 说明 |
|----------|-----------|------|----------|-----------------|------|
| `CRATE-rapidjson-rs` | `rapidjson-rs` | lib | `lib.rs` | `rapidjson-refactoring/rapidjson-rs/` | 核心 JSON 引擎，实现需求文档规定的各项功能，无 crates.io 依赖。 |

**要求:**
- `rapidjson-rs` crate 必须包含 `src/`、`tests/`、`backends/`、`logs/` 等子目录；其中 `tests/mirrors` 与 `tests/rust` 目录为测试防护网的 L1/L2 预留位置。
- 每个 crate 根目录必须有 `README.md` 和 `Cargo.toml` 文件。
- Workspace 根目录必须有 `Cargo.toml`（含 `[workspace]` 配置）和 `README.md`。
- Workspace 级共享目录（`inventory/`、`baseline/`、`ci/`、`reports/`、`shared/`、`schemas/`、`migrations/`）由所有 crate 共享，不得在单个 crate 内重复创建。

---

### 2.2 架构风格

#### 2.2.1 架构模式

**架构模式:** `库型架构 + 分层模块化 + 面向数据流的组件组合`

`rapidjson-rs` 作为纯库型 crate，不持有全局可变状态，以“数据流 + 配置”驱动行为。对外只暴露纯 Rust API，内部通过清晰模块边界实现 Parsing/DOM/SAX/Schema 等能力。

#### 2.2.2 分层设计

| 层次 | crate/module | 职责 | 依赖方向 |
|------|-----------|------|----------|
| API 层 | `rapidjson_rs::dom`, `rapidjson_rs::sax`, `rapidjson_rs::schema`, `rapidjson_rs::pointer` | 对外公开的高层 API，承载文档中的主要调用场景（DOM 操作、SAX 事件、Schema 校验、Pointer 访问）。 | 依赖核心层与基础层，不依赖外部 crate。 |
| 核心层 | `rapidjson_rs::encoding`, `rapidjson_rs::stream`, `rapidjson_rs::memory`, `rapidjson_rs::error`, `rapidjson_rs::simd` | 编码/流/内存/错误等通用能力，支撑 API 层实现。 | 依赖基础层，仅向上提供接口。 |
| 基础层 | `core`, `std` | 提供内存分配、字符串/集合、IO 等基础设施。 | 无上层依赖，作为最底层。 |

#### 2.2.3 选型理由

1. `rationale_1`: 通过库型 + 分层模式，将 JSON 解析/生成能力封装在 `rapidjson-rs` 内部，避免对业务层暴露过多实现细节，便于未来替换或裁剪实现。
2. `rationale_2`: 分层结构可直观支撑 non-functional 目标：性能相关优化集中在核心层（编码、内存、SIMD），API 层保持简单稳定，利于测试与 FFI。
3. `rationale_3`: 仅依赖 `core`/`std`，简化供应链与合规审计，符合商用项目对第三方依赖的限制。

#### 2.2.4 设计原则

1. **`NO-THIRD-PARTY-DEPS`**: 核心 crate 禁用 crates.io 依赖，所有功能依赖 `core`/`std` 与自研实现，降低法律与供应链风险。
2. **`NO-GLOBAL-MUTABLE-STATE`**: 不设计跨线程共享的可变全局状态，所有 Parser/Document/Writer 实例由调用方显式持有，显式传递可变借用。
3. **`DAG-MODULE-DEPENDENCY`**: module 间依赖结构保持有向无环，核心工具 module 不依赖高层 API，避免环状依赖和隐式耦合。
4. **`EXPLICIT-ERRORS`**: 统一使用错误码/结果类型表达错误，不以 panic 作为跨组件错误机制，panic 仅用于显然的编程错误（如调试断言）。

---

### 2.3 技术栈

#### 2.3.1 核心技术栈

| 层次 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 语言层 | Rust | 稳定版（按 workspace MSRV 要求） | 实现 `rapidjson-rs` 核心功能 |
| 标准库 | `core`, `std` | 与 Rust 版本一致 | 提供内存、集合、IO、错误基础设施 |
| C 运行时 | 平台 C runtime | N/A | 文件 IO、进程环境等（通过 `std` 间接使用） |

#### 2.3.2 构建工具链

| 工具 | 版本 | 用途 |
|------|------|------|
| `cargo` | 与 Rust 稳定版配套 | 构建与测试 `rapidjson-rs` |
| `rustc` | 与 Rust 稳定版配套 | 编译 `rapidjson-rs` |
| C/C++ 编译器 | 与 legacy 项目一致 | 构建 legacy RapidJSON + gtest（作为行为基线） |

#### 2.3.3 依赖清单（Workspace 级别）

**必需依赖:**

| 依赖 | 版本 | 类型 | 用途 | 使用 crate |
|------|------|------|------|-----------|
| `core` | 随编译器 | 语言运行时 | 提供无分配基础设施 | `rapidjson-rs` |
| `std` | 随编译器 | 标准库 | 字符串、集合、IO、错误处理 | `rapidjson-rs` |

**可选依赖（规划占位，当前不引入 crates.io）:**

| 依赖 | 版本 | 类型 | 用途 |
|------|------|------|------|
| 无 | N/A | N/A | 当前阶段无特殊设计，所有功能通过 `core`/`std` 与自研实现完成。 |

#### 2.3.4 平台支持

| 平台 | 架构 | 编译器 | 支持级别 |
|------|------|--------|----------|
| Linux | x86_64 | Rust stable + system C/C++ compiler | 完全支持（首要开发目标） |
| macOS | x86_64/arm64 | Rust stable + Xcode/clang | 完全支持 |
| Windows | x86_64 | Rust stable + MSVC/clang | 完全支持 |
| iOS/Android | arm64 | Rust stable + 平台交叉工具链 | 规划支持，需在实现阶段验证 |

---

### 2.4 架构风险与缓解

| 风险 ID | 风险描述 | 可能性 | 影响 | 缓解措施 |
|---------|----------|--------|------|----------|
| **RISK-001** | 禁止使用 crates.io 导致手写实现（如 JSON 解析、Schema、正则/模式匹配）工作量大且易出错。 | 中 | 高 | 在设计阶段明确最小必要功能与内部接口，优先沿用 C++ 算法与数据结构思路；通过全面的镜像测试与孪生测试验证行为一致。 |
| **RISK-002** | 性能无法达到 C++ RapidJSON 既有水平。 | 中 | 高 | 在解析/生成路径中预留 SIMD 与内存池优化点；通过基准测试与 flamegraph 反复迭代热点实现；在不影响 API 的前提下对内部表示进行优化。 |
| **RISK-003** | DOM/内存模型在 Rust 中难以同时满足零拷贝、可变更新与线程安全。 | 中 | 中 | 将 DOM 结构与分配策略分离，使用明确的生命周期和借用规则；在 module 级限制跨线程共享可变数据，仅允许只读共享。 |
| **RISK-004** | JSON Schema/Pointer 等高级功能实现复杂，易与 C++ 行为产生细微偏差。 | 中 | 中 | 先实现核心子集并对照 C++ gtest 和专用用例；为行为差异编写结构化文档，必要时在 API 文档中注明与 C++ 差异。 |

---

## 3. crate 间交互与数据流设计

### 3.1 crate 清单

当前已识别的 crate 清单：

| crate 名称 | 职责 | 类型 | 依赖 | 预估工时 |
|-----------|------|------|------|----------|
| `rapidjson-rs` | 实现 JSON 解析/生成/DOM/SAX/Pointer/Schema/Encoding/Stream/Memory 等核心能力，作为对外唯一 Rust 核心引擎。 | lib | 仅依赖 `core`/`std` | 高（完整重构周期内持续演进） |

> 说明：需求文档中的 FFI 层（`rapidjson-ffi`）在本 component 设计范围之外，仅作为未来 crate 参与交互，不在当前 crate 清单中。

### 3.2 crate 间依赖关系

```text
            +-----------------+
            |  外部应用 crate |
            +-----------------+
                     |
                     v
            +-----------------+
            |   rapidjson-rs  |
            +-----------------+
                     |
                     v
            +-----------------+
            |    core/std     |
            +-----------------+

未来扩展（不在本次实现范围内）：

    +-----------------+
    |  rapidjson-ffi  |  -->  仅通过公开 API 调用 rapidjson-rs
    +-----------------+
```

**依赖规则:**
- 当前 workspace 中，与本 component 直接相关的 crate 依赖结构为线性链：`外部应用/测试` → `rapidjson-rs` → `core/std`。
- `rapidjson-rs` 不依赖其他业务 crate 或工具 crate，确保可在多种项目中复用。
- 未来新增的 `rapidjson-ffi`、基准测试 crate 等必须只通过 `rapidjson-rs` 的公开 API 进行交互，不得依赖其内部模块实现细节。

### 3.3 crate 间接口关系

```text
外部应用 / future rapidjson-ffi
    |
    | 1. 通过 `rapidjson-rs` 提供的公共 API：
    |    - DOM：创建/解析/修改 Document/Value
    |    - SAX：注册 Handler，驱动解析/生成
    |    - Schema：编译 Schema，校验 JSON
    |    - Pointer：通过路径读取/写入/删除值
    v
rapidjson-rs
    |
    | 2. 仅通过 core/std 提供的能力与平台交互
    v
core / std / OS
```

### 3.4 跨 crate 数据传递

当前仅有 `rapidjson-rs` 这一核心 crate，实际运行时的“跨 crate”主要体现在：

- **外部应用 → rapidjson-rs**：
  - 传入 JSON 文本（字符串、字节切片或 IO 句柄包装），以及解析/生成配置。 
  - 传入自定义流类型（实现约定的 trait/接口）和内存分配策略对象。

- **rapidjson-rs → 外部应用**：
  - 返回 DOM 结构（Document/Value）或生成的 JSON 文本。
  - 返回错误码与详细错误信息结构（包含偏移、错误类型、上下文信息）。

**数据传递方式（规划）：**
- `{transfer_method_1}`: 使用 Rust 类型（所有权/借用）作为主要数据载体，如 `&str`、`&[u8]`、自定义 `Document`/`Value` 结构；不直接暴露裸指针或未初始化内存。
- `{transfer_method_2}`: 对于未来 FFI 交互，RapidJSON FFI crate 通过 C ABI 将 C/C++ 侧的缓冲区/指针包装为安全的 Rust 类型，再调用 `rapidjson-rs` API。本 component 仅在接口设计上为此预留序列化/反序列化友好的结构，不定义具体 FFI 类型。

**数据格式规范（规划）：**
- `{format_spec_1}`: JSON 文本严格遵循标准/宽松模式定义；错误信息使用结构化枚举 + 偏移量的形式表达，便于日志与测试 diff。
- `{format_spec_2}`: 对内部 Schema/Pointers 等结构使用 Rust 拥有的结构体与枚举，不强制使用额外的序列化格式（如 bincode），避免增加依赖；若未来需要持久化，可在 feature 级设计阶段引入。

### 3.5 系统级数据流图

```text
[输入 JSON 源]
    |  (字符串 / 字节流 / 文件句柄)
    v
[rapidjson-rs::stream]
    |  封装为统一输入流接口
    v
[rapidjson-rs::encoding]
    |  检测/转换编码，验证 Unicode
    v
[rapidjson-rs::sax 或 dom::parser]
    |  解析为事件流或 DOM
    v
[rapidjson-rs::dom / pointer / schema]
    |  DOM 操作 / Pointer 路径访问 / Schema 校验
    v
[rapidjson-rs::sax::writer / dom::serializer]
    |  序列化为 JSON 文本
    v
[输出 JSON / 错误信息]
```

### 3.6 关键数据流

| 流程名称 | 起点 | 终点 | 数据格式 | 处理逻辑 |
|----------|------|------|----------|----------|
| `DF-PARSE-DOM` | 调用方提供的 JSON 文本或流 | `Document`/`Value` DOM 结构 | 输入为 UTF-8/UTF-16/UTF-32 文本或字节流，输出为 DOM 结构 | 通过 `stream` 封装输入，`encoding` 负责编码识别与转换，解析逻辑在 `dom`/`sax` 层，遇到错误时由 `error` 统一返回错误码与偏移。 |
| `DF-SAX-PIPELINE` | 调用方构造的 Reader + Handler | 调用方自定义 Handler | 事件序列（Null/Bool/Number/String/Object/Array 及结构事件） | `sax` 解析器从 `stream` 拉取数据，经 `encoding` 解码后逐步触发 Handler 回调，支持中途终止与错误处理。 |
| `DF-SCHEMA-VALIDATION` | DOM/流 + Schema 文本 | 校验结果对象 | JSON 文本和 Schema 文本 | `schema` 编译 Schema 为内部文档，校验过程中重用 `dom`/`sax`/`pointer` 等模块，返回详细违规信息。 |
| `DF-POINTER-ACCESS` | 现有 DOM + Pointer 路径 | 目标值或修改后的 DOM | Pointer 字符串/内部结构 | `pointer` 模块解析 Pointer 字符串，沿 DOM 结构遍历并返回引用或执行修改，异常情况通过 `error` 返回错误码。 |

---

## 4. module 划分

### 4.1 module 清单

| module 命名空间路径 | 职责简要 | 预估工时 |
|----------------------|----------|----------|
| `rapidjson_rs::dom` | DOM 值模型与 Document 类型，提供树形 JSON 表示及增删改查、深拷贝、比较等操作 | 高 |
| `rapidjson_rs::sax` | SAX 风格解析与生成接口，定义 Handler trait/接口与 Reader/Writer 实现 | 高 |
| `rapidjson_rs::pointer` | JSON Pointer 解析与访问/修改/删除操作，封装 Pointer 内部表示 | 中 |
| `rapidjson_rs::schema` | JSON Schema 编译与校验逻辑，包含 SchemaDocument 与 Validator 核心算法 | 高 |
| `rapidjson_rs::encoding` | 字符编码与 Unicode 相关工具，包括 UTF-8/UTF-16/UTF-32/ASCII 支持与自动探测 | 中 |
| `rapidjson_rs::stream` | 输入/输出流抽象，包括内存字符串流、文件流、编码封装流等 | 中 |
| `rapidjson_rs::memory` | 内存池分配器、系统分配器适配、自定义分配器扩展点 | 中 |
| `rapidjson_rs::error` | 错误码定义、错误信息结构体与本地化占位能力 | 低 |
| `rapidjson_rs::simd` | SIMD 加速相关的内部工具模块（可选启用，基于编译器内建，不依赖第三方 crate） | 中 |
| `rapidjson_rs::internal` | 内部通用工具类型（如 big integer 运算、辅助宏/小工具），不对外暴露 | 中 |

### 4.2 module 划分原则

1. **`SINGLE-RESPONSIBILITY`**: 每个 module 聚焦单一领域（DOM、SAX、Schema 等），避免混合多种职责，降低理解与重构成本。
2. **`INTERNAL-ISOLATION`**: 通过 `internal` module 隔离实现细节（如 big integer、正则样式匹配），公开 API 仅依赖稳定的中间抽象，避免调用方绑定具体实现。
3. **`NO-CYCLE`**: module 依赖图为有向无环图，`internal`/`encoding`/`memory` 等基础模块只被上层引用，不依赖 API 层模块。
4. **`FFI-FRIENDLY-TYPES`**: 虽然本 component 不直接实现 FFI，但公开类型设计应尽量简单、`repr(Rust)` 即可稳定映射到 C 兼容封装类型，避免复杂的泛型/闭包跨 FFI 边界。

### 4.3 module 结构图

```text
rapidjson_rs
│
├── internal          # 通用内部工具（bigint、模式匹配等）
├── encoding          # 编码与 Unicode 工具
├── memory            # 内存分配策略
├── stream            # 输入/输出流抽象
├── error             # 错误类型与错误码
├── simd              # SIMD 加速点（可选）
│
├── dom               # DOM 值模型与 Document
├── sax               # SAX 解析/生成接口
├── pointer           # JSON Pointer 操作
└── schema            # JSON Schema 校验
```

### 4.4 module 间依赖关系

```text
          +-------------------+
          |   internal        |
          +-------------------+
                   ^
                   |
          +-------------------+
          | encoding          |
          +-------------------+
                   ^
                   |
   +--------+   +--------+   +--------+
   | memory |   | stream |   | error  |
   +--------+   +--------+   +--------+
        ^          ^            ^
        |          |            |
   +-----------------------------------+
   |   dom   |  sax  | pointer | schema|
   +-----------------------------------+
```

**依赖规则:**
- `internal` 不依赖其他业务 module，仅依赖 `core`/`std`。
- `encoding` 可以依赖 `internal`，用于复杂数据处理，但不依赖 DOM/SAX/Schema 等高层模块。
- `memory`/`stream`/`error` 仅依赖 `core`/`std` 和 `internal`/`encoding`，不依赖 `dom`/`sax`/`schema`。
- `dom`、`sax`、`pointer`、`schema` 可依赖上述基础模块，但相互之间应通过清晰接口交互，避免环状依赖。

### 4.5 module 间接口关系（宏观层面）

| 依赖方 module | 被依赖方 module | 接口名称（概要） | 用途说明 |
|-----------|-------------|----------------|----------|
| `dom` | `memory` | `allocate_node`, `deallocate_node` | DOM 在构建/修改 JSON 树时申请和释放节点内存。 |
| `dom` | `encoding` | `decode_string`, `encode_string` | DOM 在解析/序列化字符串值时执行编码转换与验证。 |
| `dom` | `error` | `DomError`, `ErrorCode` | DOM 操作过程中产生的错误统一通过错误类型返回。 |
| `sax` | `stream` | `read_token`, `peek_char` | SAX 解析器从统一流接口读取字节/字符。 |
| `sax` | `encoding` | `decode_next_char` | SAX 解析过程中对输入进行编码解码与验证。 |
| `sax` | `error` | `ParseError`, `ErrorCode` | 解析错误按统一错误模型返回调用方。 |
| `pointer` | `dom` | `get_value_at_path`, `set_value_at_path` | Pointer 查找和修改目标 DOM 节点。 |
| `schema` | `dom` | `validate_document`, `validate_value` | Schema 校验针对 DOM 树执行规则检查。 |
| `schema` | `pointer` | `resolve_pointer` | 在 Schema 校验过程中使用 Pointer 路径定位节点。 |
| `schema` | `error` | `SchemaError`, `ValidationError` | Schema 校验错误以统一结构返回。 |

> 说明：本表仅列出接口名称概要，用于体现 module 之间的责任分配与调用方向，具体函数签名和数据类型将在 feature 级设计中详细定义。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 `COMP-001` (`rapidjson-rs`) component 级系统架构设计文档，基于 `docs/requirements/requirements.md` 与 gtest 列表梳理 crate 与 module 结构。 | `TBD` |

## 附录 A：术语表

| 术语 | 定义 |
|------|------|
| DOM | Document Object Model，面向树形结构的 JSON 表示与操作方式。 |
| SAX | Simple API for XML/JSON，基于事件流的解析/生成接口风格。 |
| Pointer | JSON Pointer（RFC 6901），通过路径访问/修改 JSON 文档的标准表示法。 |
| Schema | JSON Schema，用于描述 JSON 文档结构和约束的规范。 |
| SIMD | Single Instruction Multiple Data，一种 CPU 指令集扩展，用于并行处理多个数据项。 |
| FFI | Foreign Function Interface，用于跨语言调用（如 C/C++ 与 Rust 互操作）的接口机制。 |
| MSRV | Minimum Supported Rust Version，本项目支持的最低 Rust 版本。 |
