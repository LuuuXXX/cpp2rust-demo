# Tasks: Fix Direct Binding Mode CI Failures

**Change ID:** `fix-direct-binding-ci`
**Created:** 2026-06-15
**Status:** Implementation Complete

---

## Phase 1: Extractor Bug Fixes (R1-R5)

- [x] T1: Add rust_name deduplication in build_direct_lib_spec ✓ 2026-06-15
  - Added cpp_sig dedup + rust_name numeric suffix dedup (ported from lib_spec.rs:44-70)
  - Files: `src/extractor/direct_binding.rs:446-482`

- [x] T2: Use exported class names for is_mappable_rust_type ✓ 2026-06-15
  - Changed `class_names` (from ALL classes) to `mappable_names` (from non-empty class_specs + enum names)
  - Filters factory and static method generation to only non-empty class_specs
  - Files: `src/extractor/direct_binding.rs:255-269`, `src/extractor/direct_binding.rs:382-421`, `src/extractor/direct_binding.rs:447-449`

- [x] T3: Filter unmappable constructor parameter types ✓ 2026-06-15
  - Added `is_mappable_rust_type` check on ctor param types; unmappable ctors are skipped
  - Files: `src/extractor/direct_binding.rs:394-398`

- [x] T4: Generate C++ shim wrappers for parameterized constructors ✓ 2026-06-15
  - `build_make_unique_factory` now returns `(FnBinding, Option<String>)` tuple
  - Default ctors: `hicc::make_unique<T>()` (unchanged)
  - Parameterized ctors: shim `_cpp2rust_make_unique_<snake>_<N>(args)` in `hicc::cpp!` block
  - Shim lines appended to `cpp_block_lines` in `extract()`
  - Files: `src/extractor/direct_binding.rs:540-657`, `src/extractor/mod.rs:186-193`

- [x] T5: Include enum types in mappable type set ✓ 2026-06-15
  - Added `enum_names` from `ast.enums` to `mappable_names` in both `build_direct_class_specs` and `build_direct_lib_spec`
  - Files: `src/extractor/direct_binding.rs:186-231`, `src/extractor/mod.rs:126-132,178`

**Quality Gate:** PASSED — 321 unit tests, 7 merge tests, clippy clean, fmt clean

---

## Phase 2: Example Updates

- [x] T6: Update existing examples with std::make_unique(args) pattern ✓ 2026-06-15
  - Updated 19 example files: replaced `std::make_unique<T>(args)` in `#[cpp(func)]` with C++ shim wrappers
  - Also fixed related issues in some examples (copy ctor removal, ClassMutPtr refs, String collision)
  - Files: `examples/007-047/*/rust_hicc/src/lib.rs` (19 files), some `main.rs` files

**Quality Gate:** PASSED — 48 L2 compile tests pass

---

## Phase 3: Regression Verification

- [x] T7: Full regression verification ✓ 2026-06-15
  - 321 unit tests pass
  - 7 merge integration tests pass
  - 48 L2 compile tests pass
  - 2 tinyxml2 E2E tests pass
  - `cargo clippy -- -D warnings` clean
  - `cargo fmt --check` clean

**Quality Gate:** PASSED

---

## Success Criteria Verification

- [x] tinyxml2 E2E test passes ✓
- [x] L2 compile tests: 48/48 pass ✓
- [x] Smoke test 007_class_constructor passes ✓
- [x] `cargo test --lib` + `cargo clippy` + `cargo fmt --check` pass ✓
- [x] No regression in existing shim-mode tests ✓
