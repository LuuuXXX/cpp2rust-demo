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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
check_file    "${OUT}/meta/init-interface-report.md"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
# documentOk: accepts a const rjson::Document& — passes the type gate
check_any_rs  "${OUT}" 'fn document_ok'
# The flat <stem>.rs should have type-alias mappings for Document
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'import_class!'
check_any_rs  "${OUT}" 'link_name = "writer"'
check_any_rs  "${OUT}" 'class WriterBase'
check_any_rs  "${OUT}" 'class PrettyWriterImpl'
# Inheritance syntax: "class PrettyWriterImpl: WriterBase"
check_any_rs  "${OUT}" 'PrettyWriterImpl: WriterBase'

# ---------------------------------------------------------------------------
# Step 10: rapidjson/07-operator-shim/ — operator overload shim generation
# ---------------------------------------------------------------------------
run_case "rapidjson/07-operator-shim/ (operator overload → entry.rs shim stubs + operator_shims.hpp)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature rj07 \
    --link jsonvalue \
    -- clang -x c++ -fsyntax-only examples/rapidjson/07-operator-shim/entry.cpp < /dev/null)
merge_and_export rj07 examples/rapidjson/07-operator-shim

OUT="${REPO_ROOT}/.cpp2rust/rj07"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "jsonvalue"'
# The C++ shim header must be generated
check_file    "${OUT}/meta/operator_shims.hpp"
check_contains "${OUT}/meta/operator_shims.hpp" 'Auto-generated operator shims'
# The Rust shim bindings skeleton must be generated inline in the flat module
check_contains "${OUT}/rust/src.1/entry.rs" 'operator shims'

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
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "rapidjson"'
# Each TU must produce a distinct flat .rs file under src.1 (new flat layout)
check_file    "${OUT}/rust/src.1/entry_document.rs"
check_file    "${OUT}/rust/src.1/entry_writer.rs"
check_file    "${OUT}/rust/src.1/entry_errors.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "animals"'
# Dog and Cat must appear in the class bindings
check_any_rs  "${OUT}" 'class Dog'
check_any_rs  "${OUT}" 'class Cat'

# ---------------------------------------------------------------------------
# Step 15: semi-auto/02-placement-new/ — placement new commented skeleton
# ---------------------------------------------------------------------------
run_case "semi-auto/02-placement-new/ (FixedBuffer extracted; placement-new skeleton appended to entry.rs)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature sa02 \
    --link fixed_buffer \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/semi-auto/02-placement-new/entry.cpp < /dev/null)
merge_and_export sa02 examples/semi-auto/02-placement-new

OUT="${REPO_ROOT}/.cpp2rust/sa02"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "fixed_buffer"'
check_any_rs  "${OUT}" 'FixedBuffer'
# The placement_new skeleton is generated inline in the flat module
check_contains "${OUT}/rust/src.1/entry.rs" '@placement_new'

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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
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
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "dispatcher"'
# dispatch() and reset() use only int/void params — must be extracted
check_any_rs  "${OUT}" 'fn dispatch'
check_any_rs  "${OUT}" 'fn reset'
check_file    "${OUT}/meta/init-interface-report.md"

# ---------------------------------------------------------------------------
# Step 19a: conditional/03-chained-alias/ (STEP A) — no alias, Store<T> skipped
# ---------------------------------------------------------------------------
run_case "conditional/03-chained-alias/ STEP A (no alias → Store<T> skipped, tool_conservative)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature cond03a \
    --link store \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/conditional/03-chained-alias/entry.cpp < /dev/null)
merge_and_export cond03a examples/conditional/03-chained-alias

OUT="${REPO_ROOT}/.cpp2rust/cond03a"
check_file    "${OUT}/meta/init-interface-report.md"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "store"'
# Store<T> has no alias here → must appear in the skipped section of the report
check_contains "${OUT}/meta/init-interface-report.md" 'Store'

# ---------------------------------------------------------------------------
# Step 19b: conditional/03-chained-alias/ (STEP B2) — chained alias, transitive resolve
# ---------------------------------------------------------------------------
run_case "conditional/03-chained-alias/ STEP B2 (chained alias → Store_i32 + IntStore + MyStore extracted)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature cond03b \
    --link store \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/conditional/03-chained-alias/entry-chained.cpp < /dev/null)
merge_and_export cond03b examples/conditional/03-chained-alias

OUT="${REPO_ROOT}/.cpp2rust/cond03b"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "store"'
# AliasRegistry::resolve_transitive() must produce both alias type entries
check_any_rs  "${OUT}" 'IntStore'
check_any_rs  "${OUT}" 'MyStore'
# The concrete template specialization must be extracted with the alias name
check_any_rs  "${OUT}" 'class IntStore'
# Free functions using the aliased types must be extracted
check_any_rs  "${OUT}" 'fn has_entry'
check_any_rs  "${OUT}" 'fn count_entries'

# ---------------------------------------------------------------------------
# Step 20: features/01-inline-functions/ — inline functions are transparent
# ---------------------------------------------------------------------------
run_case "features/01-inline-functions/ (inline functions extracted like non-inline)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat01 \
    --link math \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/features/01-inline-functions/entry.cpp < /dev/null)
merge_and_export feat01 examples/features/01-inline-functions

OUT="${REPO_ROOT}/.cpp2rust/feat01"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "math"'
# Both inline and non-inline functions must be extracted identically
check_any_rs  "${OUT}" 'fn add'
check_any_rs  "${OUT}" 'fn mul'
check_any_rs  "${OUT}" 'fn subtract'
# Overloaded clamp → clamp + clamp_2
check_any_rs  "${OUT}" 'fn clamp'
check_any_rs  "${OUT}" 'fn clamp_2'

# ---------------------------------------------------------------------------
# Step 21: features/02-default-params/ — default parameter values are ignored
# ---------------------------------------------------------------------------
run_case "features/02-default-params/ (default params extracted with full signature, defaults dropped)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat02 \
    --link config \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/features/02-default-params/entry.cpp < /dev/null)
merge_and_export feat02 examples/features/02-default-params

OUT="${REPO_ROOT}/.cpp2rust/feat02"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "config"'
# Functions with default params must be extracted with full param list
check_any_rs  "${OUT}" 'fn set_timeout'
check_any_rs  "${OUT}" 'fn lerp'
check_any_rs  "${OUT}" 'fn log'

# ---------------------------------------------------------------------------
# Step 22: features/03-rvalue-ref/ — && methods map to fn foo(self)
# ---------------------------------------------------------------------------
run_case "features/03-rvalue-ref/ (rvalue-ref method && → fn build(self))"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat03 \
    --link builder \
    -- clang -x c++ -fsyntax-only examples/features/03-rvalue-ref/entry.cpp < /dev/null)
merge_and_export feat03 examples/features/03-rvalue-ref

OUT="${REPO_ROOT}/.cpp2rust/feat03"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "builder"'
# const method → &self
check_any_rs  "${OUT}" 'fn get(&self)'
# mutable lvalue method → &mut self
check_any_rs  "${OUT}" 'fn set(&mut self'
# rvalue-ref method → self (consuming)
check_any_rs  "${OUT}" 'fn build(self)'
# The #[cpp(method = "...")] attribute must include && qualifier
check_any_rs  "${OUT}" 'method = "int build() &&"'

# ---------------------------------------------------------------------------
# Step 23: features/04-va-list/ — va_list last param → unsafe fn with ...
# ---------------------------------------------------------------------------
run_case "features/04-va-list/ (va_list last param → unsafe fn + trailing ...)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat04 \
    --link logger \
    --no-link \
    -- clang -x c++ -fsyntax-only examples/features/04-va-list/entry.cpp < /dev/null)
merge_and_export feat04 examples/features/04-va-list

OUT="${REPO_ROOT}/.cpp2rust/feat04"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "logger"'
# va_list functions must be emitted as unsafe fn with trailing ...
check_any_rs  "${OUT}" 'unsafe fn log_message'
check_any_rs  "${OUT}" 'unsafe fn format_string'
# Trailing ... must appear
check_any_rs  "${OUT}" '\.\.\.'
# Normal function must also be extracted
check_any_rs  "${OUT}" 'fn flush'

# ---------------------------------------------------------------------------
# Step 24: features/05-global-vars/ — global variables → #[cpp(data)]
# ---------------------------------------------------------------------------
run_case "features/05-global-vars/ (global variables → #[cpp(data = ...)] + &'static bindings)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat05 \
    --link metrics \
    -- clang -x c++ -fsyntax-only examples/features/05-global-vars/entry.cpp < /dev/null)
merge_and_export feat05 examples/features/05-global-vars

OUT="${REPO_ROOT}/.cpp2rust/feat05"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "metrics"'
# Mutable global → &'static mut
check_any_rs  "${OUT}" 'g_request_count'
check_any_rs  "${OUT}" "'static mut"
# Const global → &'static (without mut)
check_any_rs  "${OUT}" 'g_max_latency_ms'
check_any_rs  "${OUT}" "'static f64"

# ---------------------------------------------------------------------------
# Step 25: features/06-static-members/ — static class data members → #[cpp(data)]
# ---------------------------------------------------------------------------
run_case "features/06-static-members/ (static class members → #[cpp(data = \"Class::member\")])"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat06 \
    --link counter \
    -- clang -x c++ -fsyntax-only examples/features/06-static-members/entry.cpp < /dev/null)
merge_and_export feat06 examples/features/06-static-members

OUT="${REPO_ROOT}/.cpp2rust/feat06"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "counter"'
# Static members must use fully-qualified class::member form
check_any_rs  "${OUT}" 'Counter::instance_count'
check_any_rs  "${OUT}" 'Counter::max_count'
check_any_rs  "${OUT}" 'cpp(data'

# ---------------------------------------------------------------------------
# Step 26: features/07-instance-fields/ — instance fields → #[cpp(field)]
# ---------------------------------------------------------------------------
run_case "features/07-instance-fields/ (public instance fields → #[cpp(field = \"Class::field\")] accessors)"
(cd "${REPO_ROOT}" && "${BIN}" init \
    --feature feat07 \
    --link point \
    -- clang -x c++ -fsyntax-only examples/features/07-instance-fields/entry.cpp < /dev/null)
merge_and_export feat07 examples/features/07-instance-fields

OUT="${REPO_ROOT}/.cpp2rust/feat07"
check_file    "${OUT}/rust/src/lib.rs"
check_any_rs  "${OUT}" 'link_name = "point"'
# Field accessors must be generated for x and y
check_any_rs  "${OUT}" 'fn get_x'
check_any_rs  "${OUT}" 'fn get_y'
# Mutable fields also get a _mut variant
check_any_rs  "${OUT}" 'fn get_x_mut'
# const field id must have only a getter (no _mut)
check_any_rs  "${OUT}" 'fn get_id'
check_any_rs  "${OUT}" 'cpp(field'

echo ""
echo "══════════════════════════════════════════════════════"
echo "  Generated .cpp2rust feature summary"
echo "══════════════════════════════════════════════════════"
find "${REPO_ROOT}/.cpp2rust" -name "lib.rs" -path "*/src/lib.rs" | sort | while read f; do
    feature=$(echo "$f" | sed "s|${REPO_ROOT}/.cpp2rust/||" | cut -d/ -f1)
    size=$(wc -l < "$f")
    printf "  %-12s  lib.rs (FFI entry)  %d lines\n" "$feature" "$size"
done

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "VALIDATION FAILED — see errors above." >&2
    exit 1
fi

echo ""
echo "✓ All example validation checks passed."
