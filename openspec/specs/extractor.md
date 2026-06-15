# Extractor + Generator Specification

**Change ID:** `slim-and-hicc-direct-binding`
**Created:** 2026-06-15

---

## Requirements

### Requirement: 直接绑定模式（direct_binding）

新增 `src/extractor/direct_binding.rs`，自动识别 C++ 项目是否提供 extern-C shim 函数，决定走 shim 还是 direct 提取路径。

#### Scenario: 纯 direct 项目（仅原生 C++ 类，无 extern-C shim）
- GIVEN 一个 C++ 项目，仅含 `class Counter { int get() const; void inc(); }`，**不含** `counter_get(struct Counter*)` 等自由函数
- WHEN `extractor::extract` 处理该项目的 AST
- THEN `FfiSpec.binding_mode == BindingMode::Direct`
- AND `FfiSpec.class_specs[0].methods` 直接含 `get` / `inc` 等方法的签名
- AND `FfiSpec.lib_spec.fn_bindings` 不含方法访问器（仍可能含真正的全局函数）

#### Scenario: 纯 shim 项目（含 extern-C 访问器，向后兼容）
- GIVEN 一个 C++ 项目，含 `class Counter { ... }` 与 `extern "C" int counter_get(struct Counter*)` 等配对自由函数
- WHEN `extractor::extract` 处理该项目
- THEN `FfiSpec.binding_mode == BindingMode::Shim`（默认值，与现状一致）
- AND 行为与改造前完全一致

#### Scenario: 混合项目（部分类有 shim，部分类无）
- GIVEN 一个项目，`class A` 配对 `a_*` 自由函数，`class B` 无配对 shim
- WHEN `extractor::extract` 处理该项目
- THEN 整体 `binding_mode == BindingMode::Shim`（保守策略，向后兼容）

---

### Requirement: FfiSpec 增加 binding_mode 字段

`src/ffi_model.rs` 中的 `FfiSpec` 结构新增字段：

```rust
pub enum BindingMode {
    Shim,
    Direct,
}

pub struct FfiSpec {
    // ... 现有字段 ...
    pub binding_mode: BindingMode,
}
```

#### Scenario: 默认值
- GIVEN 任何代码路径构造 `FfiSpec::default()`
- THEN `binding_mode == BindingMode::Shim`（保证向后兼容）

#### Scenario: extract 完成后填充
- GIVEN `extractor::extract` 成功返回
- THEN `FfiSpec.binding_mode` 已被 `direct_binding::classify` 填充

---

### Requirement: hicc_codegen 支持 direct 模式输出

`src/generator/hicc_codegen.rs::generate` 根据 `binding_mode` 选择输出格式。

#### Scenario: Direct 模式输出格式
- GIVEN `FfiSpec { binding_mode: Direct, class_specs: [Counter { methods: [get, inc] }] }`
- WHEN 调用 `hicc_codegen::generate(&spec)`
- THEN 输出包含 `hicc::import_class!` 带方法绑定 + `hicc::import_lib!` 带 `make_unique<T>` 工厂
- AND 输出**不**包含 `counter_get` / `counter_inc` 等访问器绑定

#### Scenario: Shim 模式输出格式（保持不变）
- GIVEN `FfiSpec { binding_mode: Shim, ... }`
- WHEN 调用 `hicc_codegen::generate(&spec)`
- THEN 输出格式与改造前完全一致

---

### Requirement: 冒烟测试增强 —— assert 断言

`src/generator/smoke_test_gen.rs` 对零参工厂/方法/全局函数生成 assert 断言测试。

#### Scenario: 零参工厂函数 + 零参方法 → 完整 assert 测试
- GIVEN `ClassSpec { name: "Counter", associated_fns: [counter_new()], methods: [count() -> i32] }`
- WHEN 调用 `smoke_test_gen::generate_smoke_test`
- THEN 输出包含 assert 断言（原始类型零参方法有默认值）

#### Scenario: 无法判断返回值默认值时保留原行为
- GIVEN 一个零参方法 `fn foo() -> SomeOpaqueType`
- WHEN 调用 `smoke_test_gen::generate_smoke_test`
- THEN 输出保留 `let _result = obj.foo();`

---

### Requirement: ClassSpec 的 methods 字段语义扩展

`ClassSpec.methods` 在 shim 模式下可能为空，在 direct 模式下直接含所有方法签名。

#### Scenario: direct 模式下 methods 必须非空（若类有方法）
- GIVEN direct 模式的 `ClassSpec`
- WHEN 该类在 C++ 中含至少一个非 ctor/dtor 方法
- THEN `ClassSpec.methods` 长度 ≥ 1

#### Scenario: shim 模式下 methods 可为空
- GIVEN shim 模式的 `ClassSpec`
- WHEN 该类的方法通过 `counter_*` 自由函数访问
- THEN `ClassSpec.methods` 可能为空

---

### Requirement: Direct 模式工厂函数生成规则

- 默认构造函数 → `hicc::make_unique<T>()`
- 带参数构造函数 → `std::make_unique<T>(arg_types)`
- 移动构造函数（含 `&&`） → 跳过
- 复制构造函数（`= delete`） → 通过 `is_deleted_ctor` 过滤
- 纯静态方法类 → 无工厂函数

#### Scenario: 复制构造函数 = delete 过滤
- GIVEN `Buffer(const Buffer&) = delete`（`is_copy_ctor=true, is_default=false, body_offset=None`）
- WHEN `build_direct_lib_spec` 生成工厂
- THEN 不生成 `std::make_unique<Buffer>(const Buffer&)` 工厂

#### Scenario: 纯静态方法类无工厂
- GIVEN `SumCalculator` 类仅含静态方法（`has_only_static_methods=true`）
- WHEN `build_direct_lib_spec` 生成工厂
- THEN 不生成 `SumCalculator` 工厂，仅生成 `import_lib!` 静态方法函数

---

## Deprecated

（无）
