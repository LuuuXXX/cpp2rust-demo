# Proposal: Fix Direct Binding Mode CI Failures

**Change ID:** `fix-direct-binding-ci`
**Created:** 2026-06-15
**Status:** Implementation Complete
**Completed:** 2026-06-15

---

## Problem Statement

CI is failing in 5 job categories due to Direct binding mode bugs in the extractor:

1. **tinyxml2 E2E** (`tinyxml2_merge_phase`): `hicc::import_lib!` produces `E0428: the name X is defined multiple times` for `xml_util_to_str`, `xml_text_new_2`, etc. Also `cannot find type` for `Whitespace`, `XMLComment`, `XMLDeclaration`, etc.

2. **gen-verify (Linux)**: 28/48 tests fail — generated Rust code cannot compile. C++ side: `std::make_unique<T>()` overload resolution failures for parameterized constructors. Rust side: `ClassMutPtr` vs `&ClassMutPtr` mismatch, `String` type collision with `std::string::String`.

3. **gen-verify (MinGW)**: 27/48 tests fail — same root causes as Linux.

4. **L5 nm symbol validation**: `cargo build` fails for 13+ examples — `std::make_unique` C++ compilation errors.

5. **Smoke tests (007_class_constructor)**: C++ compile error — cast from `__unique_ptr_t<Point> (*)()` to `std::unique_ptr<Point> (*)(int, int)` fails.

### Root Causes

| # | Root Cause | Location | Symptom |
|---|-----------|----------|---------|
| R1 | No `rust_name` deduplication in `build_direct_lib_spec` | `direct_binding.rs:424-426` | Duplicate names in `import_lib!` (e.g., 8 overloads of `xml_util_to_str`) |
| R2 | `class_names` from ALL classes (incl. abstract/internal), not just exported ones | `direct_binding.rs:255` | Methods referencing abstract classes pass `is_mappable_rust_type` but those classes have no `import_class!` block → "cannot find type" |
| R3 | `build_make_unique_factory` doesn't filter unmappable constructor parameter types | `direct_binding.rs:489-543` | Factory uses `Whitespace` type that has no Rust equivalent → "cannot find type Whitespace" |
| R4 | `std::make_unique<T>(args)` in `#[cpp(func)]` is not a resolvable function pointer | `direct_binding.rs:525-531` | `std::make_unique` is a C++ template, cannot be resolved as extern-C function pointer → cast errors |
| R5 | Enum types (unscoped C++ enums like `Whitespace`) not recognized as mappable | `is_mappable_rust_type` | `Whitespace` not in `class_names` → filtered out or used as unknown type |

## Proposed Solution

### Fix R1: Add `rust_name` deduplication

Port the dedup logic from `lib_spec.rs:44-70` to `build_direct_lib_spec`:
- Dedup by `cpp_sig` first (remove identical C++ signatures)
- Add numeric suffixes `_1`, `_2`, ... for duplicate `rust_name`s

### Fix R2: Use exported class names for mappability checks

Replace `all_classes`-derived `class_names` with `class_specs`-derived `exported_class_names` for `is_mappable_rust_type` calls in `build_direct_lib_spec`. Only classes that actually get `import_class!` blocks should be considered mappable.

### Fix R3: Filter unmappable constructor parameters

Add `is_mappable_rust_type` check on constructor parameter types in `build_make_unique_factory`. If any param type is unmappable, skip that factory (or skip the specific overloaded constructor).

### Fix R4: Generate C++ shim wrappers for parameterized constructors

For parameterized constructors, instead of `std::make_unique<T>(args)` in `#[cpp(func)]`, generate a C++ shim wrapper in the `hicc::cpp!` block:
```cpp
std::unique_ptr<Point> _cpp2rust_make_unique_Point(int x, int y) {
    return std::make_unique<Point>(x, y);
}
```
Then reference this shim in `#[cpp(func)]`:
```rust
#[cpp(func = "std::unique_ptr<Point> _cpp2rust_make_unique_Point(int, int)")]
```
Default constructors continue using `hicc::make_unique<T>()` (which works correctly).

### Fix R5: Include enum types in mappability

Add enum names from `ast.enums` to the mappable type set, so unscoped C++ enums (like `Whitespace`) are recognized as valid Rust types in `is_mappable_rust_type`.

## Scope

### In Scope
- 5 bug fixes in `src/extractor/direct_binding.rs` and `src/extractor/mod.rs`
- Update `build_direct_lib_spec` to accept enum names
- Update `hicc_codegen.rs` to emit C++ shim wrappers for parameterized constructors into `cpp_block_lines`
- Regression tests for each fix
- Verify tinyxml2 E2E, gen-verify, L5 nm, and smoke tests pass

### Out of Scope
- `ClassMutPtr` vs `&ClassMutPtr` mismatch (separate issue, needs hicc-side fix)
- `String` type collision with `std::string::String` (separate issue)
- shim mode behavior (no changes)

## Impact Analysis

| Component | Change Required | Details |
|-----------|-----------------|---------|
| src/extractor/direct_binding.rs | Yes | R1-R4 fixes |
| src/extractor/mod.rs | Yes | R5 fix (pass enum names) |
| src/generator/hicc_codegen.rs | Possible | May need to handle shim wrapper lines |
| src/ffi_model.rs | Possible | May need new field for shim wrapper lines |

## Success Criteria

- [x] tinyxml2 E2E test passes
- [x] gen-verify tests: all 48 pass on Linux, ≥ 47 pass on MinGW
- [x] L5 nm symbol validation: `cargo build` succeeds for all examples
- [x] Smoke test 007_class_constructor passes
- [x] `cargo test --lib` + `cargo clippy` + `cargo fmt --check` pass
- [x] No regression in existing shim-mode tests

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Shim wrapper naming collisions | Low | Low | Use `_cpp2rust_` prefix + class name + param count |
| Enum type mapping incomplete | Low | Low | Only add enums that are actually referenced in exported functions |
| Existing unit tests break | Low | Medium | Run full test suite before merge |
