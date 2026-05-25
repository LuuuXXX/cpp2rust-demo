# component 级测试架构设计报告

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

- [1. 测试概述](#1-测试概述)
- [2. 测试防护网架构](#2-测试防护网架构)
- [3. Legacy 测试清单-字母序](#3-legacy-测试清单-字母序)

---

## 1. 测试概述

### 1.1 测试目标

测试所属的 component: `rapidjson-rs`

测试总体目标：

1. 验证 Rust 实现的 `rapidjson-rs` 在行为上与 C++ 版 RapidJSON 在给定测试范围内保持等价，覆盖需求文档中列出的核心功能（Parsing/Generation/DOM/SAX/Pointer/Schema/Encoding/Stream/Memory 管理等）。
2. 利用现有 C++ gtest 列表 `docs/requirements/gtests.csv` 构建完善的 Legacy 测试基线，为后续镜像测试（L1）与孪生测试（L2）提供可比对的行为参考。
3. 在不引入 crates.io 第三方依赖的前提下，构建可自动化执行的测试防护网，支撑持续演进与回归。

### 1.2 测试工具链

| 工具 | 用途 | 版本 |
|------|------|------|
| C/C++ gtest 框架 | 执行 legacy C++ 测试用例，形成行为基线与覆盖率数据 | 与原 RapidJSON 项目一致 |
| C/C++ 构建系统（如 CMake） | 构建 legacy RapidJSON 与其测试二进制 | 与原项目配置一致 |
| `cargo test` | 执行未来 Rust 侧镜像测试与孪生测试（L1/L2） | 与 workspace Rust 版本一致 |
| Python 3 + `fill_tests.py` | 从 `docs/requirements/gtests.csv` 生成 Legacy 测试清单表格 | Python 3 稳定版 |

> 说明：按商用依赖策略，测试阶段不引入 crates.io Rust 测试工具库，仅使用标准工具和自研脚本。

### 1.3 测试环境

- C/C++ 编译器：`与 legacy RapidJSON 项目一致的编译器版本（如 gcc/clang/MSVC），用于构建原始 C++ 代码与 gtest 测试`。
- 目标平台：`与生产环境一致的 Linux/macOS/Windows 及移动平台（按 NFR 要求配置），优先在 CI 中保证 Linux/macOS/Windows 三平台稳定通过`。
- C/C++ Legacy 项目构建方式：`沿用原 RapidJSON 的 CMake/编译脚本，不对源代码进行非必要修改`。
- C/C++ Legacy 项目测试环境：`通过 gtest 执行单元/集成测试，输出测试结果与（可选）覆盖率报告`。
- C/C++ Legacy 项目覆盖率环境：`可选地集成 gcov/llvm-cov 等覆盖率工具，为关键模块生成覆盖率基线`。

### 1.4 商用代码与测试依赖约束

| 字段 | 内容 |
|------|------|
| 是否商用代码 | `是` |
| 允许依赖范围 | `core/std-only`（禁止 crates.io 第三方依赖） |
| crates.io 使用策略 | 核心 crate `rapidjson-rs` 在测试中不引入任何 crates.io 依赖；Python 脚本与 C/C++ 测试框架按各自许可证使用。 |
| 对技术栈的影响 | 测试用例与测试工具不依赖 Rust 第三方 crate；需要的辅助逻辑（如基线 diff、数据生成）优先通过 C++/Python/自研脚本实现。 |
| 对测试工具链的影响 | 测试流程围绕 C++ gtest + `cargo test` + Python 脚本展开，不引入新的 Rust 测试框架或第三方 runner，简化合规审计。 |

---

## 2 测试防护网架构

<!-- 硬约束：原样复述下述设计，不展开任何详细设计，所有详细设计禁止由本文件给出。-->

测试断言只允许以下来源（`oracle_source`）：

1. `legacy_test`：旧 C/C++ 测试中的断言。
2. `c_output`：旧 C/C++ 实现对同一输入的实际输出。
3. `spec`：公开规格、协议文档、头文件契约或明确注释。
4. `invariant`：数学不变量、代数性质、状态机约束。
5. `metamorphic`：变形关系，例如排序前后元素集合不变、编解码往返等价。

测试防护网由三层构成，按时间顺序推进：

- **L0 基线层**：冻结旧 C/C++ 行为，保留旧测试，建立可对比基线。
- **L1 镜像测试层**：对原有 legacy gtest 编写 1:1 的 Rust 镜像测试，由 Rust 接管测试控制面；这些镜像测试通过 FFI 复现 C/C++ 测试逻辑，以验证 Rust 测试控制面与 legacy 基线一致。
- **L2 孪生测试层**：基于 L1 已有的镜像测试，编写基于 Rust 重构代码的孪生测试（原生测试），两个测试逻辑等价，验证镜像测试与孪生测试行为等价。

### 2.1 基线层功能

L0 负责冻结旧行为：

- 将给定的所有 C/C++ 测试收编到统一 CI。
- 为给定测试录制基线：**必选：测试结果、覆盖率、输入、输出、错误码**，可选：状态摘要、关键日志等。
- 为每个 C/C++ 测试打标签：`test_type`（UT/IT）、`oracle_source=legacy_test`、`determinism`（strong/weak）、环境依赖、风险等级等。
- 筛出“黄金基线”：稳定、断言明确、覆盖核心模块和历史 bugfix 的测试；严重 flaky 或断言过弱的测试不作为黄金基线。

L0 的验收条件：

- 旧测试在固定环境下可重复执行。
- 关键模块有基线样本和覆盖率报告。
- 测试资产清单与风险标注完成。
- UT/IT 已区分，行为档案可追溯。

### 2.2 镜像测试层功能

L1 的目标是**让 Rust 接管 legacy 测试控制面**。

对每个原有 gtest，编写一个 1:1 的 Rust 镜像测试，复刻 fixture/setup/assert 语义；被测实现仍然是 C/C++，通过 bindgen/hicc 等工具 生成的 FFI 调用。

L1 包含两种场景：

1. **L1-A Assert Mirror（主路径）**
   - 为每个 legacy gtest 建立 Rust 镜像测试，测试名、场景和断言语义与原测试一一对应。
   - 镜像测试运行在 `cxx_ffi backend` 上；即测试逻辑在 Rust，业务实现仍是 C/C++。
   - `oracle_source` 默认来自 `legacy_test`，`source_ref` 指向原 gtest。
   - 验证目标是：Rust 控制面下的镜像测试，与 L0 冻结的 C/C++ 测试基线一致。

2. **L1-B Structured JSON 子模式（子路径）**
   - 这类测试与 Assert Mirror 使用同一份 Rust 镜像测试，只是输入为 json 格式的批量用例；
   - 仅适用于天然具有稳定输入/输出模型的模块。
   - 在这种场景下，按需定义共享的 input schema，用统一 JSON 输入输出比较 C 与 Rust 的可观测行为。

L1 的主要设计：

1. 为 C/C++ 暴露稳定的 FFI 入口：
   - bindgen：通过 shim 或 C ABI 包装暴露可被 bindgen 消费的入口，再生成 Safe Rust Binding；
   - hicc：直接用 HICC 生成安全封装的 Rust Binding。

2. 镜像测试层组织原则：
   - 原样翻译测试逻辑。
   - 保留 legacy gtest 的输入域、fixture/setup、边界条件、错误路径和关键断言。
   - 禁止在 L1 弱化断言或修改测试意图。

3. 结果验证方式：
   - Rust 镜像测试的 pass/fail 与 L0 基线对齐为主。
   - 如果输入为 Structured JSON 子模式：在不改变核心测试产物类型的前提下，对统一输入下的 C/C++ 与 Rust 可观测结果做 diff，并归档 diff 分类。

### 2.3 孪生测试层功能

L2 的目标是：**复用 L1 已有的镜像测试，在逻辑等价的情况下，编写基于 Rust 重构代码的孪生测试（原生测试），并验证镜像测试与孪生测试行为和测试结果等价。**

主要设计：

1. 同一份 Rust tests，孪生运行：
   - 测试 A：Rust 镜像测试。
   - 测试 B：Rust 原生测试。
   - 测试逻辑等价，测试体、断言、场景保持不变。

2. 等价性验证：
   - 记录 parity 报告：通过、失败、差异分类、受影响测试。
   - Structured JSON 子模式在此层继续有效，用于结构化模块的额外守护。

3. 迁移策略：
   - 优先让 deterministic、边界清晰、FFI 易稳定的模块进入孪生测试验证。
   - 暂缓严重依赖 C++ 私有对象身份、内存布局或尚未稳定桥接的测试。

4. 并跑期：
   - 镜像测试与孪生测试一段时间内并跑。
   - 只有多轮 parity 稳定后，才讨论降低对 C++ 镜像测试的依赖。

L2 验收条件：

- 同时执行镜像测试与孪生测试。
- 镜像测试与孪生测试结果一致。
- parity 报告完整且持续维护。

---

## 3. Legacy 测试清单-字母序

<!-- 硬约束：只整理清单；禁止复制或编写现实代码。-->

| test_id | file |
|---------|------|
| Allocator.Alignment | test/unittest/allocatorstest.cpp:257 |
| Allocator.CrtAllocator | test/unittest/allocatorstest.cpp:176 |
| Allocator.Issue399 | test/unittest/allocatorstest.cpp:275 |
| Allocator.MemoryPoolAllocator | test/unittest/allocatorstest.cpp:190 |
| BigInteger.AddUint64 | test/unittest/bigintegertest.cpp:44 |
| BigInteger.Compare | test/unittest/bigintegertest.cpp:130 |
| BigInteger.Constructor | test/unittest/bigintegertest.cpp:28 |
| BigInteger.LeftShift | test/unittest/bigintegertest.cpp:107 |
| BigInteger.MultiplyUint32 | test/unittest/bigintegertest.cpp:85 |
| BigInteger.MultiplyUint64 | test/unittest/bigintegertest.cpp:63 |
| CursorStreamWrapper.MissingArrayBracket | test/unittest/cursorstreamwrappertest.cpp:81 |
| CursorStreamWrapper.MissingArrayComma | test/unittest/cursorstreamwrappertest.cpp:90 |
| CursorStreamWrapper.MissingColon | test/unittest/cursorstreamwrappertest.cpp:54 |
| CursorStreamWrapper.MissingComma | test/unittest/cursorstreamwrappertest.cpp:72 |
| CursorStreamWrapper.MissingFirstBracket | test/unittest/cursorstreamwrappertest.cpp:36 |
| CursorStreamWrapper.MissingLastArrayBracket | test/unittest/cursorstreamwrappertest.cpp:99 |
| CursorStreamWrapper.MissingLastBracket | test/unittest/cursorstreamwrappertest.cpp:108 |
| CursorStreamWrapper.MissingQuotes | test/unittest/cursorstreamwrappertest.cpp:45 |
| CursorStreamWrapper.MissingSecondQuotes | test/unittest/cursorstreamwrappertest.cpp:63 |
| Document.AcceptWriter | test/unittest/documenttest.cpp:361 |
| Document.AssertAcceptInvalidNameType | test/unittest/documenttest.cpp:390 |
| Document.CrtAllocator | test/unittest/valuetest.cpp:1703 |
| Document.Parse | test/unittest/documenttest.cpp:120 |
| Document.ParseStream_AutoUTFInputStream | test/unittest/documenttest.cpp:256 |
| Document.ParseStream_EncodedInputStream | test/unittest/documenttest.cpp:215 |
| Document.Parse_Encoding | test/unittest/documenttest.cpp:174 |
| Document.Swap | test/unittest/documenttest.cpp:293 |
| Document.UTF16_Document | test/unittest/documenttest.cpp:402 |
| Document.UnchangedOnParseError | test/unittest/documenttest.cpp:127 |
| Document.UserBuffer | test/unittest/documenttest.cpp:372 |
| DocumentMove/0.MoveAssignment | test/unittest/documenttest.cpp:563 |
| DocumentMove/0.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 |
| DocumentMove/0.MoveConstructor | test/unittest/documenttest.cpp:466 |
| DocumentMove/0.MoveConstructorParseError | test/unittest/documenttest.cpp:500 |
| DocumentMove/1.MoveAssignment | test/unittest/documenttest.cpp:563 |
| DocumentMove/1.MoveAssignmentParseError | test/unittest/documenttest.cpp:599 |
| DocumentMove/1.MoveConstructor | test/unittest/documenttest.cpp:466 |
| DocumentMove/1.MoveConstructorParseError | test/unittest/documenttest.cpp:500 |
| EncodedStreamTest.AutoUTFInputStream | test/unittest/encodedstreamtest.cpp:267 |
| EncodedStreamTest.AutoUTFOutputStream | test/unittest/encodedstreamtest.cpp:302 |
| EncodedStreamTest.EncodedInputStream | test/unittest/encodedstreamtest.cpp:254 |
| EncodedStreamTest.EncodedOutputStream | test/unittest/encodedstreamtest.cpp:289 |
| EncodingsTest.ASCII | test/unittest/encodingstest.cpp:428 |
| EncodingsTest.UTF16 | test/unittest/encodingstest.cpp:337 |
| EncodingsTest.UTF32 | test/unittest/encodingstest.cpp:397 |
| EncodingsTest.UTF8 | test/unittest/encodingstest.cpp:285 |
| FileStreamTest.FileReadStream | test/unittest/filestreamtest.cpp:90 |
| FileStreamTest.FileReadStream_Peek4 | test/unittest/filestreamtest.cpp:108 |
| FileStreamTest.FileWriteStream | test/unittest/filestreamtest.cpp:132 |
| Fwd.Fwd | test/unittest/fwdtest.cpp:224 |
| IStreamWrapper.fstream | test/unittest/istreamwrappertest.cpp:135 |
| IStreamWrapper.ifstream | test/unittest/istreamwrappertest.cpp:124 |
| IStreamWrapper.istringstream | test/unittest/istreamwrappertest.cpp:89 |
| IStreamWrapper.stringstream | test/unittest/istreamwrappertest.cpp:93 |
| IStreamWrapper.wistringstream | test/unittest/istreamwrappertest.cpp:97 |
| IStreamWrapper.wstringstream | test/unittest/istreamwrappertest.cpp:101 |
| JsonChecker.Reader | test/unittest/jsoncheckertest.cpp:69 |
| NamespaceTest.Direct | test/unittest/namespacetest.cpp:43 |
| NamespaceTest.Using | test/unittest/namespacetest.cpp:34 |
| OStreamWrapper.cout | test/unittest/ostreamwrappertest.cpp:56 |
| OStreamWrapper.fstream | test/unittest/ostreamwrappertest.cpp:90 |
| OStreamWrapper.ofstream | test/unittest/ostreamwrappertest.cpp:86 |
| OStreamWrapper.ostringstream | test/unittest/ostreamwrappertest.cpp:40 |
| OStreamWrapper.stringstream | test/unittest/ostreamwrappertest.cpp:44 |
| OStreamWrapper.wostringstream | test/unittest/ostreamwrappertest.cpp:48 |
| OStreamWrapper.wstringstream | test/unittest/ostreamwrappertest.cpp:52 |
| Platform.GetObject | test/unittest/platformtest.cpp:29 |
| Pointer.Ambiguity | test/unittest/pointertest.cpp:1562 |
| Pointer.Append | test/unittest/pointertest.cpp:567 |
| Pointer.Assignment | test/unittest/pointertest.cpp:497 |
| Pointer.ConstructorWithToken | test/unittest/pointertest.cpp:455 |
| Pointer.CopyConstructor | test/unittest/pointertest.cpp:466 |
| Pointer.Create | test/unittest/pointertest.cpp:613 |
| Pointer.CreateValueByPointer | test/unittest/pointertest.cpp:1011 |
| Pointer.CreateValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1025 |
| Pointer.DefaultConstructor | test/unittest/pointertest.cpp:38 |
| Pointer.Equality | test/unittest/pointertest.cpp:597 |
| Pointer.Erase | test/unittest/pointertest.cpp:972 |
| Pointer.EraseValueByPointer_Pointer | test/unittest/pointertest.cpp:1532 |
| Pointer.EraseValueByPointer_String | test/unittest/pointertest.cpp:1547 |
| Pointer.Get | test/unittest/pointertest.cpp:699 |
| Pointer.GetUri | test/unittest/pointertest.cpp:663 |
| Pointer.GetValueByPointer | test/unittest/pointertest.cpp:1038 |
| Pointer.GetValueByPointerWithDefault_Pointer | test/unittest/pointertest.cpp:1071 |
| Pointer.GetValueByPointerWithDefault_Pointer_NoAllocator | test/unittest/pointertest.cpp:1177 |
| Pointer.GetValueByPointerWithDefault_String | test/unittest/pointertest.cpp:1124 |
| Pointer.GetValueByPointerWithDefault_String_NoAllocator | test/unittest/pointertest.cpp:1229 |
| Pointer.GetWithDefault | test/unittest/pointertest.cpp:731 |
| Pointer.GetWithDefault_NoAllocator | test/unittest/pointertest.cpp:784 |
| Pointer.Inequality | test/unittest/pointertest.cpp:605 |
| Pointer.Issue1899 | test/unittest/pointertest.cpp:1721 |
| Pointer.Issue483 | test/unittest/pointertest.cpp:1713 |
| Pointer.LessThan | test/unittest/pointertest.cpp:1613 |
| Pointer.Parse | test/unittest/pointertest.cpp:44 |
| Pointer.Parse_URIFragment | test/unittest/pointertest.cpp:193 |
| Pointer.ResolveOnArray | test/unittest/pointertest.cpp:1597 |
| Pointer.ResolveOnObject | test/unittest/pointertest.cpp:1581 |
| Pointer.Set | test/unittest/pointertest.cpp:836 |
| Pointer.SetValueByPointer_Pointer | test/unittest/pointertest.cpp:1281 |
| Pointer.SetValueByPointer_Pointer_NoAllocator | test/unittest/pointertest.cpp:1395 |
| Pointer.SetValueByPointer_String | test/unittest/pointertest.cpp:1338 |
| Pointer.SetValueByPointer_String_NoAllocator | test/unittest/pointertest.cpp:1451 |
| Pointer.Set_NoAllocator | test/unittest/pointertest.cpp:896 |
| Pointer.Stringify | test/unittest/pointertest.cpp:404 |
| Pointer.Swap | test/unittest/pointertest.cpp:537 |
| Pointer.SwapValueByPointer | test/unittest/pointertest.cpp:1507 |
| Pointer.SwapValueByPointer_NoAllocator | test/unittest/pointertest.cpp:1520 |
| Pointer.Swap_Value | test/unittest/pointertest.cpp:955 |
| Pointer.Swap_Value_NoAllocator | test/unittest/pointertest.cpp:964 |
| PrettyWriter.Basic | test/unittest/prettywritertest.cpp:60 |
| PrettyWriter.FileWriteStream | test/unittest/prettywritertest.cpp:167 |
| PrettyWriter.FormatOptions | test/unittest/prettywritertest.cpp:69 |
| PrettyWriter.Inf | test/unittest/prettywritertest.cpp:281 |
| PrettyWriter.InvalidEventSequence | test/unittest/prettywritertest.cpp:211 |
| PrettyWriter.Issue_1336 | test/unittest/prettywritertest.cpp:342 |
| PrettyWriter.Issue_889 | test/unittest/prettywritertest.cpp:305 |
| PrettyWriter.MoveCtor | test/unittest/prettywritertest.cpp:329 |
| PrettyWriter.NaN | test/unittest/prettywritertest.cpp:262 |
| PrettyWriter.OStreamWrapper | test/unittest/prettywritertest.cpp:151 |
| PrettyWriter.RawValue | test/unittest/prettywritertest.cpp:192 |
| PrettyWriter.SetIndent | test/unittest/prettywritertest.cpp:79 |
| PrettyWriter.String | test/unittest/prettywritertest.cpp:106 |
| PrettyWriter.String_STDSTRING | test/unittest/prettywritertest.cpp:116 |
| RapidJson.DocumentAccept | test/perftest/rapidjsontest.cpp:331 |
| RapidJson.DocumentFind | test/perftest/rapidjsontest.cpp:339 |
| RapidJson.DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:271 |
| RapidJson.DocumentParseEncodedInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:261 |
| RapidJson.DocumentParseInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:199 |
| RapidJson.DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:208 |
| RapidJson.DocumentParseIterative_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:244 |
| RapidJson.DocumentParseLength_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:225 |
| RapidJson.DocumentParseStdString_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:234 |
| RapidJson.DocumentParse_CrtAllocator_SSE42 | test/perftest/rapidjsontest.cpp:252 |
| RapidJson.DocumentParse_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:217 |
| RapidJson.DocumentTraverse | test/perftest/rapidjsontest.cpp:304 |
| RapidJson.FileReadStream | test/perftest/rapidjsontest.cpp:464 |
| RapidJson.IStreamWrapper | test/perftest/rapidjsontest.cpp:487 |
| RapidJson.IStreamWrapper_Setbuffered | test/perftest/rapidjsontest.cpp:508 |
| RapidJson.IStreamWrapper_Unbuffered | test/perftest/rapidjsontest.cpp:498 |
| RapidJson.PrettyWriter_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:408 |
| RapidJson.ReaderParseInsitu_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 |
| RapidJson.ReaderParseInsitu_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 |
| RapidJson.ReaderParseInsitu_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 |
| RapidJson.ReaderParseInsitu_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 |
| RapidJson.ReaderParseInsitu_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 |
| RapidJson.ReaderParseInsitu_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 |
| RapidJson.ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 |
| RapidJson.ReaderParseInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:75 |
| RapidJson.ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:85 |
| RapidJson.ReaderParseIterativeInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:151 |
| RapidJson.ReaderParseIterativePullInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:175 |
| RapidJson.ReaderParseIterativePull_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:161 |
| RapidJson.ReaderParseIterative_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:142 |
| RapidJson.ReaderParse_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 |
| RapidJson.ReaderParse_DummyHandler_FileReadStream_SSE42 | test/perftest/rapidjsontest.cpp:475 |
| RapidJson.ReaderParse_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 |
| RapidJson.ReaderParse_DummyHandler_FullPrecision_SSE42 | test/perftest/rapidjsontest.cpp:133 |
| RapidJson.ReaderParse_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_SSE42 | test/perftest/rapidjsontest.cpp:521 |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | test/perftest/rapidjsontest.cpp:544 |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | test/perftest/rapidjsontest.cpp:533 |
| RapidJson.ReaderParse_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 |
| RapidJson.ReaderParse_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 |
| RapidJson.ReaderParse_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 |
| RapidJson.ReaderParse_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 |
| RapidJson.ReaderParse_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:95 |
| RapidJson.ReaderParse_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:190 |
| RapidJson.SkipWhitespace_Basic | test/perftest/rapidjsontest.cpp:428 |
| RapidJson.SkipWhitespace_SSE42 | test/perftest/rapidjsontest.cpp:437 |
| RapidJson.SkipWhitespace_strspn | test/perftest/rapidjsontest.cpp:445 |
| RapidJson.StringBuffer | test/perftest/rapidjsontest.cpp:558 |
| RapidJson.UTF8_Validate | test/perftest/rapidjsontest.cpp:452 |
| RapidJson.Writer_NullStream | test/perftest/rapidjsontest.cpp:365 |
| RapidJson.Writer_StringBuffer_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:398 |
| RapidJson.Writer_StringBuffer_Floats_SSE42 | test/perftest/rapidjsontest.cpp:399 |
| RapidJson.Writer_StringBuffer_Guids_SSE42 | test/perftest/rapidjsontest.cpp:400 |
| RapidJson.Writer_StringBuffer_Integers_SSE42 | test/perftest/rapidjsontest.cpp:401 |
| RapidJson.Writer_StringBuffer_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:402 |
| RapidJson.Writer_StringBuffer_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:403 |
| RapidJson.Writer_StringBuffer_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:404 |
| RapidJson.Writer_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:375 |
| RapidJson.internal_Pow10 | test/perftest/rapidjsontest.cpp:421 |
| Reader.BaseReaderHandler_Default | test/unittest/readertest.cpp:1754 |
| Reader.CustomStringStream | test/unittest/readertest.cpp:1373 |
| Reader.EmptyExceptForCommaErrors | test/unittest/readertest.cpp:2260 |
| Reader.EmptyExceptForCommaErrorsIterative | test/unittest/readertest.cpp:2264 |
| Reader.EofAfterOneLineComment | test/unittest/readertest.cpp:1891 |
| Reader.EscapedApostrophe | test/unittest/readertest.cpp:2368 |
| Reader.IncompleteMultilineComment | test/unittest/readertest.cpp:1901 |
| Reader.IncompleteMultilineComment2 | test/unittest/readertest.cpp:1911 |
| Reader.InlineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1859 |
| Reader.IterativeParsing_Count | test/unittest/readertest.cpp:1606 |
| Reader.IterativeParsing_ErrorHandling | test/unittest/readertest.cpp:1471 |
| Reader.IterativeParsing_General | test/unittest/readertest.cpp:1569 |
| Reader.IterativeParsing_ShortCircuit | test/unittest/readertest.cpp:1703 |
| Reader.IterativePullParsing_General | test/unittest/readertest.cpp:1641 |
| Reader.MultipleTrailingCommaErrors | test/unittest/readertest.cpp:2228 |
| Reader.MultipleTrailingCommaErrorsIterative | test/unittest/readertest.cpp:2232 |
| Reader.NumbersAsStrings | test/unittest/readertest.cpp:1961 |
| Reader.NumbersAsStringsWChar_t | test/unittest/readertest.cpp:2073 |
| Reader.OnelineCommentsAreDisabledByDefault | test/unittest/readertest.cpp:1882 |
| Reader.ParseArray | test/unittest/readertest.cpp:1074 |
| Reader.ParseArray_Error | test/unittest/readertest.cpp:1084 |
| Reader.ParseComments | test/unittest/readertest.cpp:1808 |
| Reader.ParseDocument_Error | test/unittest/readertest.cpp:1267 |
| Reader.ParseEmptyArray | test/unittest/readertest.cpp:1064 |
| Reader.ParseEmptyInlineComment | test/unittest/readertest.cpp:1826 |
| Reader.ParseEmptyOnelineComment | test/unittest/readertest.cpp:1836 |
| Reader.ParseFalse | test/unittest/readertest.cpp:60 |
| Reader.ParseInsituIterative_MultipleRoot | test/unittest/readertest.cpp:1249 |
| Reader.ParseInsitu_MultipleRoot | test/unittest/readertest.cpp:1245 |
| Reader.ParseIterative_MultipleRoot | test/unittest/readertest.cpp:1226 |
| Reader.ParseMultipleCommentsInARow | test/unittest/readertest.cpp:1846 |
| Reader.ParseNanAndInfinity | test/unittest/readertest.cpp:2298 |
| Reader.ParseNumberError_FullPrecisionDouble | test/unittest/readertest.cpp:752 |
| Reader.ParseNumberError_NormalPrecisionDouble | test/unittest/readertest.cpp:748 |
| Reader.ParseNumber_FullPrecisionDouble | test/unittest/readertest.cpp:592 |
| Reader.ParseNumber_Integer | test/unittest/readertest.cpp:113 |
| Reader.ParseNumber_NormalPrecisionDouble | test/unittest/readertest.cpp:588 |
| Reader.ParseNumber_NormalPrecisionError | test/unittest/readertest.cpp:596 |
| Reader.ParseObject | test/unittest/readertest.cpp:1155 |
| Reader.ParseObject_Error | test/unittest/readertest.cpp:1289 |
| Reader.ParseString | test/unittest/readertest.cpp:783 |
| Reader.ParseString_Error | test/unittest/readertest.cpp:908 |
| Reader.ParseString_NonDestructive | test/unittest/readertest.cpp:890 |
| Reader.ParseString_Transcoding | test/unittest/readertest.cpp:868 |
| Reader.ParseString_TranscodingWithValidation | test/unittest/readertest.cpp:879 |
| Reader.ParseTerminationByHandler | test/unittest/readertest.cpp:1788 |
| Reader.ParseTrue | test/unittest/readertest.cpp:52 |
| Reader.ParseValue_Error | test/unittest/readertest.cpp:1280 |
| Reader.Parse_EmptyObject | test/unittest/readertest.cpp:1189 |
| Reader.Parse_IStreamWrapper_StringStream | test/unittest/readertest.cpp:1444 |
| Reader.Parse_MultipleRoot | test/unittest/readertest.cpp:1222 |
| Reader.SkipWhitespace | test/unittest/readertest.cpp:1324 |
| Reader.TrailingCommaHandlerTermination | test/unittest/readertest.cpp:2290 |
| Reader.TrailingCommaHandlerTerminationIterative | test/unittest/readertest.cpp:2294 |
| Reader.TrailingCommas | test/unittest/readertest.cpp:2195 |
| Reader.TrailingCommasIterative | test/unittest/readertest.cpp:2199 |
| Reader.UnrecognizedComment | test/unittest/readertest.cpp:1921 |
| Regex.Alternation1 | test/unittest/regextest.cpp:41 |
| Regex.Alternation2 | test/unittest/regextest.cpp:54 |
| Regex.AnyCharacter | test/unittest/regextest.cpp:416 |
| Regex.CharacterRange1 | test/unittest/regextest.cpp:427 |
| Regex.CharacterRange2 | test/unittest/regextest.cpp:440 |
| Regex.CharacterRange3 | test/unittest/regextest.cpp:453 |
| Regex.CharacterRange4 | test/unittest/regextest.cpp:466 |
| Regex.CharacterRange5 | test/unittest/regextest.cpp:479 |
| Regex.CharacterRange6 | test/unittest/regextest.cpp:488 |
| Regex.CharacterRange7 | test/unittest/regextest.cpp:499 |
| Regex.CharacterRange8 | test/unittest/regextest.cpp:510 |
| Regex.Concatenation | test/unittest/regextest.cpp:29 |
| Regex.Escape | test/unittest/regextest.cpp:579 |
| Regex.Invalid | test/unittest/regextest.cpp:588 |
| Regex.Issue538 | test/unittest/regextest.cpp:629 |
| Regex.Issue583 | test/unittest/regextest.cpp:634 |
| Regex.OneOrMore1 | test/unittest/regextest.cpp:207 |
| Regex.OneOrMore2 | test/unittest/regextest.cpp:218 |
| Regex.OneOrMore3 | test/unittest/regextest.cpp:228 |
| Regex.OneOrMore4 | test/unittest/regextest.cpp:241 |
| Regex.Parenthesis1 | test/unittest/regextest.cpp:66 |
| Regex.Parenthesis2 | test/unittest/regextest.cpp:78 |
| Regex.Parenthesis3 | test/unittest/regextest.cpp:90 |
| Regex.QuantifierExact1 | test/unittest/regextest.cpp:251 |
| Regex.QuantifierExact2 | test/unittest/regextest.cpp:262 |
| Regex.QuantifierExact3 | test/unittest/regextest.cpp:273 |
| Regex.QuantifierMin1 | test/unittest/regextest.cpp:286 |
| Regex.QuantifierMin2 | test/unittest/regextest.cpp:298 |
| Regex.QuantifierMin3 | test/unittest/regextest.cpp:309 |
| Regex.QuantifierMinMax1 | test/unittest/regextest.cpp:322 |
| Regex.QuantifierMinMax2 | test/unittest/regextest.cpp:335 |
| Regex.QuantifierMinMax3 | test/unittest/regextest.cpp:348 |
| Regex.QuantifierMinMax4 | test/unittest/regextest.cpp:366 |
| Regex.QuantifierMinMax5 | test/unittest/regextest.cpp:385 |
| Regex.Search | test/unittest/regextest.cpp:521 |
| Regex.Search_BeginAnchor | test/unittest/regextest.cpp:537 |
| Regex.Search_BothAnchor | test/unittest/regextest.cpp:567 |
| Regex.Search_EndAnchor | test/unittest/regextest.cpp:552 |
| Regex.Single | test/unittest/regextest.cpp:20 |
| Regex.Unicode | test/unittest/regextest.cpp:406 |
| Regex.ZeroOrMore1 | test/unittest/regextest.cpp:160 |
| Regex.ZeroOrMore2 | test/unittest/regextest.cpp:171 |
| Regex.ZeroOrMore3 | test/unittest/regextest.cpp:182 |
| Regex.ZeroOrMore4 | test/unittest/regextest.cpp:196 |
| Regex.ZeroOrOne1 | test/unittest/regextest.cpp:103 |
| Regex.ZeroOrOne2 | test/unittest/regextest.cpp:112 |
| Regex.ZeroOrOne3 | test/unittest/regextest.cpp:124 |
| Regex.ZeroOrOne4 | test/unittest/regextest.cpp:136 |
| Regex.ZeroOrOne5 | test/unittest/regextest.cpp:150 |
| SIMD.ScanCopyUnescapedString_SSE42 | test/unittest/simdtest.cpp:164 |
| SIMD.ScanWriteUnescapedString_SSE42 | test/unittest/simdtest.cpp:169 |
| SIMD.SkipWhitespace_EncodedMemoryStream_SSE42 | test/unittest/simdtest.cpp:82 |
| SIMD.SkipWhitespace_SSE42 | test/unittest/simdtest.cpp:77 |
| Schema.Issue552 | test/unittest/schematest.cpp:2385 |
| Schema.Issue848 | test/unittest/schematest.cpp:2370 |
| Schema.TestSuite | test/perftest/schematest.cpp:198 |
| SchemaValidatingReader.Invalid | test/unittest/schematest.cpp:2308 |
| SchemaValidatingReader.Simple | test/unittest/schematest.cpp:2293 |
| SchemaValidatingWriter.Simple | test/unittest/schematest.cpp:2337 |
| SchemaValidator.AllOf | test/unittest/schematest.cpp:278 |
| SchemaValidator.AllOf_Nested | test/unittest/schematest.cpp:1742 |
| SchemaValidator.AnyOf | test/unittest/schematest.cpp:311 |
| SchemaValidator.Array | test/unittest/schematest.cpp:1465 |
| SchemaValidator.Array_AdditionalItems | test/unittest/schematest.cpp:1544 |
| SchemaValidator.Array_ItemsList | test/unittest/schematest.cpp:1480 |
| SchemaValidator.Array_ItemsRange | test/unittest/schematest.cpp:1579 |
| SchemaValidator.Array_ItemsTuple | test/unittest/schematest.cpp:1501 |
| SchemaValidator.Array_UniqueItems | test/unittest/schematest.cpp:1606 |
| SchemaValidator.Boolean | test/unittest/schematest.cpp:1627 |
| SchemaValidator.ContinueOnErrors | test/unittest/schematest.cpp:2678 |
| SchemaValidator.ContinueOnErrors_AllOf | test/unittest/schematest.cpp:2799 |
| SchemaValidator.ContinueOnErrors_AnyOf | test/unittest/schematest.cpp:2825 |
| SchemaValidator.ContinueOnErrors_BadSimpleType | test/unittest/schematest.cpp:2950 |
| SchemaValidator.ContinueOnErrors_Enum | test/unittest/schematest.cpp:2875 |
| SchemaValidator.ContinueOnErrors_OneOf | test/unittest/schematest.cpp:2773 |
| SchemaValidator.ContinueOnErrors_RogueArray | test/unittest/schematest.cpp:2894 |
| SchemaValidator.ContinueOnErrors_RogueObject | test/unittest/schematest.cpp:2913 |
| SchemaValidator.ContinueOnErrors_RogueString | test/unittest/schematest.cpp:2928 |
| SchemaValidator.ContinueOnErrors_UniqueItems | test/unittest/schematest.cpp:2853 |
| SchemaValidator.DuplicateKeyword | test/unittest/schematest.cpp:2982 |
| SchemaValidator.Enum_InvalidType | test/unittest/schematest.cpp:264 |
| SchemaValidator.Enum_Typed | test/unittest/schematest.cpp:242 |
| SchemaValidator.Enum_Typeless | test/unittest/schematest.cpp:252 |
| SchemaValidator.EscapedPointer | test/unittest/schematest.cpp:1824 |
| SchemaValidator.Hasher | test/unittest/schematest.cpp:51 |
| SchemaValidator.Integer | test/unittest/schematest.cpp:541 |
| SchemaValidator.Integer_MultipleOf | test/unittest/schematest.cpp:706 |
| SchemaValidator.Integer_MultipleOf64Boundary | test/unittest/schematest.cpp:729 |
| SchemaValidator.Integer_Range | test/unittest/schematest.cpp:566 |
| SchemaValidator.Integer_Range64Boundary | test/unittest/schematest.cpp:594 |
| SchemaValidator.Integer_Range64BoundaryExclusive | test/unittest/schematest.cpp:683 |
| SchemaValidator.Integer_RangeU64Boundary | test/unittest/schematest.cpp:626 |
| SchemaValidator.Issue1017_allOfHandler | test/unittest/schematest.cpp:2423 |
| SchemaValidator.Issue608 | test/unittest/schematest.cpp:2399 |
| SchemaValidator.Issue728_AllOfRef | test/unittest/schematest.cpp:2414 |
| SchemaValidator.MultiType | test/unittest/schematest.cpp:227 |
| SchemaValidator.MultiTypeInObject | test/unittest/schematest.cpp:1696 |
| SchemaValidator.MultiTypeWithObject | test/unittest/schematest.cpp:1719 |
| SchemaValidator.Not | test/unittest/schematest.cpp:365 |
| SchemaValidator.Null | test/unittest/schematest.cpp:1648 |
| SchemaValidator.NullableFalse | test/unittest/schematest.cpp:3540 |
| SchemaValidator.NullableTrue | test/unittest/schematest.cpp:3509 |
| SchemaValidator.Number_MultipleOf | test/unittest/schematest.cpp:995 |
| SchemaValidator.Number_MultipleOfOne | test/unittest/schematest.cpp:1039 |
| SchemaValidator.Number_Range | test/unittest/schematest.cpp:744 |
| SchemaValidator.Number_RangeDouble | test/unittest/schematest.cpp:855 |
| SchemaValidator.Number_RangeDoubleU64Boundary | test/unittest/schematest.cpp:944 |
| SchemaValidator.Number_RangeInt | test/unittest/schematest.cpp:780 |
| SchemaValidator.Object | test/unittest/schematest.cpp:1054 |
| SchemaValidator.ObjectInArray | test/unittest/schematest.cpp:1676 |
| SchemaValidator.Object_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1108 |
| SchemaValidator.Object_AdditionalPropertiesObject | test/unittest/schematest.cpp:1134 |
| SchemaValidator.Object_PatternProperties | test/unittest/schematest.cpp:1322 |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesBoolean | test/unittest/schematest.cpp:1441 |
| SchemaValidator.Object_PatternProperties_AdditionalPropertiesObject | test/unittest/schematest.cpp:1414 |
| SchemaValidator.Object_PatternProperties_ErrorConflict | test/unittest/schematest.cpp:1351 |
| SchemaValidator.Object_Properties | test/unittest/schematest.cpp:1075 |
| SchemaValidator.Object_PropertiesRange | test/unittest/schematest.cpp:1221 |
| SchemaValidator.Object_Properties_PatternProperties | test/unittest/schematest.cpp:1378 |
| SchemaValidator.Object_PropertyDependencies | test/unittest/schematest.cpp:1248 |
| SchemaValidator.Object_Required | test/unittest/schematest.cpp:1160 |
| SchemaValidator.Object_Required_PassWithDefault | test/unittest/schematest.cpp:1191 |
| SchemaValidator.Object_SchemaDependencies | test/unittest/schematest.cpp:1284 |
| SchemaValidator.OneOf | test/unittest/schematest.cpp:337 |
| SchemaValidator.ReadOnlyWhenWriting | test/unittest/schematest.cpp:3465 |
| SchemaValidator.Ref | test/unittest/schematest.cpp:376 |
| SchemaValidator.Ref_AllOf | test/unittest/schematest.cpp:404 |
| SchemaValidator.Ref_internal_id_1 | test/unittest/schematest.cpp:2537 |
| SchemaValidator.Ref_internal_id_2 | test/unittest/schematest.cpp:2555 |
| SchemaValidator.Ref_internal_id_and_schema_pointer | test/unittest/schematest.cpp:2591 |
| SchemaValidator.Ref_internal_id_in_array | test/unittest/schematest.cpp:2573 |
| SchemaValidator.Ref_internal_multiple_ids | test/unittest/schematest.cpp:2610 |
| SchemaValidator.Ref_remote | test/unittest/schematest.cpp:2442 |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path | test/unittest/schematest.cpp:2499 |
| SchemaValidator.Ref_remote_change_resolution_scope_absolute_path_document | test/unittest/schematest.cpp:2518 |
| SchemaValidator.Ref_remote_change_resolution_scope_relative_path | test/unittest/schematest.cpp:2480 |
| SchemaValidator.Ref_remote_change_resolution_scope_uri | test/unittest/schematest.cpp:2461 |
| SchemaValidator.Ref_remote_issue1210 | test/unittest/schematest.cpp:2639 |
| SchemaValidator.SchemaPointer | test/unittest/schematest.cpp:1842 |
| SchemaValidator.Schema_DraftAndVersion | test/unittest/schematest.cpp:3318 |
| SchemaValidator.Schema_IgnoreDraftEmbedded | test/unittest/schematest.cpp:3088 |
| SchemaValidator.Schema_MultipleErrors | test/unittest/schematest.cpp:3335 |
| SchemaValidator.Schema_ReadOnlyAndWriteOnly | test/unittest/schematest.cpp:3455 |
| SchemaValidator.Schema_RefCyclical | test/unittest/schematest.cpp:3440 |
| SchemaValidator.Schema_RefEmptyString | test/unittest/schematest.cpp:3365 |
| SchemaValidator.Schema_RefNoRemoteProvider | test/unittest/schematest.cpp:3374 |
| SchemaValidator.Schema_RefNoRemoteSchema | test/unittest/schematest.cpp:3383 |
| SchemaValidator.Schema_RefPlainNameOpenApi | test/unittest/schematest.cpp:3346 |
| SchemaValidator.Schema_RefPlainNameRemote | test/unittest/schematest.cpp:3355 |
| SchemaValidator.Schema_RefPointerInvalid | test/unittest/schematest.cpp:3393 |
| SchemaValidator.Schema_RefPointerInvalidRemote | test/unittest/schematest.cpp:3402 |
| SchemaValidator.Schema_RefUnknownPlainName | test/unittest/schematest.cpp:3412 |
| SchemaValidator.Schema_RefUnknownPointer | test/unittest/schematest.cpp:3421 |
| SchemaValidator.Schema_RefUnknownPointerRemote | test/unittest/schematest.cpp:3430 |
| SchemaValidator.Schema_StartUnknown | test/unittest/schematest.cpp:3327 |
| SchemaValidator.Schema_SupportedDraft4 | test/unittest/schematest.cpp:3044 |
| SchemaValidator.Schema_SupportedDraft4NoFrag | test/unittest/schematest.cpp:3055 |
| SchemaValidator.Schema_SupportedDraft5 | test/unittest/schematest.cpp:3066 |
| SchemaValidator.Schema_SupportedDraft5NoFrag | test/unittest/schematest.cpp:3077 |
| SchemaValidator.Schema_SupportedDraft5Static | test/unittest/schematest.cpp:3033 |
| SchemaValidator.Schema_SupportedDraftOverride | test/unittest/schematest.cpp:3099 |
| SchemaValidator.Schema_SupportedNoSpec | test/unittest/schematest.cpp:3011 |
| SchemaValidator.Schema_SupportedNoSpecStatic | test/unittest/schematest.cpp:3022 |
| SchemaValidator.Schema_SupportedNotObject | test/unittest/schematest.cpp:3000 |
| SchemaValidator.Schema_SupportedVersion20 | test/unittest/schematest.cpp:3219 |
| SchemaValidator.Schema_SupportedVersion20Static | test/unittest/schematest.cpp:3208 |
| SchemaValidator.Schema_SupportedVersion30x | test/unittest/schematest.cpp:3230 |
| SchemaValidator.Schema_SupportedVersionOverride | test/unittest/schematest.cpp:3241 |
| SchemaValidator.Schema_UnknownDraft | test/unittest/schematest.cpp:3132 |
| SchemaValidator.Schema_UnknownDraftNotString | test/unittest/schematest.cpp:3143 |
| SchemaValidator.Schema_UnknownDraftOverride | test/unittest/schematest.cpp:3110 |
| SchemaValidator.Schema_UnknownVersion | test/unittest/schematest.cpp:3274 |
| SchemaValidator.Schema_UnknownVersionNotString | test/unittest/schematest.cpp:3296 |
| SchemaValidator.Schema_UnknownVersionOverride | test/unittest/schematest.cpp:3252 |
| SchemaValidator.Schema_UnknownVersionShort | test/unittest/schematest.cpp:3285 |
| SchemaValidator.Schema_UnsupportedDraft2019_09 | test/unittest/schematest.cpp:3186 |
| SchemaValidator.Schema_UnsupportedDraft2020_12 | test/unittest/schematest.cpp:3197 |
| SchemaValidator.Schema_UnsupportedDraft3 | test/unittest/schematest.cpp:3154 |
| SchemaValidator.Schema_UnsupportedDraft6 | test/unittest/schematest.cpp:3165 |
| SchemaValidator.Schema_UnsupportedDraft7 | test/unittest/schematest.cpp:3175 |
| SchemaValidator.Schema_UnsupportedDraftOverride | test/unittest/schematest.cpp:3121 |
| SchemaValidator.Schema_UnsupportedVersion31 | test/unittest/schematest.cpp:3307 |
| SchemaValidator.Schema_UnsupportedVersionOverride | test/unittest/schematest.cpp:3263 |
| SchemaValidator.String | test/unittest/schematest.cpp:448 |
| SchemaValidator.String_LengthRange | test/unittest/schematest.cpp:486 |
| SchemaValidator.String_Pattern | test/unittest/schematest.cpp:508 |
| SchemaValidator.String_Pattern_Invalid | test/unittest/schematest.cpp:529 |
| SchemaValidator.TestSuite | test/unittest/schematest.cpp:2183 |
| SchemaValidator.Typeless | test/unittest/schematest.cpp:217 |
| SchemaValidator.UnknownValidationError | test/unittest/schematest.cpp:2977 |
| SchemaValidator.ValidateMetaSchema | test/unittest/schematest.cpp:2050 |
| SchemaValidator.ValidateMetaSchema_UTF16 | test/unittest/schematest.cpp:2078 |
| SchemaValidator.WriteOnlyWhenReading | test/unittest/schematest.cpp:3487 |
| StrFunc.CountStringCodePoint | test/unittest/strfunctest.cpp:21 |
| StringBuffer.Clear | test/unittest/stringbuffertest.cpp:51 |
| StringBuffer.GetLength_Issue744 | test/unittest/stringbuffertest.cpp:89 |
| StringBuffer.InitialSize | test/unittest/stringbuffertest.cpp:26 |
| StringBuffer.MoveAssignment | test/unittest/stringbuffertest.cpp:169 |
| StringBuffer.MoveConstructor | test/unittest/stringbuffertest.cpp:141 |
| StringBuffer.Pop | test/unittest/stringbuffertest.cpp:75 |
| StringBuffer.Push | test/unittest/stringbuffertest.cpp:63 |
| StringBuffer.Put | test/unittest/stringbuffertest.cpp:33 |
| StringBuffer.PutN_Issue672 | test/unittest/stringbuffertest.cpp:42 |
| Strtod.CheckApproximationCase | test/unittest/strtodtest.cpp:28 |
| Uri.Assignment | test/unittest/uritest.cpp:311 |
| Uri.CopyConstructor | test/unittest/uritest.cpp:301 |
| Uri.DefaultConstructor | test/unittest/uritest.cpp:32 |
| Uri.Equals | test/unittest/uritest.cpp:685 |
| Uri.Issue1899 | test/unittest/uritest.cpp:714 |
| Uri.Match | test/unittest/uritest.cpp:697 |
| Uri.Parse | test/unittest/uritest.cpp:51 |
| Uri.Parse_Std | test/unittest/uritest.cpp:267 |
| Uri.Parse_UTF16 | test/unittest/uritest.cpp:158 |
| Uri.Parse_UTF16_Std | test/unittest/uritest.cpp:283 |
| Uri.Resolve | test/unittest/uritest.cpp:322 |
| Uri.Resolve_UTF16 | test/unittest/uritest.cpp:503 |
| Value.AcceptTerminationByHandler | test/unittest/valuetest.cpp:1761 |
| Value.AllocateShortString | test/unittest/valuetest.cpp:1725 |
| Value.Array | test/unittest/valuetest.cpp:1080 |
| Value.ArrayHelper | test/unittest/valuetest.cpp:1134 |
| Value.ArrayHelperRangeFor | test/unittest/valuetest.cpp:1196 |
| Value.AssignmentOperator | test/unittest/valuetest.cpp:121 |
| Value.BigNestedArray | test/unittest/valuetest.cpp:1627 |
| Value.BigNestedObject | test/unittest/valuetest.cpp:1648 |
| Value.CopyFrom | test/unittest/valuetest.cpp:283 |
| Value.DefaultConstructor | test/unittest/valuetest.cpp:38 |
| Value.Double | test/unittest/valuetest.cpp:628 |
| Value.EqualtoOperator | test/unittest/valuetest.cpp:180 |
| Value.EraseMember_String | test/unittest/valuetest.cpp:1609 |
| Value.False | test/unittest/valuetest.cpp:358 |
| Value.Float | test/unittest/valuetest.cpp:660 |
| Value.Int | test/unittest/valuetest.cpp:384 |
| Value.Int64 | test/unittest/valuetest.cpp:512 |
| Value.IsLosslessDouble | test/unittest/valuetest.cpp:697 |
| Value.IsLosslessFloat | test/unittest/valuetest.cpp:722 |
| Value.MergeDuplicateKey | test/unittest/valuetest.cpp:1830 |
| Value.MoveConstructor | test/unittest/valuetest.cpp:96 |
| Value.Null | test/unittest/valuetest.cpp:304 |
| Value.Object | test/unittest/valuetest.cpp:1493 |
| Value.ObjectHelper | test/unittest/valuetest.cpp:1515 |
| Value.ObjectHelperRangeFor | test/unittest/valuetest.cpp:1571 |
| Value.RemoveLastElement | test/unittest/valuetest.cpp:1690 |
| Value.SSOMemoryOverlapTest | test/unittest/valuetest.cpp:1860 |
| Value.SetStringNull | test/unittest/valuetest.cpp:882 |
| Value.Size | test/unittest/valuetest.cpp:26 |
| Value.Sorting | test/unittest/valuetest.cpp:1786 |
| Value.String | test/unittest/valuetest.cpp:732 |
| Value.Swap | test/unittest/valuetest.cpp:288 |
| Value.True | test/unittest/valuetest.cpp:327 |
| Value.Uint | test/unittest/valuetest.cpp:455 |
| Value.Uint64 | test/unittest/valuetest.cpp:576 |
| Write.RawValue_Issue1152 | test/unittest/writertest.cpp:571 |
| Writer.AssertIncorrectArrayLevel | test/unittest/writertest.cpp:263 |
| Writer.AssertIncorrectEndArray | test/unittest/writertest.cpp:278 |
| Writer.AssertIncorrectEndObject | test/unittest/writertest.cpp:271 |
| Writer.AssertIncorrectObjectLevel | test/unittest/writertest.cpp:255 |
| Writer.AssertMultipleRoot | test/unittest/writertest.cpp:306 |
| Writer.AssertObjectKeyNotString | test/unittest/writertest.cpp:285 |
| Writer.AssertRootMayBeAnyValue | test/unittest/writertest.cpp:236 |
| Writer.Compact | test/unittest/writertest.cpp:30 |
| Writer.Double | test/unittest/writertest.cpp:129 |
| Writer.Inf | test/unittest/writertest.cpp:515 |
| Writer.InfToNull | test/unittest/writertest.cpp:539 |
| Writer.Int | test/unittest/writertest.cpp:64 |
| Writer.Int64 | test/unittest/writertest.cpp:78 |
| Writer.InvalidEncoding | test/unittest/writertest.cpp:377 |
| Writer.InvalidEventSequence | test/unittest/writertest.cpp:433 |
| Writer.Issue_889 | test/unittest/writertest.cpp:103 |
| Writer.MoveCtor | test/unittest/writertest.cpp:617 |
| Writer.NaN | test/unittest/writertest.cpp:484 |
| Writer.NaNToNull | test/unittest/writertest.cpp:503 |
| Writer.OStreamWrapper | test/unittest/writertest.cpp:221 |
| Writer.RawValue | test/unittest/writertest.cpp:557 |
| Writer.Root | test/unittest/writertest.cpp:54 |
| Writer.RootArrayIsComplete | test/unittest/writertest.cpp:342 |
| Writer.RootObjectIsComplete | test/unittest/writertest.cpp:328 |
| Writer.RootValueIsComplete | test/unittest/writertest.cpp:356 |
| Writer.ScanWriteUnescapedString | test/unittest/writertest.cpp:116 |
| Writer.String | test/unittest/writertest.cpp:88 |
| Writer.Transcode | test/unittest/writertest.cpp:160 |
| Writer.UInt | test/unittest/writertest.cpp:70 |
| Writer.Uint64 | test/unittest/writertest.cpp:83 |
| Writer.ValidateEncoding | test/unittest/writertest.cpp:406 |
| clzll.normal | test/unittest/clzlltest.cpp:24 |
| dtoa.maxDecimalPlaces | test/unittest/dtoatest.cpp:54 |
| dtoa.normal | test/unittest/dtoatest.cpp:25 |
| itoa.i32toa | test/unittest/itoatest.cpp:146 |
| itoa.i64toa | test/unittest/itoatest.cpp:154 |
| itoa.u32toa | test/unittest/itoatest.cpp:142 |
| itoa.u64toa | test/unittest/itoatest.cpp:150 |
