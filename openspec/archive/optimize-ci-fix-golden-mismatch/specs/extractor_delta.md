# Delta: Extractor Direct Binding Factory Naming

**Change ID:** `optimize-ci-fix-golden-mismatch`
**Affects:** src/extractor/direct_binding.rs, L1 golden tests, examples/*/rust_hicc/src/lib.rs

---

## ADDED

### Requirement: make_unique 工厂 shim 命名规则统一化

`build_make_unique_factory` 的 C++ shim wrapper 命名规则与 rust_name 规则完全对齐：

- 默认构造函数：rust_name `<class_snake>_new`，C++ sig `hicc::make_unique<T>()`（无 shim）
- 单参数构造函数：rust_name `<class_snake>_new_with_<param>`，shim `_cpp2rust_make_unique_<class_snake>_with_<param>`
- 多参数构造函数：rust_name `<class_snake>_new_<N>`，shim `_cpp2rust_make_unique_<class_snake>_<N>`

#### Scenario: 单参数 ctor shim 命名
- GIVEN `Buffer` 类含构造函数 `Buffer(int sz)`
- WHEN `build_make_unique_factory` 生成 shim
- THEN shim 名为 `_cpp2rust_make_unique_buffer_with_sz`（非 `_cpp2rust_make_unique_buffer_sz`）
- AND `#[cpp(func)]` 引用为 `std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_sz(int)`
- AND rust_name 为 `buffer_new_with_sz`

#### Scenario: 默认 ctor 不生成 shim wrapper
- GIVEN `Buffer` 类含默认构造函数 `Buffer()`
- WHEN `build_make_unique_factory` 生成绑定
- THEN cpp_sig 为 `std::unique_ptr<Buffer> hicc::make_unique<Buffer>()`（不生成 `_0()` shim）
- AND 无 C++ shim wrapper 行返回（`Option<String>` 为 `None`）

#### Scenario: 多参数 ctor shim 命名不变
- GIVEN `Derived` 类含构造函数 `Derived(int v1, int v2, int dv)`
- WHEN `build_make_unique_factory` 生成 shim
- THEN shim 名为 `_cpp2rust_make_unique_derived_3`（数字后缀，与现有规则一致）

---

### Requirement: 复制 ctor 工厂过滤验证

确保 `build_make_unique_factory` 不为 `= delete` 复制构造函数或 `const T&` 复制构造函数生成工厂绑定。

#### Scenario: = delete 复制 ctor 不生成工厂
- GIVEN `Buffer(const Buffer&) = delete`
- WHEN `build_direct_lib_spec` 处理该构造函数
- THEN 不生成 `buffer_new_with_other` 或类似复制 ctor 工厂

#### Scenario: 非 delete 复制 ctor 过滤策略
- GIVEN `Buffer(const Buffer& other)` 且非 `= delete`
- WHEN `build_direct_lib_spec` 处理
- THEN 根据 `is_copy_ctor` 标记跳过该构造函数（不生成工厂）
- AND 仅为默认和参数化 ctor 生成工厂

---

## MODIFIED

### Requirement: Direct 模式工厂函数生成规则（原 spec: extractor.md）

原 spec 定义 shim 命名为 `_<short_param>`，现改为 `_with_<param>` 对齐 rust_name 规则。

#### Scenario: 单参数 ctor 命名变更
- GIVEN direct 模式的 `ClassSpec` 含单参数 ctor `Buffer(int sz)`
- WHEN `build_make_unique_factory` 生成工厂
- THEN C++ shim 为 `_cpp2rust_make_unique_buffer_with_sz(int sz)`
- AND `#[cpp(func)]` 为 `std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_sz(int)`
- （原规则：`_cpp2rust_make_unique_buffer_sz`）

---

## REMOVED

（None — 仅命名变更，无功能删除）
