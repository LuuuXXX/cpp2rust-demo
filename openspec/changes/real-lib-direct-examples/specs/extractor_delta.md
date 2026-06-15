# Delta: Extractor + Generator

**Change ID:** `real-lib-direct-examples`
**Affects:** src/extractor/direct_binding.rs, src/generator/hicc_codegen.rs, src/generator/smoke_test_gen.rs

---

## ADDED

### Requirement: 真实第三方库 classify 增强

`classify()` 和 `build_direct_class_specs()` 需正确处理真实第三方库的复杂 C++ 特性。

#### Scenario: template instantiation 类（如 rapidjson::Document / rapidjson::Value）
- GIVEN rapidjson 头文件含 `template<typename Encoding, typename Allocator> class GenericValue` 与 `typedef GenericValue<UTF8<char>, MemoryPoolAllocator<CrtAllocator>> Value`
- WHEN `classify()` 处理该项目的 AST
- THEN `Value` / `Document` 被识别为可绑定的 C++ 类（而非被 template 机制跳过）
- AND `build_direct_class_specs()` 为 `Value` / `Document` 生成正确的 ClassSpec

#### Scenario: internal typedef 导致的类名识别
- GIVEN `typedef GenericValue<...> Value` 在 rapidjson namespace 内
- WHEN `classify()` 处理 AST
- THEN typedef 被正确解析为具体实例化类名 `Value`
- AND 不尝试绑定模板本身 `GenericValue<Encoding, Allocator>`

#### Scenario: 大量方法类的处理（30+ 方法）
- GIVEN rapidjson::Value 含 30+ 方法（如 `IsInt()`, `GetInt()`, `SetInt()`, `operator[]` 等）
- WHEN `build_direct_class_specs()` 为 `Value` 生成 ClassSpec
- THEN 所有可映射方法被正确收集到 `methods` 字段
- AND 不可映射方法（如 `operator[]` 的 template 版本）被正确过滤

---

## MODIFIED

（无——取决于 Phase 3 中实际发现的缺陷，此处为预留框架）

---

## REMOVED

（无）
