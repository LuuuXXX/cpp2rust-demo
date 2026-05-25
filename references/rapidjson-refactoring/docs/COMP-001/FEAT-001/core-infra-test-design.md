# Feature 级测试设计文档

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
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | [`core-infra-dev-design.md` 开发设计文档](./core-infra-dev-design.md) |
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
| crates.io 使用策略 | 核心 crate `rapidjson-rs` 在测试中不引入任何 crates.io 依赖；Python 脚本与 C/C++ 测试框架按各自许可证使用。 |
| 对当前 feature 技术选型的影响 | 本 feature 的测试框架仅使用 gtest（C++）与 `cargo test`（Rust），不使用 Rust 第三方测试/基准库；任何数据生成、diff 等逻辑如需辅助，优先采用 C++/Python/自研脚本实现。 |

**约束说明:**
- 所有与 `core-infra` 相关的 Rust 测试用例应位于 `rapidjson-rs` 自身或独立测试 crate 中，且不依赖 crates.io 第三方库。

### 1.1 测试目标

**本 feature 测试重点:**
- 验证内存分配器（系统分配器、内存池分配器）的行为与 C++ 实现一致，包括对齐、溢出处理、重置策略等。
- 验证流抽象（内存流、文件流、输入/输出包装器）在读取、写入、游标管理等方面与 C++ `Stream` 行为一致。
- 验证内部工具（大整数、字符串/数值转换、正则引擎等）的结果与 C++ 单元测试基线一致，特别是边界和异常输入场景。

| 范围类别 | 内容说明 | 备注 |
|----------|----------|------|
| 包含范围 | 内存分配器、流抽象、内部工具模块的行为与性能回归 | 仅功能和性能层面，不涉及 DOM/SAX 语义 |
| 排除范围 | JSON 语义相关测试（解析、生成、DOM 操作） | 由其他 feature 测试承担 |

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest 框架 | 执行与 allocators/streams/internal 工具相关的 legacy 测试用例，形成行为基线 | 与 legacy RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy RapidJSON 与其测试二进制 | 与 legacy 项目配置一致 |
| `cargo test` | 执行 Rust 侧镜像测试与孪生测试 | 与 workspace Rust 版本一致 |
| Python 3 | 编写必要的辅助脚本（如基线结果收集、diff 报告） | Python 3 稳定版 |

### 1.3 测试环境

- C/C++ 编译器：`与 legacy RapidJSON 项目一致的编译器版本（如 gcc/clang/MSVC），用于构建原始 C++ 代码与 gtest 测试`。
- 目标平台：`Linux/macOS/Windows 三平台为首要 CI 目标，必要时扩展至移动平台`。
- C/C++ Legacy 项目构建方式：`沿用 rapidjson_legacy 中原有 CMake/编译脚本`。
- C/C++ Legacy 项目测试环境：`通过 gtest 执行 allocator/stream/internal 相关测试，输出测试结果与覆盖率报告`。
- C/C++ Legacy 项目覆盖率环境：`可选地整合 gcov/llvm-cov 以评估核心基础设施模块覆盖率`。

### 1.4 三层测试防护网摘要

| 层级 | 层级名称 | 当前 feature 覆盖说明 | 关联章节 |
|------|----------|-------------------|----------|
| L0 | 基线层 | 选取 allocator/stream/internal 工具相关的 C++ gtest 用例，形成行为与性能基线。 | [2](#2-基线层) |
| L1 | 镜像测试层 | 为上述 gtest 编写 Rust 镜像测试，借助 FFI 调用 C++ 实现，验证 Rust 侧测试控制面行为一致。 | [3](#3-镜像测试层) |
| L2 | 孪生测试层 | 在 Rust 重构实现稳定后，为基础设施模块编写纯 Rust 孪生测试，与镜像测试并跑验证等价性。 | [4](#4-孪生测试层) |

测试防护网由三层构成，按时间顺序推进：

- **L0 基线层**：冻结旧 C/C++ 行为，保留旧测试，建立可对比基线。
- **L1 镜像测试层**：对原有 legacy gtest 编写 1:1 的 Rust 镜像测试，由 Rust 接管测试控制面；这些镜像测试通过 FFI 复现 C/C++ 测试逻辑，以验证 Rust 测试控制面与 legacy 基线一致。
- **L2 孪生测试层**：基于 L1 已有的镜像测试，编写基于 Rust 重构代码的孪生测试（原生测试），两个测试逻辑等价，验证镜像测试与孪生测试行为等价。

测试断言只允许以下来源（`oracle_source`）：

1. `legacy_test`：旧 C/C++ 测试中的断言。
2. `c_output`：旧 C/C++ 实现对同一输入的实际输出。
3. `spec`：公开规格、协议文档、头文件契约或明确注释。
4. `invariant`：数学不变量、代数性质、状态机约束。
5. `metamorphic`：变形关系，例如编解码往返等价等。

**UT/IT 与测试切面:**
- `test_type`：
  - UT：单模块测试（如单个分配器、单个流类型、单个内部工具函数）。
  - IT：跨模块协作测试（如流与编码结合、内存池与 DOM 共同使用）。对 `core-infra` 而言，重点是 UT，IT 主要在其他 feature 测试中体现。

---

## 2. 基线层

### 2.1 基线层结构

```text
rapidjson-refactoring/                           # Workspace 根目录
├── rapidjson_legacy/                            # 旧 C/C++ 工程
│   └── test/unittest/                           # gtest 单元测试
│       ├── allocatorstest.cpp                   # 分配器相关测试
│       ├── filestreamtest.cpp                   # 文件流测试
│       ├── istreamwrappertest.cpp               # 输入流包装器
│       ├── ostreamwrappertest.cpp               # 输出流包装器
│       ├── bigintegertest.cpp                   # 大整数测试
│       ├── itoatest.cpp                         # 整数转字符串测试
│       ├── dtoatest.cpp                         # 浮点转字符串测试
│       ├── strfunctest.cpp                      # 字符串工具测试
│       ├── strtodtest.cpp                       # 字符串转浮点测试
│       ├── regextest.cpp                        # 内部正则测试
│       ├── clzlltest.cpp                        # 位操作辅助测试
│       └── ...
│
├── inventory/                                   # [基线层] 测试资产清单目录
│   └── core-infra.legacy_tests.json             # [交付件] core-infra 相关 Legacy 测试资产清单
│
├── baseline/                                    # [基线层] Legacy 行为基线样本目录
│   └── core-infra.golden_samples.jsonl          # [交付件] 关键测试的输入/输出黄金样本
│
└── reports/                                     # [基线层] 测试报告汇总目录
    ├── core-infra.legacy.junit.xml              # [交付件] Legacy 测试执行结果
    └── core-infra.legacy.coverage.xml           # [交付件] Legacy 覆盖率报告
```

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| Allocator.Alignment | test/unittest/allocatorstest.cpp:257 | legacy_test |
| Allocator.CrtAllocator | test/unittest/allocatorstest.cpp:176 | legacy_test |
| Allocator.Issue399 | test/unittest/allocatorstest.cpp:275 | legacy_test |
| Allocator.MemoryPoolAllocator | test/unittest/allocatorstest.cpp:190 | legacy_test |
| BigInteger.AddUint64 | test/unittest/bigintegertest.cpp:44 | legacy_test |
| BigInteger.Compare | test/unittest/bigintegertest.cpp:130 | legacy_test |
| BigInteger.Constructor | test/unittest/bigintegertest.cpp:28 | legacy_test |
| BigInteger.LeftShift | test/unittest/bigintegertest.cpp:107 | legacy_test |
| BigInteger.MultiplyUint32 | test/unittest/bigintegertest.cpp:85 | legacy_test |
| BigInteger.MultiplyUint64 | test/unittest/bigintegertest.cpp:63 | legacy_test |
| clzll.normal | test/unittest/clzlltest.cpp:24 | legacy_test |
| dtoa.maxDecimalPlaces | test/unittest/dtoatest.cpp:54 | legacy_test |
| dtoa.normal | test/unittest/dtoatest.cpp:25 | legacy_test |
| FileStreamTest.FileReadStream | test/unittest/filestreamtest.cpp:90 | legacy_test |
| FileStreamTest.FileReadStream_Peek4 | test/unittest/filestreamtest.cpp:108 | legacy_test |
| FileStreamTest.FileWriteStream | test/unittest/filestreamtest.cpp:132 | legacy_test |
| IStreamWrapper.fstream | test/unittest/istreamwrappertest.cpp:135 | legacy_test |
| IStreamWrapper.ifstream | test/unittest/istreamwrappertest.cpp:124 | legacy_test |
| IStreamWrapper.istringstream | test/unittest/istreamwrappertest.cpp:89 | legacy_test |
| IStreamWrapper.stringstream | test/unittest/istreamwrappertest.cpp:93 | legacy_test |
| IStreamWrapper.wistringstream | test/unittest/istreamwrappertest.cpp:97 | legacy_test |
| IStreamWrapper.wstringstream | test/unittest/istreamwrappertest.cpp:101 | legacy_test |
| itoa.i32toa | test/unittest/itoatest.cpp:146 | legacy_test |
| itoa.i64toa | test/unittest/itoatest.cpp:154 | legacy_test |
| itoa.u32toa | test/unittest/itoatest.cpp:142 | legacy_test |
| itoa.u64toa | test/unittest/itoatest.cpp:150 | legacy_test |
| OStreamWrapper.cout | test/unittest/ostreamwrappertest.cpp:56 | legacy_test |
| OStreamWrapper.fstream | test/unittest/ostreamwrappertest.cpp:90 | legacy_test |
| OStreamWrapper.ofstream | test/unittest/ostreamwrappertest.cpp:86 | legacy_test |
| OStreamWrapper.ostringstream | test/unittest/ostreamwrappertest.cpp:40 | legacy_test |
| OStreamWrapper.stringstream | test/unittest/ostreamwrappertest.cpp:44 | legacy_test |
| OStreamWrapper.wostringstream | test/unittest/ostreamwrappertest.cpp:48 | legacy_test |
| OStreamWrapper.wstringstream | test/unittest/ostreamwrappertest.cpp:52 | legacy_test |
| StrFunc.CountStringCodePoint | test/unittest/strfunctest.cpp:21 | legacy_test |
| StringBuffer.Clear | test/unittest/stringbuffertest.cpp:51 | legacy_test |
| StringBuffer.GetLength_Issue744 | test/unittest/stringbuffertest.cpp:89 | legacy_test |
| StringBuffer.InitialSize | test/unittest/stringbuffertest.cpp:26 | legacy_test |
| StringBuffer.MoveAssignment | test/unittest/stringbuffertest.cpp:169 | legacy_test |
| StringBuffer.MoveConstructor | test/unittest/stringbuffertest.cpp:141 | legacy_test |
| StringBuffer.Pop | test/unittest/stringbuffertest.cpp:75 | legacy_test |
| StringBuffer.Push | test/unittest/stringbuffertest.cpp:63 | legacy_test |
| StringBuffer.Put | test/unittest/stringbuffertest.cpp:33 | legacy_test |
| StringBuffer.PutN_Issue672 | test/unittest/stringbuffertest.cpp:42 | legacy_test |
| Strtod.CheckApproximationCase | test/unittest/strtodtest.cpp:28 | legacy_test |
| Uri.Assignment | test/unittest/uritest.cpp:311 | legacy_test |
| Uri.CopyConstructor | test/unittest/uritest.cpp:301 | legacy_test |
| Uri.DefaultConstructor | test/unittest/uritest.cpp:32 | legacy_test |
| Uri.Equals | test/unittest/uritest.cpp:685 | legacy_test |
| Uri.Issue1899 | test/unittest/uritest.cpp:714 | legacy_test |
| Uri.Match | test/unittest/uritest.cpp:697 | legacy_test |
| Uri.Parse | test/unittest/uritest.cpp:51 | legacy_test |
| Uri.Parse_Std | test/unittest/uritest.cpp:267 | legacy_test |
| Uri.Parse_UTF16 | test/unittest/uritest.cpp:158 | legacy_test |
| Uri.Parse_UTF16_Std | test/unittest/uritest.cpp:283 | legacy_test |
| Uri.Resolve | test/unittest/uritest.cpp:322 | legacy_test |
| Uri.Resolve_UTF16 | test/unittest/uritest.cpp:503 | legacy_test |## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── rapidjson_refactoring_sys/                    # [镜像层] C++ 工程 bindgen 胶水层目录
│   └── core_infra_ffi/                           # core-infra 相关 FFI 适配代码（规划）
│
├── migrations/
│   └── core-infra.legacy_to_mirror.json          # gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── core-infra.mirror.junit.xml               # 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── core_infra_mirror.rs              # core-infra 的 Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| Allocator.Alignment | test/unittest/allocatorstest.cpp:257 | core_infra_mirror::Allocator_Alignment | |
| Allocator.CrtAllocator | test/unittest/allocatorstest.cpp:176 | core_infra_mirror::Allocator_CrtAllocator | |
| Allocator.Issue399 | test/unittest/allocatorstest.cpp:275 | core_infra_mirror::Allocator_Issue399 | |
| Allocator.MemoryPoolAllocator | test/unittest/allocatorstest.cpp:190 | core_infra_mirror::Allocator_MemoryPoolAllocator | |
| BigInteger.AddUint64 | test/unittest/bigintegertest.cpp:44 | core_infra_mirror::BigInteger_AddUint64 | |
| BigInteger.Compare | test/unittest/bigintegertest.cpp:130 | core_infra_mirror::BigInteger_Compare | |
| BigInteger.Constructor | test/unittest/bigintegertest.cpp:28 | core_infra_mirror::BigInteger_Constructor | |
| BigInteger.LeftShift | test/unittest/bigintegertest.cpp:107 | core_infra_mirror::BigInteger_LeftShift | |
| BigInteger.MultiplyUint32 | test/unittest/bigintegertest.cpp:85 | core_infra_mirror::BigInteger_MultiplyUint32 | |
| BigInteger.MultiplyUint64 | test/unittest/bigintegertest.cpp:63 | core_infra_mirror::BigInteger_MultiplyUint64 | |
| clzll.normal | test/unittest/clzlltest.cpp:24 | core_infra_mirror::clzll_normal | |
| dtoa.maxDecimalPlaces | test/unittest/dtoatest.cpp:54 | core_infra_mirror::dtoa_maxDecimalPlaces | |
| dtoa.normal | test/unittest/dtoatest.cpp:25 | core_infra_mirror::dtoa_normal | |
| FileStreamTest.FileReadStream | test/unittest/filestreamtest.cpp:90 | core_infra_mirror::FileStreamTest_FileReadStream | |
| FileStreamTest.FileReadStream_Peek4 | test/unittest/filestreamtest.cpp:108 | core_infra_mirror::FileStreamTest_FileReadStream_Peek4 | |
| FileStreamTest.FileWriteStream | test/unittest/filestreamtest.cpp:132 | core_infra_mirror::FileStreamTest_FileWriteStream | |
| IStreamWrapper.fstream | test/unittest/istreamwrappertest.cpp:135 | core_infra_mirror::IStreamWrapper_fstream | |
| IStreamWrapper.ifstream | test/unittest/istreamwrappertest.cpp:124 | core_infra_mirror::IStreamWrapper_ifstream | |
| IStreamWrapper.istringstream | test/unittest/istreamwrappertest.cpp:89 | core_infra_mirror::IStreamWrapper_istringstream | |
| IStreamWrapper.stringstream | test/unittest/istreamwrappertest.cpp:93 | core_infra_mirror::IStreamWrapper_stringstream | |
| IStreamWrapper.wistringstream | test/unittest/istreamwrappertest.cpp:97 | core_infra_mirror::IStreamWrapper_wistringstream | |
| IStreamWrapper.wstringstream | test/unittest/istreamwrappertest.cpp:101 | core_infra_mirror::IStreamWrapper_wstringstream | |
| itoa.i32toa | test/unittest/itoatest.cpp:146 | core_infra_mirror::itoa_i32toa | |
| itoa.i64toa | test/unittest/itoatest.cpp:154 | core_infra_mirror::itoa_i64toa | |
| itoa.u32toa | test/unittest/itoatest.cpp:142 | core_infra_mirror::itoa_u32toa | |
| itoa.u64toa | test/unittest/itoatest.cpp:150 | core_infra_mirror::itoa_u64toa | |
| OStreamWrapper.cout | test/unittest/ostreamwrappertest.cpp:56 | core_infra_mirror::OStreamWrapper_cout | |
| OStreamWrapper.fstream | test/unittest/ostreamwrappertest.cpp:90 | core_infra_mirror::OStreamWrapper_fstream | |
| OStreamWrapper.ofstream | test/unittest/ostreamwrappertest.cpp:86 | core_infra_mirror::OStreamWrapper_ofstream | |
| OStreamWrapper.ostringstream | test/unittest/ostreamwrappertest.cpp:40 | core_infra_mirror::OStreamWrapper_ostringstream | |
| OStreamWrapper.stringstream | test/unittest/ostreamwrappertest.cpp:44 | core_infra_mirror::OStreamWrapper_stringstream | |
| OStreamWrapper.wostringstream | test/unittest/ostreamwrappertest.cpp:48 | core_infra_mirror::OStreamWrapper_wostringstream | |
| OStreamWrapper.wstringstream | test/unittest/ostreamwrappertest.cpp:52 | core_infra_mirror::OStreamWrapper_wstringstream | |
| StrFunc.CountStringCodePoint | test/unittest/strfunctest.cpp:21 | core_infra_mirror::StrFunc_CountStringCodePoint | |
| StringBuffer.Clear | test/unittest/stringbuffertest.cpp:51 | core_infra_mirror::StringBuffer_Clear | |
| StringBuffer.GetLength_Issue744 | test/unittest/stringbuffertest.cpp:89 | core_infra_mirror::StringBuffer_GetLength_Issue744 | |
| StringBuffer.InitialSize | test/unittest/stringbuffertest.cpp:26 | core_infra_mirror::StringBuffer_InitialSize | |
| StringBuffer.MoveAssignment | test/unittest/stringbuffertest.cpp:169 | core_infra_mirror::StringBuffer_MoveAssignment | |
| StringBuffer.MoveConstructor | test/unittest/stringbuffertest.cpp:141 | core_infra_mirror::StringBuffer_MoveConstructor | |
| StringBuffer.Pop | test/unittest/stringbuffertest.cpp:75 | core_infra_mirror::StringBuffer_Pop | |
| StringBuffer.Push | test/unittest/stringbuffertest.cpp:63 | core_infra_mirror::StringBuffer_Push | |
| StringBuffer.Put | test/unittest/stringbuffertest.cpp:33 | core_infra_mirror::StringBuffer_Put | |
| StringBuffer.PutN_Issue672 | test/unittest/stringbuffertest.cpp:42 | core_infra_mirror::StringBuffer_PutN_Issue672 | |
| Strtod.CheckApproximationCase | test/unittest/strtodtest.cpp:28 | core_infra_mirror::Strtod_CheckApproximationCase | |
| Uri.Assignment | test/unittest/uritest.cpp:311 | core_infra_mirror::Uri_Assignment | |
| Uri.CopyConstructor | test/unittest/uritest.cpp:301 | core_infra_mirror::Uri_CopyConstructor | |
| Uri.DefaultConstructor | test/unittest/uritest.cpp:32 | core_infra_mirror::Uri_DefaultConstructor | |
| Uri.Equals | test/unittest/uritest.cpp:685 | core_infra_mirror::Uri_Equals | |
| Uri.Issue1899 | test/unittest/uritest.cpp:714 | core_infra_mirror::Uri_Issue1899 | |
| Uri.Match | test/unittest/uritest.cpp:697 | core_infra_mirror::Uri_Match | |
| Uri.Parse | test/unittest/uritest.cpp:51 | core_infra_mirror::Uri_Parse | |
| Uri.Parse_Std | test/unittest/uritest.cpp:267 | core_infra_mirror::Uri_Parse_Std | |
| Uri.Parse_UTF16 | test/unittest/uritest.cpp:158 | core_infra_mirror::Uri_Parse_UTF16 | |
| Uri.Parse_UTF16_Std | test/unittest/uritest.cpp:283 | core_infra_mirror::Uri_Parse_UTF16_Std | |
| Uri.Resolve | test/unittest/uritest.cpp:322 | core_infra_mirror::Uri_Resolve | |
| Uri.Resolve_UTF16 | test/unittest/uritest.cpp:503 | core_infra_mirror::Uri_Resolve_UTF16 | |#### 3.3 迁移策略建议

| 测试 ID | 进入 L1 优先级建议 | 原因 |
|---------|--------------------|------|
| `Allocator.*` | high | 直接影响所有 DOM/Schema/解析的内存正确性和稳定性。 |
| `FileStreamTest.*` | medium | 主要影响基于文件的 IO 场景，可在基础逻辑稳定后迁移。 |
| `IStreamWrapper.*`/`OStreamWrapper.*` | medium | 用于封装标准流，重要但非功能核心。 |
| `BigInteger.*` / `itoa.*` / `dtoa.*` / `Strtod.*` | high | 直接影响数值转换正确性，Schema 和打印路径高度依赖。 |
| `StrFunc.*` | medium | 属于工具函数，重要程度次于数值转换。 |
| `Regex.*` | high | 直接影响 Schema pattern 行为，应尽早镜像。 |
| `clzll.normal` | medium | 辅助函数，优先级中等。 |

### 3.4 镜像测试交付件

| 交付件 | 说明 |
|--------|------|
| `reports/core-infra.mirror.junit.xml` | 镜像层执行结果（Rust 控制、C++ 实现）。 |
| `migrations/core-infra.legacy_to_mirror.json` | 从 gtest 到 Rust 镜像测试的映射表。 |
| `rapidjson-rs/tests/mirrors/core_infra_mirror.rs` | 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── core-infra.rust.junit.xml                 # 孪生层执行结果（Rust 实现）
│   └── core-infra.parity.json                    # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── core-infra.mirror_to_rust.json            # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── core_infra.rs                     # core-infra 的 Rust 原生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| Allocator.Alignment | core_infra_mirror::Allocator_Alignment | core_infra | core_infra::Allocator_Alignment_rust | | |
| Allocator.CrtAllocator | core_infra_mirror::Allocator_CrtAllocator | core_infra | core_infra::Allocator_CrtAllocator_rust | | |
| Allocator.Issue399 | core_infra_mirror::Allocator_Issue399 | core_infra | core_infra::Allocator_Issue399_rust | | |
| Allocator.MemoryPoolAllocator | core_infra_mirror::Allocator_MemoryPoolAllocator | core_infra | core_infra::Allocator_MemoryPoolAllocator_rust | | |
| BigInteger.AddUint64 | core_infra_mirror::BigInteger_AddUint64 | core_infra | core_infra::BigInteger_AddUint64_rust | | |
| BigInteger.Compare | core_infra_mirror::BigInteger_Compare | core_infra | core_infra::BigInteger_Compare_rust | | |
| BigInteger.Constructor | core_infra_mirror::BigInteger_Constructor | core_infra | core_infra::BigInteger_Constructor_rust | | |
| BigInteger.LeftShift | core_infra_mirror::BigInteger_LeftShift | core_infra | core_infra::BigInteger_LeftShift_rust | | |
| BigInteger.MultiplyUint32 | core_infra_mirror::BigInteger_MultiplyUint32 | core_infra | core_infra::BigInteger_MultiplyUint32_rust | | |
| BigInteger.MultiplyUint64 | core_infra_mirror::BigInteger_MultiplyUint64 | core_infra | core_infra::BigInteger_MultiplyUint64_rust | | |
| clzll.normal | core_infra_mirror::clzll_normal | core_infra | core_infra::clzll_normal_rust | | |
| dtoa.maxDecimalPlaces | core_infra_mirror::dtoa_maxDecimalPlaces | core_infra | core_infra::dtoa_maxDecimalPlaces_rust | | |
| dtoa.normal | core_infra_mirror::dtoa_normal | core_infra | core_infra::dtoa_normal_rust | | |
| FileStreamTest.FileReadStream | core_infra_mirror::FileStreamTest_FileReadStream | core_infra | core_infra::FileStreamTest_FileReadStream_rust | | |
| FileStreamTest.FileReadStream_Peek4 | core_infra_mirror::FileStreamTest_FileReadStream_Peek4 | core_infra | core_infra::FileStreamTest_FileReadStream_Peek4_rust | | |
| FileStreamTest.FileWriteStream | core_infra_mirror::FileStreamTest_FileWriteStream | core_infra | core_infra::FileStreamTest_FileWriteStream_rust | | |
| IStreamWrapper.fstream | core_infra_mirror::IStreamWrapper_fstream | core_infra | core_infra::IStreamWrapper_fstream_rust | | |
| IStreamWrapper.ifstream | core_infra_mirror::IStreamWrapper_ifstream | core_infra | core_infra::IStreamWrapper_ifstream_rust | | |
| IStreamWrapper.istringstream | core_infra_mirror::IStreamWrapper_istringstream | core_infra | core_infra::IStreamWrapper_istringstream_rust | | |
| IStreamWrapper.stringstream | core_infra_mirror::IStreamWrapper_stringstream | core_infra | core_infra::IStreamWrapper_stringstream_rust | | |
| IStreamWrapper.wistringstream | core_infra_mirror::IStreamWrapper_wistringstream | core_infra | core_infra::IStreamWrapper_wistringstream_rust | | |
| IStreamWrapper.wstringstream | core_infra_mirror::IStreamWrapper_wstringstream | core_infra | core_infra::IStreamWrapper_wstringstream_rust | | |
| itoa.i32toa | core_infra_mirror::itoa_i32toa | core_infra | core_infra::itoa_i32toa_rust | | |
| itoa.i64toa | core_infra_mirror::itoa_i64toa | core_infra | core_infra::itoa_i64toa_rust | | |
| itoa.u32toa | core_infra_mirror::itoa_u32toa | core_infra | core_infra::itoa_u32toa_rust | | |
| itoa.u64toa | core_infra_mirror::itoa_u64toa | core_infra | core_infra::itoa_u64toa_rust | | |
| OStreamWrapper.cout | core_infra_mirror::OStreamWrapper_cout | core_infra | core_infra::OStreamWrapper_cout_rust | | |
| OStreamWrapper.fstream | core_infra_mirror::OStreamWrapper_fstream | core_infra | core_infra::OStreamWrapper_fstream_rust | | |
| OStreamWrapper.ofstream | core_infra_mirror::OStreamWrapper_ofstream | core_infra | core_infra::OStreamWrapper_ofstream_rust | | |
| OStreamWrapper.ostringstream | core_infra_mirror::OStreamWrapper_ostringstream | core_infra | core_infra::OStreamWrapper_ostringstream_rust | | |
| OStreamWrapper.stringstream | core_infra_mirror::OStreamWrapper_stringstream | core_infra | core_infra::OStreamWrapper_stringstream_rust | | |
| OStreamWrapper.wostringstream | core_infra_mirror::OStreamWrapper_wostringstream | core_infra | core_infra::OStreamWrapper_wostringstream_rust | | |
| OStreamWrapper.wstringstream | core_infra_mirror::OStreamWrapper_wstringstream | core_infra | core_infra::OStreamWrapper_wstringstream_rust | | |
| StrFunc.CountStringCodePoint | core_infra_mirror::StrFunc_CountStringCodePoint | core_infra | core_infra::StrFunc_CountStringCodePoint_rust | | |
| StringBuffer.Clear | core_infra_mirror::StringBuffer_Clear | core_infra | core_infra::StringBuffer_Clear_rust | | |
| StringBuffer.GetLength_Issue744 | core_infra_mirror::StringBuffer_GetLength_Issue744 | core_infra | core_infra::StringBuffer_GetLength_Issue744_rust | | |
| StringBuffer.InitialSize | core_infra_mirror::StringBuffer_InitialSize | core_infra | core_infra::StringBuffer_InitialSize_rust | | |
| StringBuffer.MoveAssignment | core_infra_mirror::StringBuffer_MoveAssignment | core_infra | core_infra::StringBuffer_MoveAssignment_rust | | |
| StringBuffer.MoveConstructor | core_infra_mirror::StringBuffer_MoveConstructor | core_infra | core_infra::StringBuffer_MoveConstructor_rust | | |
| StringBuffer.Pop | core_infra_mirror::StringBuffer_Pop | core_infra | core_infra::StringBuffer_Pop_rust | | |
| StringBuffer.Push | core_infra_mirror::StringBuffer_Push | core_infra | core_infra::StringBuffer_Push_rust | | |
| StringBuffer.Put | core_infra_mirror::StringBuffer_Put | core_infra | core_infra::StringBuffer_Put_rust | | |
| StringBuffer.PutN_Issue672 | core_infra_mirror::StringBuffer_PutN_Issue672 | core_infra | core_infra::StringBuffer_PutN_Issue672_rust | | |
| Strtod.CheckApproximationCase | core_infra_mirror::Strtod_CheckApproximationCase | core_infra | core_infra::Strtod_CheckApproximationCase_rust | | |
| Uri.Assignment | core_infra_mirror::Uri_Assignment | core_infra | core_infra::Uri_Assignment_rust | | |
| Uri.CopyConstructor | core_infra_mirror::Uri_CopyConstructor | core_infra | core_infra::Uri_CopyConstructor_rust | | |
| Uri.DefaultConstructor | core_infra_mirror::Uri_DefaultConstructor | core_infra | core_infra::Uri_DefaultConstructor_rust | | |
| Uri.Equals | core_infra_mirror::Uri_Equals | core_infra | core_infra::Uri_Equals_rust | | |
| Uri.Issue1899 | core_infra_mirror::Uri_Issue1899 | core_infra | core_infra::Uri_Issue1899_rust | | |
| Uri.Match | core_infra_mirror::Uri_Match | core_infra | core_infra::Uri_Match_rust | | |
| Uri.Parse | core_infra_mirror::Uri_Parse | core_infra | core_infra::Uri_Parse_rust | | |
| Uri.Parse_Std | core_infra_mirror::Uri_Parse_Std | core_infra | core_infra::Uri_Parse_Std_rust | | |
| Uri.Parse_UTF16 | core_infra_mirror::Uri_Parse_UTF16 | core_infra | core_infra::Uri_Parse_UTF16_rust | | |
| Uri.Parse_UTF16_Std | core_infra_mirror::Uri_Parse_UTF16_Std | core_infra | core_infra::Uri_Parse_UTF16_Std_rust | | |
| Uri.Resolve | core_infra_mirror::Uri_Resolve | core_infra | core_infra::Uri_Resolve_rust | | |
| Uri.Resolve_UTF16 | core_infra_mirror::Uri_Resolve_UTF16 | core_infra | core_infra::Uri_Resolve_UTF16_rust | | |#### 4.3 迁移策略建议

| 测试 ID | 进入 L2 优先级建议 | 原因 |
|---------|--------------------|------|
| `Allocator.*` | high | 分配器是整个库稳定性的基础，应最早完成孪生测试验证。 |
| `BigInteger.*`/`itoa.*`/`dtoa.*`/`Strtod.*` | high | 数值转换错误会直接影响输出与 Schema 校验。 |
| `Regex.*` | high | 影响 Schema `pattern` 与 URI 等，后续 feature 强依赖。 |
| 流相关 `FileStreamTest.*`/`IStreamWrapper.*`/`OStreamWrapper.*` | medium | 重要但对核心算法的影响相对间接。 |

### 4.4 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/core-infra.rust.junit.xml` | 孪生层执行结果（Rust 实现的基础设施行为）。 |
| `reports/core-infra.parity.json` | 镜像测试 vs 孪生测试一致性报告。 |
| `migrations/core-infra.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/core_infra.rs` | 孪生测试源文件。 |

---

## 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| `0.1.0-draft` | `2026-05-14` | 初始化 FEAT-001 `core-infra` feature 级测试设计文档，梳理基础设施相关 Legacy 测试与三层防护网结构。 | `TBD` |
