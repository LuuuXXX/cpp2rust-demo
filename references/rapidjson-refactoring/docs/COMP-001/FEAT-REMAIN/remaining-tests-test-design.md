# Feature 级测试设计文档

## 报告基本信息

| 字段 | 内容 |
|------|------|
| feature ID | `FEAT-REMAIN` |
| feature 名称 | `remaining-tests` |
| 所属 component | `rapidjson-rs` (`COMP-001`) |
| 所属 crate | `rapidjson-rs` |
| 文档版本 | `0.1.0-draft` |
| 设计日期 | `2026-05-14` |
| 架构师 | `TBD` |
| 文档状态 | `草稿` |
| component 设计追溯 | [`rapidjson-rs-test-design.md`](../rapidjson-rs-test-design.md) |
| 对应开发文档 | 无（仅测试设计） |
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

本文档收拢所有不天然归属于单一 feature 的 Legacy 测试，例如 `Fwd.Fwd`、`NamespaceTest.*`、`Platform.*` 以及 `perftest/rapidjsontest.cpp` 中 跨多模块的性能场景，用于设计 component 级跨 feature 行为与性能回归的测试策略。

## 2. 基线层

### 2.1 基线层结构

_结构描述略，后续可参考其他 feature 的 2.1 段落补全_

### 2.2 基线测试清单

| 测试 ID | file | oracle 来源 |
|---------|------|-------------|
| Fwd.Fwd | test/unittest/fwdtest.cpp:224 | legacy_test |
| NamespaceTest.Direct | test/unittest/namespacetest.cpp:43 | legacy_test |
| NamespaceTest.Using | test/unittest/namespacetest.cpp:34 | legacy_test |
| Platform.GetObject | test/unittest/platformtest.cpp:29 | legacy_test |
| RapidJson.DocumentAccept | test/perftest/rapidjsontest.cpp:331 | spec + c_output |
| RapidJson.DocumentFind | test/perftest/rapidjsontest.cpp:339 | spec + c_output |
| RapidJson.DocumentParse_CrtAllocator_SSE42 | test/perftest/rapidjsontest.cpp:252 | spec + c_output |
| RapidJson.DocumentParse_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:217 | spec + c_output |
| RapidJson.DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:271 | spec + c_output |
| RapidJson.DocumentParseEncodedInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:261 | spec + c_output |
| RapidJson.DocumentParseInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:199 | spec + c_output |
| RapidJson.DocumentParseIterative_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:244 | spec + c_output |
| RapidJson.DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:208 | spec + c_output |
| RapidJson.DocumentParseLength_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:225 | spec + c_output |
| RapidJson.DocumentParseStdString_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:234 | spec + c_output |
| RapidJson.DocumentTraverse | test/perftest/rapidjsontest.cpp:304 | spec + c_output |
| RapidJson.FileReadStream | test/perftest/rapidjsontest.cpp:464 | spec + c_output |
| RapidJson.internal_Pow10 | test/perftest/rapidjsontest.cpp:421 | spec + c_output |
| RapidJson.IStreamWrapper | test/perftest/rapidjsontest.cpp:487 | spec + c_output |
| RapidJson.IStreamWrapper_Setbuffered | test/perftest/rapidjsontest.cpp:508 | spec + c_output |
| RapidJson.IStreamWrapper_Unbuffered | test/perftest/rapidjsontest.cpp:498 | spec + c_output |
| RapidJson.PrettyWriter_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:408 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_FileReadStream_SSE42 | test/perftest/rapidjsontest.cpp:475 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_FullPrecision_SSE42 | test/perftest/rapidjsontest.cpp:133 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | test/perftest/rapidjsontest.cpp:544 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_SSE42 | test/perftest/rapidjsontest.cpp:521 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | test/perftest/rapidjsontest.cpp:533 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:95 | spec + c_output |
| RapidJson.ReaderParse_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:190 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:75 | spec + c_output |
| RapidJson.ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:85 | spec + c_output |
| RapidJson.ReaderParseIterative_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:142 | spec + c_output |
| RapidJson.ReaderParseIterativeInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:151 | spec + c_output |
| RapidJson.ReaderParseIterativePull_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:161 | spec + c_output |
| RapidJson.ReaderParseIterativePullInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:175 | spec + c_output |
| RapidJson.SkipWhitespace_Basic | test/perftest/rapidjsontest.cpp:428 | spec + c_output |
| RapidJson.SkipWhitespace_SSE42 | test/perftest/rapidjsontest.cpp:437 | spec + c_output |
| RapidJson.SkipWhitespace_strspn | test/perftest/rapidjsontest.cpp:445 | spec + c_output |
| RapidJson.StringBuffer | test/perftest/rapidjsontest.cpp:558 | spec + c_output |
| RapidJson.UTF8_Validate | test/perftest/rapidjsontest.cpp:452 | spec + c_output |
| RapidJson.Writer_NullStream | test/perftest/rapidjsontest.cpp:365 | spec + c_output |
| RapidJson.Writer_StringBuffer_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:398 | spec + c_output |
| RapidJson.Writer_StringBuffer_Floats_SSE42 | test/perftest/rapidjsontest.cpp:399 | spec + c_output |
| RapidJson.Writer_StringBuffer_Guids_SSE42 | test/perftest/rapidjsontest.cpp:400 | spec + c_output |
| RapidJson.Writer_StringBuffer_Integers_SSE42 | test/perftest/rapidjsontest.cpp:401 | spec + c_output |
| RapidJson.Writer_StringBuffer_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:402 | spec + c_output |
| RapidJson.Writer_StringBuffer_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:403 | spec + c_output |
| RapidJson.Writer_StringBuffer_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:404 | spec + c_output |
| RapidJson.Writer_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:375 | spec + c_output |
| Regex.Alternation1 | test/unittest/regextest.cpp:41 | legacy_test |
| Regex.Alternation2 | test/unittest/regextest.cpp:54 | legacy_test |
| Regex.AnyCharacter | test/unittest/regextest.cpp:416 | legacy_test |
| Regex.CharacterRange1 | test/unittest/regextest.cpp:427 | legacy_test |
| Regex.CharacterRange2 | test/unittest/regextest.cpp:440 | legacy_test |
| Regex.CharacterRange3 | test/unittest/regextest.cpp:453 | legacy_test |
| Regex.CharacterRange4 | test/unittest/regextest.cpp:466 | legacy_test |
| Regex.CharacterRange5 | test/unittest/regextest.cpp:479 | legacy_test |
| Regex.CharacterRange6 | test/unittest/regextest.cpp:488 | legacy_test |
| Regex.CharacterRange7 | test/unittest/regextest.cpp:499 | legacy_test |
| Regex.CharacterRange8 | test/unittest/regextest.cpp:510 | legacy_test |
| Regex.Concatenation | test/unittest/regextest.cpp:29 | legacy_test |
| Regex.Escape | test/unittest/regextest.cpp:579 | legacy_test |
| Regex.Invalid | test/unittest/regextest.cpp:588 | legacy_test |
| Regex.Issue538 | test/unittest/regextest.cpp:629 | legacy_test |
| Regex.Issue583 | test/unittest/regextest.cpp:634 | legacy_test |
| Regex.OneOrMore1 | test/unittest/regextest.cpp:207 | legacy_test |
| Regex.OneOrMore2 | test/unittest/regextest.cpp:218 | legacy_test |
| Regex.OneOrMore3 | test/unittest/regextest.cpp:228 | legacy_test |
| Regex.OneOrMore4 | test/unittest/regextest.cpp:241 | legacy_test |
| Regex.Parenthesis1 | test/unittest/regextest.cpp:66 | legacy_test |
| Regex.Parenthesis2 | test/unittest/regextest.cpp:78 | legacy_test |
| Regex.Parenthesis3 | test/unittest/regextest.cpp:90 | legacy_test |
| Regex.QuantifierExact1 | test/unittest/regextest.cpp:251 | legacy_test |
| Regex.QuantifierExact2 | test/unittest/regextest.cpp:262 | legacy_test |
| Regex.QuantifierExact3 | test/unittest/regextest.cpp:273 | legacy_test |
| Regex.QuantifierMin1 | test/unittest/regextest.cpp:286 | legacy_test |
| Regex.QuantifierMin2 | test/unittest/regextest.cpp:298 | legacy_test |
| Regex.QuantifierMin3 | test/unittest/regextest.cpp:309 | legacy_test |
| Regex.QuantifierMinMax1 | test/unittest/regextest.cpp:322 | legacy_test |
| Regex.QuantifierMinMax2 | test/unittest/regextest.cpp:335 | legacy_test |
| Regex.QuantifierMinMax3 | test/unittest/regextest.cpp:348 | legacy_test |
| Regex.QuantifierMinMax4 | test/unittest/regextest.cpp:366 | legacy_test |
| Regex.QuantifierMinMax5 | test/unittest/regextest.cpp:385 | legacy_test |
| Regex.Search | test/unittest/regextest.cpp:521 | legacy_test |
| Regex.Search_BeginAnchor | test/unittest/regextest.cpp:537 | legacy_test |
| Regex.Search_BothAnchor | test/unittest/regextest.cpp:567 | legacy_test |
| Regex.Search_EndAnchor | test/unittest/regextest.cpp:552 | legacy_test |
| Regex.Single | test/unittest/regextest.cpp:20 | legacy_test |
| Regex.Unicode | test/unittest/regextest.cpp:406 | legacy_test |
| Regex.ZeroOrMore1 | test/unittest/regextest.cpp:160 | legacy_test |
| Regex.ZeroOrMore2 | test/unittest/regextest.cpp:171 | legacy_test |
| Regex.ZeroOrMore3 | test/unittest/regextest.cpp:182 | legacy_test |
| Regex.ZeroOrMore4 | test/unittest/regextest.cpp:196 | legacy_test |
| Regex.ZeroOrOne1 | test/unittest/regextest.cpp:103 | legacy_test |
| Regex.ZeroOrOne2 | test/unittest/regextest.cpp:112 | legacy_test |
| Regex.ZeroOrOne3 | test/unittest/regextest.cpp:124 | legacy_test |
| Regex.ZeroOrOne4 | test/unittest/regextest.cpp:136 | legacy_test |
| Regex.ZeroOrOne5 | test/unittest/regextest.cpp:150 | legacy_test |

---

## 3. 镜像测试层

### 3.1 镜像测试层结构

```text
rapidjson-refactoring/
├── migrations/
│   └── remaining-tests.legacy_to_mirror.json        # FEAT-REMAIN gtest -> Rust 镜像测试映射表
│
├── reports/
│   └── remaining-tests.mirror.junit.xml            # FEAT-REMAIN 镜像层执行结果
│
└── rapidjson-rs/
    └── tests/
        └── mirrors/
            └── remaining_tests_mirror.rs           # FEAT-REMAIN Rust 镜像测试源文件
```

### 3.2 镜像测试清单

| 测试 ID | 涉及 C/C++ 数据结构/接口 | 镜像测试 ID（Rust Test 路径） | 备注 |
|---------|-------------------------|------------------------------|------|
| Fwd.Fwd | test/unittest/fwdtest.cpp:224 | remaining_tests_mirror::Fwd_Fwd | |
| NamespaceTest.Direct | test/unittest/namespacetest.cpp:43 | remaining_tests_mirror::NamespaceTest_Direct | |
| NamespaceTest.Using | test/unittest/namespacetest.cpp:34 | remaining_tests_mirror::NamespaceTest_Using | |
| Platform.GetObject | test/unittest/platformtest.cpp:29 | remaining_tests_mirror::Platform_GetObject | |
| RapidJson.DocumentAccept | test/perftest/rapidjsontest.cpp:331 | remaining_tests_mirror::RapidJson_DocumentAccept | |
| RapidJson.DocumentFind | test/perftest/rapidjsontest.cpp:339 | remaining_tests_mirror::RapidJson_DocumentFind | |
| RapidJson.DocumentParse_CrtAllocator_SSE42 | test/perftest/rapidjsontest.cpp:252 | remaining_tests_mirror::RapidJson_DocumentParse_CrtAllocator_SSE42 | |
| RapidJson.DocumentParse_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:217 | remaining_tests_mirror::RapidJson_DocumentParse_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:271 | remaining_tests_mirror::RapidJson_DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | |
| RapidJson.DocumentParseEncodedInputStream_MemoryStream_SSE42 | test/perftest/rapidjsontest.cpp:261 | remaining_tests_mirror::RapidJson_DocumentParseEncodedInputStream_MemoryStream_SSE42 | |
| RapidJson.DocumentParseInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:199 | remaining_tests_mirror::RapidJson_DocumentParseInsitu_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentParseIterative_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:244 | remaining_tests_mirror::RapidJson_DocumentParseIterative_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:208 | remaining_tests_mirror::RapidJson_DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentParseLength_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:225 | remaining_tests_mirror::RapidJson_DocumentParseLength_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentParseStdString_MemoryPoolAllocator_SSE42 | test/perftest/rapidjsontest.cpp:234 | remaining_tests_mirror::RapidJson_DocumentParseStdString_MemoryPoolAllocator_SSE42 | |
| RapidJson.DocumentTraverse | test/perftest/rapidjsontest.cpp:304 | remaining_tests_mirror::RapidJson_DocumentTraverse | |
| RapidJson.FileReadStream | test/perftest/rapidjsontest.cpp:464 | remaining_tests_mirror::RapidJson_FileReadStream | |
| RapidJson.internal_Pow10 | test/perftest/rapidjsontest.cpp:421 | remaining_tests_mirror::RapidJson_internal_Pow10 | |
| RapidJson.IStreamWrapper | test/perftest/rapidjsontest.cpp:487 | remaining_tests_mirror::RapidJson_IStreamWrapper | |
| RapidJson.IStreamWrapper_Setbuffered | test/perftest/rapidjsontest.cpp:508 | remaining_tests_mirror::RapidJson_IStreamWrapper_Setbuffered | |
| RapidJson.IStreamWrapper_Unbuffered | test/perftest/rapidjsontest.cpp:498 | remaining_tests_mirror::RapidJson_IStreamWrapper_Unbuffered | |
| RapidJson.PrettyWriter_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:408 | remaining_tests_mirror::RapidJson_PrettyWriter_StringBuffer_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Booleans_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_FileReadStream_SSE42 | test/perftest/rapidjsontest.cpp:475 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_FileReadStream_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Floats_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_FullPrecision_SSE42 | test/perftest/rapidjsontest.cpp:133 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_FullPrecision_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Guids_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Integers_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | test/perftest/rapidjsontest.cpp:544 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_SSE42 | test/perftest/rapidjsontest.cpp:521 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | test/perftest/rapidjsontest.cpp:533 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Mixed_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Nulls_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Paragraphs_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:95 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_SSE42 | |
| RapidJson.ReaderParse_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:190 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_ValidateEncoding_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:123 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Booleans_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Floats_SSE42 | test/perftest/rapidjsontest.cpp:124 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Floats_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Guids_SSE42 | test/perftest/rapidjsontest.cpp:125 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Guids_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Integers_SSE42 | test/perftest/rapidjsontest.cpp:126 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Integers_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:127 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Mixed_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:128 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Nulls_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:129 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:75 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_SSE42 | |
| RapidJson.ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | test/perftest/rapidjsontest.cpp:85 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | |
| RapidJson.ReaderParseIterative_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:142 | remaining_tests_mirror::RapidJson_ReaderParseIterative_DummyHandler_SSE42 | |
| RapidJson.ReaderParseIterativeInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:151 | remaining_tests_mirror::RapidJson_ReaderParseIterativeInsitu_DummyHandler_SSE42 | |
| RapidJson.ReaderParseIterativePull_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:161 | remaining_tests_mirror::RapidJson_ReaderParseIterativePull_DummyHandler_SSE42 | |
| RapidJson.ReaderParseIterativePullInsitu_DummyHandler_SSE42 | test/perftest/rapidjsontest.cpp:175 | remaining_tests_mirror::RapidJson_ReaderParseIterativePullInsitu_DummyHandler_SSE42 | |
| RapidJson.SkipWhitespace_Basic | test/perftest/rapidjsontest.cpp:428 | remaining_tests_mirror::RapidJson_SkipWhitespace_Basic | |
| RapidJson.SkipWhitespace_SSE42 | test/perftest/rapidjsontest.cpp:437 | remaining_tests_mirror::RapidJson_SkipWhitespace_SSE42 | |
| RapidJson.SkipWhitespace_strspn | test/perftest/rapidjsontest.cpp:445 | remaining_tests_mirror::RapidJson_SkipWhitespace_strspn | |
| RapidJson.StringBuffer | test/perftest/rapidjsontest.cpp:558 | remaining_tests_mirror::RapidJson_StringBuffer | |
| RapidJson.UTF8_Validate | test/perftest/rapidjsontest.cpp:452 | remaining_tests_mirror::RapidJson_UTF8_Validate | |
| RapidJson.Writer_NullStream | test/perftest/rapidjsontest.cpp:365 | remaining_tests_mirror::RapidJson_Writer_NullStream | |
| RapidJson.Writer_StringBuffer_Booleans_SSE42 | test/perftest/rapidjsontest.cpp:398 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Booleans_SSE42 | |
| RapidJson.Writer_StringBuffer_Floats_SSE42 | test/perftest/rapidjsontest.cpp:399 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Floats_SSE42 | |
| RapidJson.Writer_StringBuffer_Guids_SSE42 | test/perftest/rapidjsontest.cpp:400 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Guids_SSE42 | |
| RapidJson.Writer_StringBuffer_Integers_SSE42 | test/perftest/rapidjsontest.cpp:401 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Integers_SSE42 | |
| RapidJson.Writer_StringBuffer_Mixed_SSE42 | test/perftest/rapidjsontest.cpp:402 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Mixed_SSE42 | |
| RapidJson.Writer_StringBuffer_Nulls_SSE42 | test/perftest/rapidjsontest.cpp:403 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Nulls_SSE42 | |
| RapidJson.Writer_StringBuffer_Paragraphs_SSE42 | test/perftest/rapidjsontest.cpp:404 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Paragraphs_SSE42 | |
| RapidJson.Writer_StringBuffer_SSE42 | test/perftest/rapidjsontest.cpp:375 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_SSE42 | |
| Regex.Alternation1 | test/unittest/regextest.cpp:41 | remaining_tests_mirror::Regex_Alternation1 | |
| Regex.Alternation2 | test/unittest/regextest.cpp:54 | remaining_tests_mirror::Regex_Alternation2 | |
| Regex.AnyCharacter | test/unittest/regextest.cpp:416 | remaining_tests_mirror::Regex_AnyCharacter | |
| Regex.CharacterRange1 | test/unittest/regextest.cpp:427 | remaining_tests_mirror::Regex_CharacterRange1 | |
| Regex.CharacterRange2 | test/unittest/regextest.cpp:440 | remaining_tests_mirror::Regex_CharacterRange2 | |
| Regex.CharacterRange3 | test/unittest/regextest.cpp:453 | remaining_tests_mirror::Regex_CharacterRange3 | |
| Regex.CharacterRange4 | test/unittest/regextest.cpp:466 | remaining_tests_mirror::Regex_CharacterRange4 | |
| Regex.CharacterRange5 | test/unittest/regextest.cpp:479 | remaining_tests_mirror::Regex_CharacterRange5 | |
| Regex.CharacterRange6 | test/unittest/regextest.cpp:488 | remaining_tests_mirror::Regex_CharacterRange6 | |
| Regex.CharacterRange7 | test/unittest/regextest.cpp:499 | remaining_tests_mirror::Regex_CharacterRange7 | |
| Regex.CharacterRange8 | test/unittest/regextest.cpp:510 | remaining_tests_mirror::Regex_CharacterRange8 | |
| Regex.Concatenation | test/unittest/regextest.cpp:29 | remaining_tests_mirror::Regex_Concatenation | |
| Regex.Escape | test/unittest/regextest.cpp:579 | remaining_tests_mirror::Regex_Escape | |
| Regex.Invalid | test/unittest/regextest.cpp:588 | remaining_tests_mirror::Regex_Invalid | |
| Regex.Issue538 | test/unittest/regextest.cpp:629 | remaining_tests_mirror::Regex_Issue538 | |
| Regex.Issue583 | test/unittest/regextest.cpp:634 | remaining_tests_mirror::Regex_Issue583 | |
| Regex.OneOrMore1 | test/unittest/regextest.cpp:207 | remaining_tests_mirror::Regex_OneOrMore1 | |
| Regex.OneOrMore2 | test/unittest/regextest.cpp:218 | remaining_tests_mirror::Regex_OneOrMore2 | |
| Regex.OneOrMore3 | test/unittest/regextest.cpp:228 | remaining_tests_mirror::Regex_OneOrMore3 | |
| Regex.OneOrMore4 | test/unittest/regextest.cpp:241 | remaining_tests_mirror::Regex_OneOrMore4 | |
| Regex.Parenthesis1 | test/unittest/regextest.cpp:66 | remaining_tests_mirror::Regex_Parenthesis1 | |
| Regex.Parenthesis2 | test/unittest/regextest.cpp:78 | remaining_tests_mirror::Regex_Parenthesis2 | |
| Regex.Parenthesis3 | test/unittest/regextest.cpp:90 | remaining_tests_mirror::Regex_Parenthesis3 | |
| Regex.QuantifierExact1 | test/unittest/regextest.cpp:251 | remaining_tests_mirror::Regex_QuantifierExact1 | |
| Regex.QuantifierExact2 | test/unittest/regextest.cpp:262 | remaining_tests_mirror::Regex_QuantifierExact2 | |
| Regex.QuantifierExact3 | test/unittest/regextest.cpp:273 | remaining_tests_mirror::Regex_QuantifierExact3 | |
| Regex.QuantifierMin1 | test/unittest/regextest.cpp:286 | remaining_tests_mirror::Regex_QuantifierMin1 | |
| Regex.QuantifierMin2 | test/unittest/regextest.cpp:298 | remaining_tests_mirror::Regex_QuantifierMin2 | |
| Regex.QuantifierMin3 | test/unittest/regextest.cpp:309 | remaining_tests_mirror::Regex_QuantifierMin3 | |
| Regex.QuantifierMinMax1 | test/unittest/regextest.cpp:322 | remaining_tests_mirror::Regex_QuantifierMinMax1 | |
| Regex.QuantifierMinMax2 | test/unittest/regextest.cpp:335 | remaining_tests_mirror::Regex_QuantifierMinMax2 | |
| Regex.QuantifierMinMax3 | test/unittest/regextest.cpp:348 | remaining_tests_mirror::Regex_QuantifierMinMax3 | |
| Regex.QuantifierMinMax4 | test/unittest/regextest.cpp:366 | remaining_tests_mirror::Regex_QuantifierMinMax4 | |
| Regex.QuantifierMinMax5 | test/unittest/regextest.cpp:385 | remaining_tests_mirror::Regex_QuantifierMinMax5 | |
| Regex.Search | test/unittest/regextest.cpp:521 | remaining_tests_mirror::Regex_Search | |
| Regex.Search_BeginAnchor | test/unittest/regextest.cpp:537 | remaining_tests_mirror::Regex_Search_BeginAnchor | |
| Regex.Search_BothAnchor | test/unittest/regextest.cpp:567 | remaining_tests_mirror::Regex_Search_BothAnchor | |
| Regex.Search_EndAnchor | test/unittest/regextest.cpp:552 | remaining_tests_mirror::Regex_Search_EndAnchor | |
| Regex.Single | test/unittest/regextest.cpp:20 | remaining_tests_mirror::Regex_Single | |
| Regex.Unicode | test/unittest/regextest.cpp:406 | remaining_tests_mirror::Regex_Unicode | |
| Regex.ZeroOrMore1 | test/unittest/regextest.cpp:160 | remaining_tests_mirror::Regex_ZeroOrMore1 | |
| Regex.ZeroOrMore2 | test/unittest/regextest.cpp:171 | remaining_tests_mirror::Regex_ZeroOrMore2 | |
| Regex.ZeroOrMore3 | test/unittest/regextest.cpp:182 | remaining_tests_mirror::Regex_ZeroOrMore3 | |
| Regex.ZeroOrMore4 | test/unittest/regextest.cpp:196 | remaining_tests_mirror::Regex_ZeroOrMore4 | |
| Regex.ZeroOrOne1 | test/unittest/regextest.cpp:103 | remaining_tests_mirror::Regex_ZeroOrOne1 | |
| Regex.ZeroOrOne2 | test/unittest/regextest.cpp:112 | remaining_tests_mirror::Regex_ZeroOrOne2 | |
| Regex.ZeroOrOne3 | test/unittest/regextest.cpp:124 | remaining_tests_mirror::Regex_ZeroOrOne3 | |
| Regex.ZeroOrOne4 | test/unittest/regextest.cpp:136 | remaining_tests_mirror::Regex_ZeroOrOne4 | |
| Regex.ZeroOrOne5 | test/unittest/regextest.cpp:150 | remaining_tests_mirror::Regex_ZeroOrOne5 | |

### 3.3 迁移策略建议

| 测试 ID 范围 | 进入 L1 优先级建议 | 原因 |
|--------------|--------------------|------|
| `Fwd.*`/`NamespaceTest.*`/`Platform.*` | medium | 跨命名空间/平台的 API 行为校验，建议在核心 feature 稳定后补齐镜像测试。 |
| `RapidJson.*` | medium | 性能基准与综合场景，镜像层主要用于回归对比，可按需要逐步实现。 |
| `Regex.*` | high | 内部正则引擎是多处行为的基础，建议优先补齐镜像测试以支撑后续 feature 的孪生测试。 |

### 3.4 镜像测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/remaining-tests.mirror.junit.xml` | FEAT-REMAIN 镜像层执行结果。 |
| `migrations/remaining-tests.legacy_to_mirror.json` | gtest -> Rust 镜像测试映射表。 |
| `rapidjson-rs/tests/mirrors/remaining_tests_mirror.rs` | FEAT-REMAIN 镜像测试源文件。 |

---

## 4. 孪生测试层

### 4.1 孪生测试层结构

```text
rapidjson-refactoring/
├── reports/
│   ├── remaining-tests.rust.junit.xml              # FEAT-REMAIN 孪生层执行结果
│   └── remaining-tests.parity.json                 # 镜像 vs 孪生 一致性报告
│
├── migrations/
│   └── remaining-tests.mirror_to_rust.json         # 镜像测试 -> 孪生测试映射表
│
└── rapidjson-rs/
    └── tests/
        └── rust/
            └── remaining_tests.rs                  # FEAT-REMAIN Rust 孪生测试源文件
```

### 4.2 孪生测试清单

| 测试 ID | 镜像测试 ID | 涉及 Rust 数据结构/接口 | 原生测试 ID（Test 路径） | 是否验收 | 备注 |
|---------|-------------|------------------------|---------------------------|----------|------|
| Fwd.Fwd | remaining_tests_mirror::Fwd_Fwd | remaining_tests | remaining_tests::Fwd_Fwd_rust | | |
| NamespaceTest.Direct | remaining_tests_mirror::NamespaceTest_Direct | remaining_tests | remaining_tests::NamespaceTest_Direct_rust | | |
| NamespaceTest.Using | remaining_tests_mirror::NamespaceTest_Using | remaining_tests | remaining_tests::NamespaceTest_Using_rust | | |
| Platform.GetObject | remaining_tests_mirror::Platform_GetObject | remaining_tests | remaining_tests::Platform_GetObject_rust | | |
| RapidJson.DocumentAccept | remaining_tests_mirror::RapidJson_DocumentAccept | remaining_tests | remaining_tests::RapidJson_DocumentAccept_rust | | |
| RapidJson.DocumentFind | remaining_tests_mirror::RapidJson_DocumentFind | remaining_tests | remaining_tests::RapidJson_DocumentFind_rust | | |
| RapidJson.DocumentParse_CrtAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParse_CrtAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParse_CrtAllocator_SSE42_rust | | |
| RapidJson.DocumentParse_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParse_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParse_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseAutoUTFInputStream_MemoryStream_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseAutoUTFInputStream_MemoryStream_SSE42_rust | | |
| RapidJson.DocumentParseEncodedInputStream_MemoryStream_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseEncodedInputStream_MemoryStream_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseEncodedInputStream_MemoryStream_SSE42_rust | | |
| RapidJson.DocumentParseInsitu_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseInsitu_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseInsitu_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentParseIterative_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseIterative_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseIterative_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseIterativeInsitu_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentParseLength_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseLength_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseLength_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentParseStdString_MemoryPoolAllocator_SSE42 | remaining_tests_mirror::RapidJson_DocumentParseStdString_MemoryPoolAllocator_SSE42 | remaining_tests | remaining_tests::RapidJson_DocumentParseStdString_MemoryPoolAllocator_SSE42_rust | | |
| RapidJson.DocumentTraverse | remaining_tests_mirror::RapidJson_DocumentTraverse | remaining_tests | remaining_tests::RapidJson_DocumentTraverse_rust | | |
| RapidJson.FileReadStream | remaining_tests_mirror::RapidJson_FileReadStream | remaining_tests | remaining_tests::RapidJson_FileReadStream_rust | | |
| RapidJson.internal_Pow10 | remaining_tests_mirror::RapidJson_internal_Pow10 | remaining_tests | remaining_tests::RapidJson_internal_Pow10_rust | | |
| RapidJson.IStreamWrapper | remaining_tests_mirror::RapidJson_IStreamWrapper | remaining_tests | remaining_tests::RapidJson_IStreamWrapper_rust | | |
| RapidJson.IStreamWrapper_Setbuffered | remaining_tests_mirror::RapidJson_IStreamWrapper_Setbuffered | remaining_tests | remaining_tests::RapidJson_IStreamWrapper_Setbuffered_rust | | |
| RapidJson.IStreamWrapper_Unbuffered | remaining_tests_mirror::RapidJson_IStreamWrapper_Unbuffered | remaining_tests | remaining_tests::RapidJson_IStreamWrapper_Unbuffered_rust | | |
| RapidJson.PrettyWriter_StringBuffer_SSE42 | remaining_tests_mirror::RapidJson_PrettyWriter_StringBuffer_SSE42 | remaining_tests | remaining_tests::RapidJson_PrettyWriter_StringBuffer_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Booleans_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Booleans_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Booleans_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_FileReadStream_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_FileReadStream_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_FileReadStream_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Floats_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Floats_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Floats_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_FullPrecision_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_FullPrecision_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_FullPrecision_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Guids_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Guids_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Guids_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Integers_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Integers_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Integers_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Setbuffered_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_IStreamWrapper_Unbuffered_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Mixed_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Mixed_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Mixed_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Nulls_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Nulls_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Nulls_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_Paragraphs_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_Paragraphs_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_Paragraphs_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_SSE42_rust | | |
| RapidJson.ReaderParse_DummyHandler_ValidateEncoding_SSE42 | remaining_tests_mirror::RapidJson_ReaderParse_DummyHandler_ValidateEncoding_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParse_DummyHandler_ValidateEncoding_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Booleans_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Booleans_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Booleans_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Floats_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Floats_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Floats_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Guids_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Guids_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Guids_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Integers_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Integers_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Integers_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Mixed_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Mixed_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Mixed_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Nulls_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Nulls_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Nulls_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_Paragraphs_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_Paragraphs_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_SSE42_rust | | |
| RapidJson.ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseInsitu_DummyHandler_ValidateEncoding_SSE42_rust | | |
| RapidJson.ReaderParseIterative_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseIterative_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseIterative_DummyHandler_SSE42_rust | | |
| RapidJson.ReaderParseIterativeInsitu_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseIterativeInsitu_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseIterativeInsitu_DummyHandler_SSE42_rust | | |
| RapidJson.ReaderParseIterativePull_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseIterativePull_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseIterativePull_DummyHandler_SSE42_rust | | |
| RapidJson.ReaderParseIterativePullInsitu_DummyHandler_SSE42 | remaining_tests_mirror::RapidJson_ReaderParseIterativePullInsitu_DummyHandler_SSE42 | remaining_tests | remaining_tests::RapidJson_ReaderParseIterativePullInsitu_DummyHandler_SSE42_rust | | |
| RapidJson.SkipWhitespace_Basic | remaining_tests_mirror::RapidJson_SkipWhitespace_Basic | remaining_tests | remaining_tests::RapidJson_SkipWhitespace_Basic_rust | | |
| RapidJson.SkipWhitespace_SSE42 | remaining_tests_mirror::RapidJson_SkipWhitespace_SSE42 | remaining_tests | remaining_tests::RapidJson_SkipWhitespace_SSE42_rust | | |
| RapidJson.SkipWhitespace_strspn | remaining_tests_mirror::RapidJson_SkipWhitespace_strspn | remaining_tests | remaining_tests::RapidJson_SkipWhitespace_strspn_rust | | |
| RapidJson.StringBuffer | remaining_tests_mirror::RapidJson_StringBuffer | remaining_tests | remaining_tests::RapidJson_StringBuffer_rust | | |
| RapidJson.UTF8_Validate | remaining_tests_mirror::RapidJson_UTF8_Validate | remaining_tests | remaining_tests::RapidJson_UTF8_Validate_rust | | |
| RapidJson.Writer_NullStream | remaining_tests_mirror::RapidJson_Writer_NullStream | remaining_tests | remaining_tests::RapidJson_Writer_NullStream_rust | | |
| RapidJson.Writer_StringBuffer_Booleans_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Booleans_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Booleans_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Floats_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Floats_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Floats_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Guids_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Guids_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Guids_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Integers_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Integers_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Integers_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Mixed_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Mixed_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Mixed_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Nulls_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Nulls_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Nulls_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_Paragraphs_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_Paragraphs_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_Paragraphs_SSE42_rust | | |
| RapidJson.Writer_StringBuffer_SSE42 | remaining_tests_mirror::RapidJson_Writer_StringBuffer_SSE42 | remaining_tests | remaining_tests::RapidJson_Writer_StringBuffer_SSE42_rust | | |
| Regex.Alternation1 | remaining_tests_mirror::Regex_Alternation1 | remaining_tests | remaining_tests::Regex_Alternation1_rust | | |
| Regex.Alternation2 | remaining_tests_mirror::Regex_Alternation2 | remaining_tests | remaining_tests::Regex_Alternation2_rust | | |
| Regex.AnyCharacter | remaining_tests_mirror::Regex_AnyCharacter | remaining_tests | remaining_tests::Regex_AnyCharacter_rust | | |
| Regex.CharacterRange1 | remaining_tests_mirror::Regex_CharacterRange1 | remaining_tests | remaining_tests::Regex_CharacterRange1_rust | | |
| Regex.CharacterRange2 | remaining_tests_mirror::Regex_CharacterRange2 | remaining_tests | remaining_tests::Regex_CharacterRange2_rust | | |
| Regex.CharacterRange3 | remaining_tests_mirror::Regex_CharacterRange3 | remaining_tests | remaining_tests::Regex_CharacterRange3_rust | | |
| Regex.CharacterRange4 | remaining_tests_mirror::Regex_CharacterRange4 | remaining_tests | remaining_tests::Regex_CharacterRange4_rust | | |
| Regex.CharacterRange5 | remaining_tests_mirror::Regex_CharacterRange5 | remaining_tests | remaining_tests::Regex_CharacterRange5_rust | | |
| Regex.CharacterRange6 | remaining_tests_mirror::Regex_CharacterRange6 | remaining_tests | remaining_tests::Regex_CharacterRange6_rust | | |
| Regex.CharacterRange7 | remaining_tests_mirror::Regex_CharacterRange7 | remaining_tests | remaining_tests::Regex_CharacterRange7_rust | | |
| Regex.CharacterRange8 | remaining_tests_mirror::Regex_CharacterRange8 | remaining_tests | remaining_tests::Regex_CharacterRange8_rust | | |
| Regex.Concatenation | remaining_tests_mirror::Regex_Concatenation | remaining_tests | remaining_tests::Regex_Concatenation_rust | | |
| Regex.Escape | remaining_tests_mirror::Regex_Escape | remaining_tests | remaining_tests::Regex_Escape_rust | | |
| Regex.Invalid | remaining_tests_mirror::Regex_Invalid | remaining_tests | remaining_tests::Regex_Invalid_rust | | |
| Regex.Issue538 | remaining_tests_mirror::Regex_Issue538 | remaining_tests | remaining_tests::Regex_Issue538_rust | | |
| Regex.Issue583 | remaining_tests_mirror::Regex_Issue583 | remaining_tests | remaining_tests::Regex_Issue583_rust | | |
| Regex.OneOrMore1 | remaining_tests_mirror::Regex_OneOrMore1 | remaining_tests | remaining_tests::Regex_OneOrMore1_rust | | |
| Regex.OneOrMore2 | remaining_tests_mirror::Regex_OneOrMore2 | remaining_tests | remaining_tests::Regex_OneOrMore2_rust | | |
| Regex.OneOrMore3 | remaining_tests_mirror::Regex_OneOrMore3 | remaining_tests | remaining_tests::Regex_OneOrMore3_rust | | |
| Regex.OneOrMore4 | remaining_tests_mirror::Regex_OneOrMore4 | remaining_tests | remaining_tests::Regex_OneOrMore4_rust | | |
| Regex.Parenthesis1 | remaining_tests_mirror::Regex_Parenthesis1 | remaining_tests | remaining_tests::Regex_Parenthesis1_rust | | |
| Regex.Parenthesis2 | remaining_tests_mirror::Regex_Parenthesis2 | remaining_tests | remaining_tests::Regex_Parenthesis2_rust | | |
| Regex.Parenthesis3 | remaining_tests_mirror::Regex_Parenthesis3 | remaining_tests | remaining_tests::Regex_Parenthesis3_rust | | |
| Regex.QuantifierExact1 | remaining_tests_mirror::Regex_QuantifierExact1 | remaining_tests | remaining_tests::Regex_QuantifierExact1_rust | | |
| Regex.QuantifierExact2 | remaining_tests_mirror::Regex_QuantifierExact2 | remaining_tests | remaining_tests::Regex_QuantifierExact2_rust | | |
| Regex.QuantifierExact3 | remaining_tests_mirror::Regex_QuantifierExact3 | remaining_tests | remaining_tests::Regex_QuantifierExact3_rust | | |
| Regex.QuantifierMin1 | remaining_tests_mirror::Regex_QuantifierMin1 | remaining_tests | remaining_tests::Regex_QuantifierMin1_rust | | |
| Regex.QuantifierMin2 | remaining_tests_mirror::Regex_QuantifierMin2 | remaining_tests | remaining_tests::Regex_QuantifierMin2_rust | | |
| Regex.QuantifierMin3 | remaining_tests_mirror::Regex_QuantifierMin3 | remaining_tests | remaining_tests::Regex_QuantifierMin3_rust | | |
| Regex.QuantifierMinMax1 | remaining_tests_mirror::Regex_QuantifierMinMax1 | remaining_tests | remaining_tests::Regex_QuantifierMinMax1_rust | | |
| Regex.QuantifierMinMax2 | remaining_tests_mirror::Regex_QuantifierMinMax2 | remaining_tests | remaining_tests::Regex_QuantifierMinMax2_rust | | |
| Regex.QuantifierMinMax3 | remaining_tests_mirror::Regex_QuantifierMinMax3 | remaining_tests | remaining_tests::Regex_QuantifierMinMax3_rust | | |
| Regex.QuantifierMinMax4 | remaining_tests_mirror::Regex_QuantifierMinMax4 | remaining_tests | remaining_tests::Regex_QuantifierMinMax4_rust | | |
| Regex.QuantifierMinMax5 | remaining_tests_mirror::Regex_QuantifierMinMax5 | remaining_tests | remaining_tests::Regex_QuantifierMinMax5_rust | | |
| Regex.Search | remaining_tests_mirror::Regex_Search | remaining_tests | remaining_tests::Regex_Search_rust | | |
| Regex.Search_BeginAnchor | remaining_tests_mirror::Regex_Search_BeginAnchor | remaining_tests | remaining_tests::Regex_Search_BeginAnchor_rust | | |
| Regex.Search_BothAnchor | remaining_tests_mirror::Regex_Search_BothAnchor | remaining_tests | remaining_tests::Regex_Search_BothAnchor_rust | | |
| Regex.Search_EndAnchor | remaining_tests_mirror::Regex_Search_EndAnchor | remaining_tests | remaining_tests::Regex_Search_EndAnchor_rust | | |
| Regex.Single | remaining_tests_mirror::Regex_Single | remaining_tests | remaining_tests::Regex_Single_rust | | |
| Regex.Unicode | remaining_tests_mirror::Regex_Unicode | remaining_tests | remaining_tests::Regex_Unicode_rust | | |
| Regex.ZeroOrMore1 | remaining_tests_mirror::Regex_ZeroOrMore1 | remaining_tests | remaining_tests::Regex_ZeroOrMore1_rust | | |
| Regex.ZeroOrMore2 | remaining_tests_mirror::Regex_ZeroOrMore2 | remaining_tests | remaining_tests::Regex_ZeroOrMore2_rust | | |
| Regex.ZeroOrMore3 | remaining_tests_mirror::Regex_ZeroOrMore3 | remaining_tests | remaining_tests::Regex_ZeroOrMore3_rust | | |
| Regex.ZeroOrMore4 | remaining_tests_mirror::Regex_ZeroOrMore4 | remaining_tests | remaining_tests::Regex_ZeroOrMore4_rust | | |
| Regex.ZeroOrOne1 | remaining_tests_mirror::Regex_ZeroOrOne1 | remaining_tests | remaining_tests::Regex_ZeroOrOne1_rust | | |
| Regex.ZeroOrOne2 | remaining_tests_mirror::Regex_ZeroOrOne2 | remaining_tests | remaining_tests::Regex_ZeroOrOne2_rust | | |
| Regex.ZeroOrOne3 | remaining_tests_mirror::Regex_ZeroOrOne3 | remaining_tests | remaining_tests::Regex_ZeroOrOne3_rust | | |
| Regex.ZeroOrOne4 | remaining_tests_mirror::Regex_ZeroOrOne4 | remaining_tests | remaining_tests::Regex_ZeroOrOne4_rust | | |
| Regex.ZeroOrOne5 | remaining_tests_mirror::Regex_ZeroOrOne5 | remaining_tests | remaining_tests::Regex_ZeroOrOne5_rust | | |

### 4.3 孪生测试层交付件

| 交付件 | 说明 |
|--------|------|
| `reports/remaining-tests.rust.junit.xml` | FEAT-REMAIN 孪生层执行结果。 |
| `reports/remaining-tests.parity.json` | 镜像 vs 孪生 一致性报告。 |
| `migrations/remaining-tests.mirror_to_rust.json` | 镜像测试 -> 孪生测试映射表。 |
| `rapidjson-rs/tests/rust/remaining_tests.rs` | FEAT-REMAIN 孪生测试源文件。 |
