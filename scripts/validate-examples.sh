#!/usr/bin/env bash
# validate-examples.sh
#
# End-to-end validation of cpp2rust-demo against the self-contained examples
# in the examples/ directory.  Every example that carries an entry.cpp is run
# through the full init → merge pipeline and the generated Rust output is
# checked for the patterns documented in each example's README.md.
#
# Usage:
#   ./scripts/validate-examples.sh [--release]
#
# The script can also be run locally to reproduce CI results.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
PROFILE="debug"
CARGO_FLAGS=()
for arg in "$@"; do
    case "$arg" in
        --release)
            PROFILE="release"
            CARGO_FLAGS+=(--release)
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            exit 1
            ;;
    esac
done

BIN="${REPO_ROOT}/target/${PROFILE}/cpp2rust-demo"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
FAIL=0

check_file() {
    local path="$1"
    if [ -f "$path" ]; then
        echo "  [OK]  exists: $(basename "$path")"
    else
        echo "  [FAIL] missing: $path" >&2
        FAIL=1
    fi
}

# check_contains PATH PATTERN
# Searches a single file for PATTERN.
check_contains() {
    local path="$1"
    local pattern="$2"
    if grep -q "$pattern" "$path" 2>/dev/null; then
        echo "  [OK]  '$pattern' found"
    else
        echo "  [FAIL] '$pattern' NOT found in $path" >&2
        FAIL=1
    fi
}

# check_any_rs OUT_DIR PATTERN
# Searches all *.rs files under OUT_DIR/rust/src/ for PATTERN.
check_any_rs() {
    local out="$1"
    local pattern="$2"
    if grep -rq "$pattern" "${out}/rust/src/" 2>/dev/null; then
        echo "  [OK]  '$pattern' found in rust/src/"
    else
        echo "  [FAIL] '$pattern' NOT found anywhere under ${out}/rust/src/" >&2
        FAIL=1
    fi
}

# check_any_rs_file OUT_DIR GLOB_PATTERN
# Checks at least one file matching GLOB_PATTERN exists under OUT_DIR/rust/.
check_glob_rs() {
    local out="$1"
    local glob="$2"
    local found
    found=$(find "${out}/rust" -name "$glob" 2>/dev/null | head -1)
    if [ -n "$found" ]; then
        echo "  [OK]  '$glob' found: $(basename "$found")"
    else
        echo "  [FAIL] no file matching '$glob' under ${out}/rust/" >&2
        FAIL=1
    fi
}

run_case() {
    local label="$1"
    echo ""
    echo "══════════════════════════════════════════════════════"
    echo "  Example: ${label}"
    echo "══════════════════════════════════════════════════════"
}

# merge_and_export FEATURE EXAMPLE_DIR
# Runs "merge --feature FEATURE" and exports the Rust output to
# EXAMPLE_DIR/rust/, cleaning any previous export first.
merge_and_export() {
    local feature="$1"
    local example_dir="$2"
    local dest="${REPO_ROOT}/${example_dir}/rust"
    rm -rf "${dest}"
    (cd "${REPO_ROOT}" && "${BIN}" merge --feature "${feature}" \
        -o "${dest}")
}

# ---------------------------------------------------------------------------
# Step 1: Build cpp2rust-demo
# ---------------------------------------------------------------------------
echo "=== Step 1: Building cpp2rust-demo (${PROFILE}) ==="
(cd "${REPO_ROOT}" && cargo build "${CARGO_FLAGS[@]}")
echo "Binary: ${BIN}"

# Remove any previous example outputs to make the run idempotent.
rm -rf "${REPO_ROOT}/.cpp2rust"

# ---------------------------------------------------------------------------
# Step 2: simple/ — free functions, namespace, overloads
# ---------------------------------------------------------------------------
run_case "simple/ (free functions, namespace, overloads)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature simple \
    --link mylib \
    -- clang -x c++ -fsyntax-only examples/simple/mylib.cpp < /dev/null)
merge_and_export simple examples/simple

OUT="${REPO_ROOT}/.cpp2rust/simple"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_file    "${OUT}/meta/init-interface-report.md"
check_any_rs  "${OUT}" 'link_name = "mylib"'
check_any_rs  "${OUT}" 'fn add'
check_any_rs  "${OUT}" 'fn process'
# Overloads: process_2 and process_3 must appear
check_any_rs  "${OUT}" 'fn process_2'
check_any_rs  "${OUT}" 'fn process_3'

# ---------------------------------------------------------------------------
# Step 3: class/ — classes, virtual methods, inheritance, @make_proxy
# ---------------------------------------------------------------------------
run_case "class/ (classes, virtual methods, inheritance, @make_proxy)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature widget \
    --link widget \
    -- clang -x c++ -fsyntax-only examples/class/widget.cpp < /dev/null)
merge_and_export widget examples/class

OUT="${REPO_ROOT}/.cpp2rust/widget"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'import_class!'
check_any_rs  "${OUT}" '#\[interface\]'
check_any_rs  "${OUT}" 'class Shape'
check_any_rs  "${OUT}" 'class Widget'
check_any_rs  "${OUT}" 'link_name = "widget"'
check_any_rs  "${OUT}" 'make_proxy'

# ---------------------------------------------------------------------------
# Step 4: rapidjson/01-enum/ — C/C++ enum → #[repr(C)] enum
# ---------------------------------------------------------------------------
run_case "rapidjson/01-enum/ (enum / enum class → #[repr(C)] enum)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj01 \
    --link rapidjson \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/01-enum/entry.cpp < /dev/null)
merge_and_export rj01 examples/rapidjson/01-enum

OUT="${REPO_ROOT}/.cpp2rust/rj01"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
check_any_rs  "${OUT}" 'ParseErrorCode'
check_any_rs  "${OUT}" 'WriteErrorCode'
check_any_rs  "${OUT}" 'kParseErrorNone'
check_any_rs  "${OUT}" '#\[repr(C)\]'

# ---------------------------------------------------------------------------
# Step 5: rapidjson/02-typedef-alias/ — AliasRegistry, two-map lookup
# ---------------------------------------------------------------------------
run_case "rapidjson/02-typedef-alias/ (typedef/using → AliasRegistry)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj02 \
    --link rapidjson \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/02-typedef-alias/entry.cpp < /dev/null)
merge_and_export rj02 examples/rapidjson/02-typedef-alias

OUT="${REPO_ROOT}/.cpp2rust/rj02"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_file    "${OUT}/meta/init-interface-report.md"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
# documentOk: accepts a const rjson::Document& — passes the type gate
check_any_rs  "${OUT}" 'fn document_ok'
# The types/mod.rs should have type-alias mappings for Document
check_any_rs  "${OUT}" 'Document'

# ---------------------------------------------------------------------------
# Step 6: rapidjson/03-template-class/ — template class skipped, alias suggestions
# ---------------------------------------------------------------------------
run_case "rapidjson/03-template-class/ (template class without explicit use → skipped)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj03 \
    --link rapidjson \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/03-template-class/entry.cpp < /dev/null)
merge_and_export rj03 examples/rapidjson/03-template-class

OUT="${REPO_ROOT}/.cpp2rust/rj03"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_file    "${OUT}/meta/init-interface-report.md"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
# Template classes are skipped (tool_conservative) but the report must mention them
check_contains "${OUT}/meta/init-interface-report.md" 'GenericDocument'
check_contains "${OUT}/meta/init-interface-report.md" 'template_decl'

# ---------------------------------------------------------------------------
# Step 7: rapidjson/04-abstract-interface/ — pure-virtual → #[interface] + @make_proxy
# ---------------------------------------------------------------------------
run_case "rapidjson/04-abstract-interface/ (pure-virtual class → #[interface] + @make_proxy)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj04 \
    --link rapidjson \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/rapidjson/04-abstract-interface/entry.cpp < /dev/null)
merge_and_export rj04 examples/rapidjson/04-abstract-interface

OUT="${REPO_ROOT}/.cpp2rust/rj04"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" '#\[interface\]'
check_any_rs  "${OUT}" 'class IAllocator'
check_any_rs  "${OUT}" 'make_proxy'
check_any_rs  "${OUT}" 'link_name = "rapidjson"'

# ---------------------------------------------------------------------------
# Step 8: rapidjson/05-virtual-methods/ — non-pure virtual, static data member
# ---------------------------------------------------------------------------
run_case "rapidjson/05-virtual-methods/ (non-pure virtual methods extracted like regular methods)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj05 \
    --link allocator \
    -- clang -x c++ -fsyntax-only examples/rapidjson/05-virtual-methods/entry.cpp < /dev/null)
merge_and_export rj05 examples/rapidjson/05-virtual-methods

OUT="${REPO_ROOT}/.cpp2rust/rj05"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'import_class!'
check_any_rs  "${OUT}" 'link_name = "allocator"'
check_any_rs  "${OUT}" 'BaseAllocator'
# can_free() and align() return POD types — must be extracted
check_any_rs  "${OUT}" 'fn can_free'
check_any_rs  "${OUT}" 'fn align'

# ---------------------------------------------------------------------------
# Step 9: rapidjson/06-inheritance/ — public inheritance chain
# ---------------------------------------------------------------------------
run_case "rapidjson/06-inheritance/ (public inheritance: PrettyWriterImpl: WriterBase)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj06 \
    --link writer \
    -- clang -x c++ -fsyntax-only examples/rapidjson/06-inheritance/entry.cpp < /dev/null)
merge_and_export rj06 examples/rapidjson/06-inheritance

OUT="${REPO_ROOT}/.cpp2rust/rj06"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'import_class!'
check_any_rs  "${OUT}" 'link_name = "writer"'
check_any_rs  "${OUT}" 'class WriterBase'
check_any_rs  "${OUT}" 'class PrettyWriterImpl'
# Inheritance syntax: "class PrettyWriterImpl: WriterBase"
check_any_rs  "${OUT}" 'PrettyWriterImpl: WriterBase'

# ---------------------------------------------------------------------------
# Step 10: rapidjson/07-operator-shim/ — operator overload shim generation
# ---------------------------------------------------------------------------
run_case "rapidjson/07-operator-shim/ (operator overload → shim_ops.rs + operator_shims.hpp)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj07 \
    --link jsonvalue \
    -- clang -x c++ -fsyntax-only examples/rapidjson/07-operator-shim/entry.cpp < /dev/null)
merge_and_export rj07 examples/rapidjson/07-operator-shim

OUT="${REPO_ROOT}/.cpp2rust/rj07"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "jsonvalue"'
# The C++ shim header must be generated
check_file    "${OUT}/meta/operator_shims.hpp"
check_contains "${OUT}/meta/operator_shims.hpp" 'Auto-generated operator shims'
# The Rust shim bindings skeleton must be generated
check_glob_rs "${OUT}" "shim_ops.rs"

# ---------------------------------------------------------------------------
# Step 11: rapidjson/08-multi-tu/ — multiple translation units + merge
# ---------------------------------------------------------------------------
run_case "rapidjson/08-multi-tu/ (multiple TUs: document + writer + errors)"
FEATURE="rj08"
for tu in entry-document.cpp entry-writer.cpp entry-errors.cpp; do
    (cd "${REPO_ROOT}" && "${BIN}" init \
        --feature "${FEATURE}" \
        --link rapidjson \
        --no-link \
        -- clang -x c++ -fsyntax-only "examples/rapidjson/08-multi-tu/${tu}" < /dev/null)
done
merge_and_export "${FEATURE}" examples/rapidjson/08-multi-tu

OUT="${REPO_ROOT}/.cpp2rust/${FEATURE}"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
# Each TU must produce a distinct group directory under src.1
check_file    "${OUT}/rust/src.1/mod_examples_rapidjson_08_multi_tu_entry_document/include/mod.rs"
check_file    "${OUT}/rust/src.1/mod_examples_rapidjson_08_multi_tu_entry_writer/include/mod.rs"
check_file    "${OUT}/rust/src.1/mod_examples_rapidjson_08_multi_tu_entry_errors/include/mod.rs"
# Errors TU contains the ParseErrorCode enum
check_any_rs  "${OUT}" 'ParseErrorCode'

# ---------------------------------------------------------------------------
# Step 12: conditional/01-template-no-alias/ — template skipped without alias
# ---------------------------------------------------------------------------
run_case "conditional/01-template-no-alias/ (template class skipped; report suggests aliases)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature cond01 \
    --link stack \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/conditional/01-template-no-alias/entry.cpp < /dev/null)
merge_and_export cond01 examples/conditional/01-template-no-alias

OUT="${REPO_ROOT}/.cpp2rust/cond01"
check_file    "${OUT}/meta/init-interface-report.md"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "stack"'
# Stack<T> has no alias → skipped with tool_conservative; must appear in report
check_contains "${OUT}/meta/init-interface-report.md" 'Stack'

# ---------------------------------------------------------------------------
# Step 13: conditional/02-function-template/ — function template skipped
# ---------------------------------------------------------------------------
run_case "conditional/02-function-template/ (function template skipped without explicit specialization)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature cond02 \
    --link algorithms \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/conditional/02-function-template/entry.cpp < /dev/null)
merge_and_export cond02 examples/conditional/02-function-template

OUT="${REPO_ROOT}/.cpp2rust/cond02"
check_file    "${OUT}/meta/init-interface-report.md"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "algorithms"'
# Template functions are skipped; report must mention them
check_contains "${OUT}/meta/init-interface-report.md" 'clamp'

# ---------------------------------------------------------------------------
# Step 14: semi-auto/01-dynamic-cast/ — dynamic_cast commented skeleton
# ---------------------------------------------------------------------------
run_case "semi-auto/01-dynamic-cast/ (Dog/Cat classes extracted; dynamic_cast skeleton)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature sa01 \
    --link animals \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/semi-auto/01-dynamic-cast/entry.cpp < /dev/null)
merge_and_export sa01 examples/semi-auto/01-dynamic-cast

OUT="${REPO_ROOT}/.cpp2rust/sa01"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "animals"'
# Dog and Cat must appear in the class bindings
check_any_rs  "${OUT}" 'class Dog'
check_any_rs  "${OUT}" 'class Cat'

# ---------------------------------------------------------------------------
# Step 15: semi-auto/02-placement-new/ — placement new commented skeleton
# ---------------------------------------------------------------------------
run_case "semi-auto/02-placement-new/ (FixedBuffer extracted; placement_new.rs skeleton generated)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature sa02 \
    --link fixed_buffer \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/semi-auto/02-placement-new/entry.cpp < /dev/null)
merge_and_export sa02 examples/semi-auto/02-placement-new

OUT="${REPO_ROOT}/.cpp2rust/sa02"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "fixed_buffer"'
check_any_rs  "${OUT}" 'FixedBuffer'
# The placement_new.rs skeleton is generated for concrete classes with ctors
check_glob_rs "${OUT}" "placement_new.rs"

# ---------------------------------------------------------------------------
# Step 16: guided/01-std-string/ — std::string skipped; POD method extracted
# ---------------------------------------------------------------------------
run_case "guided/01-std-string/ (std::string params skipped; prefix() extracted)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature g01 \
    --link string_utils \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/guided/01-std-string/entry.cpp < /dev/null)
merge_and_export g01 examples/guided/01-std-string

OUT="${REPO_ROOT}/.cpp2rust/g01"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "string_utils"'
# StringProcessor class must be extracted (it has a POD ctor and std::string methods)
check_any_rs  "${OUT}" 'class StringProcessor'
# std::string-parameter methods and the StringProcessor report
check_file    "${OUT}/meta/init-interface-report.md"
check_contains "${OUT}/meta/init-interface-report.md" 'StringProcessor'

# ---------------------------------------------------------------------------
# Step 17: guided/02-std-function/ — std::function skipped; POD methods extracted
# ---------------------------------------------------------------------------
run_case "guided/02-std-function/ (std::function params skipped; emit/handler_count extracted)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature g02 \
    --link event_emitter \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/guided/02-std-function/entry.cpp < /dev/null)
merge_and_export g02 examples/guided/02-std-function

OUT="${REPO_ROOT}/.cpp2rust/g02"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "event_emitter"'
# POD methods must be extracted even when other methods are skipped
check_any_rs  "${OUT}" 'fn emit'
check_any_rs  "${OUT}" 'fn handler_count'
check_file    "${OUT}/meta/init-interface-report.md"

# ---------------------------------------------------------------------------
# Step 18: guided/03-function-pointer/ — function pointer skipped; POD methods extracted
# ---------------------------------------------------------------------------
run_case "guided/03-function-pointer/ (function-pointer params skipped; dispatch/reset extracted)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature g03 \
    --link dispatcher \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/guided/03-function-pointer/entry.cpp < /dev/null)
merge_and_export g03 examples/guided/03-function-pointer

OUT="${REPO_ROOT}/.cpp2rust/g03"
check_file    "${OUT}/rust/src/merged_ffi.rs"
check_any_rs  "${OUT}" 'link_name = "dispatcher"'
# dispatch() and reset() use only int/void params — must be extracted
check_any_rs  "${OUT}" 'fn dispatch'
check_any_rs  "${OUT}" 'fn reset'
check_file    "${OUT}/meta/init-interface-report.md"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "══════════════════════════════════════════════════════"
echo "  Generated .cpp2rust feature summary"
echo "══════════════════════════════════════════════════════"
find "${REPO_ROOT}/.cpp2rust" -name "merged_ffi.rs" | sort | while read f; do
    feature=$(echo "$f" | sed "s|${REPO_ROOT}/.cpp2rust/||" | cut -d/ -f1)
    size=$(wc -l < "$f")
    printf "  %-12s  merged_ffi.rs  %d lines\n" "$feature" "$size"
done

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "VALIDATION FAILED — see errors above." >&2
    exit 1
fi

echo ""
echo "✓ All example validation checks passed."
