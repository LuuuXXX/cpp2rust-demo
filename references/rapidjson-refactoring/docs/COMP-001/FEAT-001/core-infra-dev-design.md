# Feature 级开发设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-001` |
| feature 名称 | `core-infra` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-dev-design.md` 第 2.1.3、2.3、4.1 节](../rapidjson-rs-dev-design.md#2-系统架构) |
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
| crates.io 使用策略 | 核心 crate `rapidjson-rs` 不得引入任何 crates.io 依赖；如需 SIMD/测试相关能力，应优先使用 Rust 标准工具链或 C/C++ 侧能力，而不是第三方 Rust crate。 |
| 对当前 feature 技术选型的影响 | 本 feature 中的内存分配器、流抽象、错误类型和内部工具（如大整数、字符串/数值转换、正则样式匹配等）必须全部以自研实现或移植 C++ 算法为主，不得依赖 `regex`、`thiserror` 等第三方 crate。 |

**约束说明:**
- 本 feature 为所有上层功能提供基础设施，其依赖策略将直接影响后续所有 feature 的实现方式，因此必须严格遵守 `core/std-only`，避免在底层引入任何第三方依赖导致锁定整个系统。

### 1.1 feature 职责

**一句话描述:**
- `core-infra` 提供 `rapidjson-rs` 的基础运行时支撑，包括错误模型、内存分配策略、流抽象以及内部数值/字符串运算工具，为 DOM、SAX、Pointer、Schema 等上层功能提供稳定且高性能的公共底座。

**详细职责:**
- 定义统一的错误枚举与错误信息结构，并为上层模块提供一致的错误返回接口。
- 提供抽象的内存分配接口及实现：包括系统堆分配器、内存池分配器和预分配缓冲支持，满足需求文档 2.9 内存管理相关要求。
- 提供输入/输出流抽象：字符串流、文件流、编码流包装器以及带游标的流封装器，满足需求文档 2.8 流相关要求。
- 提供内部通用工具模块：
  - 大整数运算（biginteger）用于精度较高的数值处理；
  - 字符串与数值转换（itoa/dtoa/strtod 等）；
  - 基础正则/模式匹配工具（regex 内部实现）用于 Schema 等高层功能；
  - 常用的元编程/类型工具（meta）、栈封装（stack）、位操作（clzll）等。

**不在职责范围内:**
- 不负责任何 JSON 语义相关逻辑（解析、生成、DOM 操作、Schema 规则等），这些由其他 feature 实现。
- 不直接对外暴露 Rust API 以供业务调用，其对外接口主要供 `rapidjson-rs` 内部 module 和未来 FFI 层使用。
- 不设计任何并发 primitives（锁等），并发语义由上层调用方控制，本 feature 仅保证类型的 `Send`/`Sync` 安全前提。

### 1.2 项目中的位置

- 所属 crate：`rapidjson-rs`
- 预期 module 路径：
  - `rapidjson_rs::error`
  - `rapidjson_rs::memory`
  - `rapidjson_rs::stream`
  - `rapidjson_rs::internal`（子模块：`biginteger`, `itoa`, `dtoa`, `strtod`, `regex`, `stack`, `meta`, `clzll` 等）

### 1.3 重构策略

**重构策略:** `渐进式 + 完全 Rust 化`

- 首先在 Rust 中定义与 C++ 等价的抽象边界（错误类型、分配器接口、流抽象），以最小实现支撑上层 feature 的编译与基础测试。
- 随着上层 DOM/SAX/Schema 等行为逐步迁移到 Rust，实现内部算法（biginteger、dtoa/strtod、regex 等）的完整移植和优化，对照 C++ 单元测试与性能基准逐步收敛行为差异。

**技术选型:**
- 关键依赖：`core`, `std`（`alloc` 可选，仅在确有必要时评估）。
- 依赖限制：
  - 不可引入 `regex`、`num-bigint`、`thiserror` 等 crates.io 包。
  - 错误类型采用手写枚举 + Display/Debug 实现；内部算法直接移植 C++ 逻辑或重写为 Rust。

**C/C++ 代码对应部分:**

| 源文件 | 范围 | 复杂度 |
|-------|-----|-------|
| `rapidjson_legacy/include/rapidjson/allocators.h` | 内存池分配器、CRT 分配器接口与实现 | 中 |
| `rapidjson_legacy/include/rapidjson/stream.h` | 内存/文件流、编码流包装器、带游标流等 | 中 |
| `rapidjson_legacy/include/rapidjson/internal/biginteger.h` | 大整数运算实现 | 高 |
| `rapidjson_legacy/include/rapidjson/internal/itoa.h` | 整数转字符串 | 中 |
| `rapidjson_legacy/include/rapidjson/internal/dtoa.h` | 浮点数转字符串 | 高 |
| `rapidjson_legacy/include/rapidjson/internal/strtod.h` | 字符串转浮点数 | 高 |
| `rapidjson_legacy/include/rapidjson/internal/strfunc.h` | 字符串工具函数 | 中 |
| `rapidjson_legacy/include/rapidjson/internal/regex.h` | 简化正则引擎实现 | 高 |
| `rapidjson_legacy/include/rapidjson/internal/stack.h` | 内部栈封装 | 中 |
| `rapidjson_legacy/include/rapidjson/internal/clzll.h` | 位操作辅助 | 低 |
| `rapidjson_legacy/include/rapidjson/internal/meta.h` | 模板元编程/类型工具 | 中 |

**相关测试（优先适配至本 feature）：**
- `rapidjson_legacy/test/unittest/allocatorstest.cpp`
- `rapidjson_legacy/test/unittest/filestreamtest.cpp`
- `rapidjson_legacy/test/unittest/istreamwrappertest.cpp`
- `rapidjson_legacy/test/unittest/ostreamwrappertest.cpp`
- `rapidjson_legacy/test/unittest/bigintegertest.cpp`
- `rapidjson_legacy/test/unittest/itoatest.cpp`
- `rapidjson_legacy/test/unittest/dtoatest.cpp`
- `rapidjson_legacy/test/unittest/strfunctest.cpp`
- `rapidjson_legacy/test/unittest/strtodtest.cpp`
- `rapidjson_legacy/test/unittest/regextest.cpp`
- `rapidjson_legacy/test/unittest/clzlltest.cpp`

---

## 2. 功能需求

### 2.1 功能清单

| 需求 ID | 功能名称 | 优先级 | 复杂度 | component 开发设计文档追溯 | 状态 |
|---------|----------|--------|--------|-----------------------------|------|
| `COREINF-REQ-ERR-001` | 统一错误类型与错误码 | P0 | 中 | [1.3.1, 2.3, 4.1 error 模块](../rapidjson-rs-dev-design.md#1-关键需求) | 待开发 |
| `COREINF-REQ-MEM-001` | 内存池分配器与系统分配器封装 | P0 | 高 | [2.3, 4.1 memory 模块](../rapidjson-rs-dev-design.md#4-module-划分) | 待开发 |
| `COREINF-REQ-STRM-001` | 内存/文件/编码流抽象 | P0 | 中 | [2.3, 3.4, 4.1 stream 模块](../rapidjson-rs-dev-design.md#3-crate-间交互与数据流设计) | 待开发 |
| `COREINF-REQ-INTERNAL-001` | 内部工具（biginteger/itoa/dtoa/strtod/regex/stack 等） | P1 | 高 | [4.1 internal 模块](../rapidjson-rs-dev-design.md#4-module-划分) | 待开发 |

### 2.2 功能详细规格

#### COREINF-REQ-ERR-001：统一错误类型与错误码

**输入:**
- 来自上层模块的错误场景描述（解析错误、编码错误、IO 错误、内存耗尽等）。

**输出:**
- 一个统一的错误类型（如 `Error` 枚举）或包裹该枚举的结果类型（`Result<T, Error>`）。

**处理逻辑（要点）:**
1. 按错误来源维度划分：解析、编码、IO、内存、内部不变量违背等。
2. 为每类错误提供结构化信息字段（如位置偏移、上下文描述），但在本 feature 中仅定义类型，不限定具体产生逻辑。
3. 保证错误类型 `Send + Sync`，可跨线程传递；禁止在错误类型中携带非线程安全引用或裸指针。

**异常场景:**
- 如果上层试图使用 panic 作为跨边界错误机制，应在设计中明确禁止，要求转换为 `Error` 返回。

#### COREINF-REQ-MEM-001：内存池分配器与系统分配器封装

**输入:**
- 上层请求分配/释放内存的大小、对齐要求和策略选择（系统分配器/内存池/预分配缓冲）。

**输出:**
- 表示已分配内存区域的指针或安全包装类型；在释放时回收到对应分配器。

**处理逻辑（要点）:**
1. 定义统一的分配器 trait（仅表述接口，本文不写具体实现代码）。
2. 提供至少两种实现：
   - 系统分配器：包装 `std` 堆分配能力；
   - 内存池分配器：顺序分配，不支持单独释放，仅整体重置。
3. 预留接口支持用户提供预分配缓冲区，避免堆分配（用于嵌入式/高性能场景）。

**异常场景:**
- 分配失败时返回错误而不是 panic；需要为上层提供可检测的错误码。

#### COREINF-REQ-STRM-001：内存/文件/编码流抽象

**输入:**
- 字符串缓冲区、文件句柄或用户实现的流对象。

**输出:**
- 满足 Reader/Writer 需求的统一流接口（如 `peek/take/tell` 与 `put/flush` 抽象）。

**处理逻辑（要点）:**
1. 定义抽象流接口 trait，支持只读/只写/读写模式，接口风格与 C++ `Stream` 概念兼容。
2. 提供：
   - 内存字符串流实现；
   - 基于 `std::fs` 的文件流实现；
   - 编码流包装器，由 `encoding-unicode` feature 完成编码转换。
3. 为带游标流提供位置跟踪能力，便于错误偏移计算。

**异常场景:**
- IO 错误必须转换为统一 `Error` 类型中的 IO 变体，不在流层 panic。

#### COREINF-REQ-INTERNAL-001：内部工具实现

**输入/输出:**
- 输入为内部算法所需的基本类型（整数、浮点数、字符串缓冲等）；输出为转换结果或匹配结果。

**处理逻辑（要点）:**
1. 按子模块细分职责：
   - `biginteger`：提供用于高精度转换和 Schema 校验的内部大整数；
   - `itoa`/`dtoa`：提供整数/浮点转字符串的高性能实现；
   - `strtod`：提供字符串转浮点的解析算法；
   - `regex`：提供简化正则引擎，用于 Schema `pattern` 等；
   - `stack`：提供带自动扩展能力的内部栈结构。
2. 所有实现必须兼顾性能与安全，不使用未初始化内存或未检查的指针操作。

**异常场景:**
- 对于非法输入（如无效浮点表示、非法正则表达式），必须返回错误而非未定义行为；详细错误类型可以由上层包装。

---

## 3. 接口设计

### 3.1 外部接口（extern "C"）

本 feature 主要服务于 `rapidjson-rs` 内部，不直接定义对外 FFI 接口。外部 FFI 接口将由 `rapidjson-ffi` component 统一设计。本节**无特殊设计**。

### 3.2 内部接口

> 以下仅给出代表性函数签名示意，实际签名将在实现阶段根据需要细化；本节不包含任何函数体或实现代码。

```rust
// 统一错误类型
pub enum Error {
    Parse,
    Encode,
    Io,
    Memory,
    Internal,
}

// 分配器抽象
pub trait Allocator {
    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8>;
    fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize);
}

// 流抽象（只读示意）
pub trait InputStream {
    fn peek(&self) -> Option<u8>;
    fn take(&mut self) -> Option<u8>;
    fn tell(&self) -> usize;
}
```

> 说明：上述签名仅用于说明接口设计方向，具体枚举变体与 trait 方法集合将在实现阶段结合上层需求细化。

### 3.3 与其他 feature 的接口关系

**使用其他 feature 的接口:**

| feature ID | 接口名称 | 用途 | feature 开发设计文档追溯 |
|------------|----------|------|--------------------------|
| 无 | 无 | `core-infra` 为基础设施层，本身不依赖高层 feature。 | 无 |

**暴露给其他 feature 的接口（宏观）：**

| 接口名称 | 接口类型 | 暴露给的 feature | 用途 | 函数签名（示意） |
|----------|----------|------------------|------|-------------------|
| 错误类型 `Error` | public | 全部 | 统一错误返回类型 | `pub enum Error { .. }` |
| 分配器 trait `Allocator` | public | `dom-core`, `schema-validate` 等 | 为 DOM/Schema 提供统一内存管理接口 | `pub trait Allocator { .. }` |
| 流 traits `InputStream`/`OutputStream` | public | `sax-io`, `dom-core` | 为解析/生成与 DOM 序列化提供统一流接口 | `pub trait InputStream { .. }` |
| 内部工具模块（如 `internal::biginteger`） | internal | `dom-core`, `schema-validate` | 为数值转换和 Schema 校验提供内部算法支持 | `mod internal::biginteger` |

**接口一致性说明:**
- 上层 feature 仅通过 public trait/类型与本 feature 交互，不直接依赖内部实现细节。
- 任何对 public 接口的变更必须评估对依赖 feature 的影响，并在相应 feature 设计文档中同步更新。

---

## 4. 数据结构

### 4.1 类型定义（概念级）

**公共类型（示意）：**

```rust
pub enum Error {
    Parse { message: &'static str },
    Encode { message: &'static str },
    Io,
    Memory,
    Internal,
}
```

```rust
pub struct MemoryPoolAllocator {
    // 内存池元数据字段略
}
```

```rust
pub struct StringInputStream<'a> {
    // 基于 &str 或 &[u8] 的游标与长度
    // 具体字段在实现阶段确定
    _phantom: core::marker::PhantomData<&'a ()>,
}
```

> 以上为结构设计方向示意，具体字段和布局将在实现阶段结合性能与安全需求确定。本节不展开完整定义。

### 4.2 数据结构设计（描述性）

| 字段/变量 | 类型 | 用途 |
|-----------|------|------|
| `MemoryPoolAllocator.pool` | 原始缓冲区指针或切片 | 存放内存池分配得到的对象内存。 |
| `MemoryPoolAllocator.capacity` | `usize` | 分配器当前可用容量。 |
| `MemoryPoolAllocator.size` | `usize` | 已使用容量，用于顺序分配。 |
| `StringInputStream.cursor` | `usize` | 当前读取位置，用于 `peek`/`take`。 |

### 4.3 内存布局（如需要）

**布局对齐:** 内部结构体遵循 Rust 默认对齐规则；如需与 C 结构对齐，将在后续 FFI 设计中单独定义 `repr(C)` 结构，本 feature **无特殊设计**。

---

## 5. 实现要点

### 5.1 关键算法

本 feature 的关键算法包括（仅列出名称及来源，不给出实现代码）：

- 大整数运算（`internal::biginteger`）：用于高精度数值转换。
- 浮点数转字符串（`internal::dtoa`）：用于 Writer 输出；复杂度接近 C++ 实现。
- 字符串转浮点数（`internal::strtod`）：用于解析器；需对照 `strtodtest.cpp` 保证精度与边界行为。
- 简化正则引擎（`internal::regex`）：用于 Schema `pattern` 匹配，需与 `regextest.cpp` 行为对齐。

实现阶段应按如下原则移植：

```text
- 优先直接翻译核心算法思路，保持行为一致；
- 使用安全 Rust 表达内部状态，避免裸指针与未初始化内存；
- 使用局部注释记录与 C++ 实现的关键差异（如泛型替代、迭代器使用）。
```

### 5.2 错误处理

**错误处理策略:**
- 所有对外暴露的基础设施接口使用 `Result<T, Error>` 或等价形式返回错误，不以 panic 作为错误通路。
- 内部逻辑中仅在明显编程错误（如 `debug_assert!` 场景）使用 panic，且不跨 crate 边界传播。

**错误传播:**
- 本 feature 内部：
  - 算法内部错误应尽早转换为 `Error::Internal` 或更具体的错误变体。
- 跨 feature：
  - 上层 DOM/SAX/Schema 等功能收到 `Error` 后可再包装为更具体的错误类型，但不得丢失原始错误原因。

### 5.3 性能优化

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| 内存分配开销 | 相比 C++ 内存池实现不出现数量级退化 | 基于 `allocatorstest.cpp` 对应的 Rust 镜像测试和基准测试 |
| 字符串/数值转换吞吐 | 与 C++ dtoa/itoa/strtod 实现同量级 | 使用 `dtoatest.cpp`、`itoatest.cpp`、`strtodtest.cpp` 场景对比 |

**优化策略:**
- 使用顺序分配、批量增长策略减少分配次数；
- 使用堆栈缓冲和预分配缓冲减少堆分配；
- 在不引入 unsafe 的前提下尽量利用切片与迭代器优化访问。

### 5.4 内存管理（本 feature 特有）

**内存分配策略:**
- 默认使用内存池分配器，适用于 JSON DOM 大量小对象的分配；
- 提供系统分配器封装，以兼容非池化场景或测试用途；
- 为嵌入式/高性能场景预留固定缓冲区分配策略接口。

**生命周期管理:**
- 内存池分配器的生命周期由上层持有，通常与 Document 或解析上下文绑定；
- 禁止跨分配器移动对象的所有权（避免双重释放或泄漏）；
- 内部数据结构避免持有短生命周期引用，优先使用索引或偏移表示内部关系。

### 5.5 并发安全

**并发模型:**
- 本 feature 不引入内部锁或线程；并发模型为“多实例多线程”，即每个线程使用独立分配器/流实例。

**线程安全保证:**
- 分配器、流等结构缺省不实现 `Sync`，仅在严格证明线程安全的前提下提供 `Send` 标记；
- 错误类型 `Error` 应实现 `Send + Sync`，便于跨线程传递。

**同步机制:**
- 无特殊设计；若上层需要共享基础设施实例，应由上层显式使用同步原语（如 `Mutex`），本 feature 不内建。

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-001 `core-infra` feature 级开发设计文档，定义职责边界、功能需求、接口关系与实现要点。 | `TBD` |
