#!/usr/bin/env bash
# validate-rapidjson.sh
#
# End-to-end validation of cpp2rust-demo against Tencent/rapidjson.
#
# Usage:
#   ./scripts/validate-rapidjson.sh [--release]
#
# The script:
#   1. Builds cpp2rust-demo (debug by default, --release for release build).
#   2. Clones Tencent/rapidjson into /tmp/rapidjson (re-uses existing clone).
#   3. Runs `cpp2rust-demo init` inside the rapidjson directory via multiple
#      translation units to trigger the complete “compile→capture→middleware” flow.
#   4. Runs `cpp2rust-demo merge`.
#   5. Validates the expected output files exist and contain expected content.
#
# This script mirrors the CI workflow in .github/workflows/validate-rapidjson.yml
# and can be run locally to reproduce CI results.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

RAPIDJSON_DIR="/tmp/rapidjson"
# Feature name passed to cpp2rust-demo --feature; also used to locate the
# output directory .cpp2rust/<FEATURE>/ during validation.
FEATURE="default"

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
# Step 1: Build cpp2rust-demo
# ---------------------------------------------------------------------------
echo "=== Step 1: Building cpp2rust-demo (${PROFILE}) ==="
(cd "${REPO_ROOT}" && cargo build "${CARGO_FLAGS[@]}")
echo "Binary: ${BIN}"
echo ""

# ---------------------------------------------------------------------------
# Step 2: Clone Tencent/rapidjson (shallow, reuse existing clone)
# ---------------------------------------------------------------------------
echo "=== Step 2: Preparing Tencent/rapidjson ==="
if [ -d "${RAPIDJSON_DIR}/.git" ]; then
    echo "Re-using existing clone at ${RAPIDJSON_DIR}"
else
    echo "Cloning Tencent/rapidjson into ${RAPIDJSON_DIR} ..."
    git clone --depth=1 https://github.com/Tencent/rapidjson.git "${RAPIDJSON_DIR}"
fi
echo ""

# rapidjson validation cases by capability dimension:
# - document     => DOM/document
# - reader       => Reader/SAX
# - writer       => Writer
# - prettywriter => PrettyWriter
# - pointer      => Pointer
# - schema       => Schema/Error related
CASE_NAMES=(document reader writer prettywriter pointer schema)
CASE_INCLUDES=(
    'rapidjson/document.h'
    'rapidjson/reader.h'
    'rapidjson/writer.h'
    'rapidjson/prettywriter.h'
    'rapidjson/pointer.h'
    'rapidjson/schema.h'
)

# ---------------------------------------------------------------------------
# Step 3: Run cpp2rust-demo init
# ---------------------------------------------------------------------------
echo "=== Step 3: Running cpp2rust-demo init ==="
(
    cd "${RAPIDJSON_DIR}"
    # Remove previous output so this script is idempotent.
    rm -rf .cpp2rust
    BUILD_STEPS=()
    for i in "${!CASE_NAMES[@]}"; do
        case_name="${CASE_NAMES[$i]}"
        include_header="${CASE_INCLUDES[$i]}"
        source_file="${case_name}.cpp"
        cat > "${source_file}" <<CPP
#include "${include_header}"
CPP
        # Keep one shared compile profile across cases for stable CI behavior:
        # rapidjson is header-only and supports C++11 as a baseline.
        BUILD_STEPS+=("clang++ -x c++ -std=c++11 -fsyntax-only -Iinclude ${source_file}")
    done
    BUILD_CMD=""
    for step in "${BUILD_STEPS[@]}"; do
        if [ -n "${BUILD_CMD}" ]; then
            BUILD_CMD="${BUILD_CMD} && "
        fi
        BUILD_CMD="${BUILD_CMD}${step}"
    done
    "${BIN}" init \
        --feature "${FEATURE}" \
        --link rapidjson \
        -- sh -c "${BUILD_CMD}" < /dev/null
)
echo ""

# ---------------------------------------------------------------------------
# Step 4: Run cpp2rust-demo merge
# ---------------------------------------------------------------------------
echo "=== Step 4: Running cpp2rust-demo merge ==="
(
    cd "${RAPIDJSON_DIR}"
    "${BIN}" merge --feature "${FEATURE}"
)
echo ""

# ---------------------------------------------------------------------------
# Step 5: Validate output
# ---------------------------------------------------------------------------
echo "=== Step 5: Validating output ==="

FAIL=0
check_file() {
    local path="$1"
    if [ -f "$path" ]; then
        echo "  [OK]  $path"
    else
        echo "  [FAIL] missing: $path" >&2
        FAIL=1
    fi
}
check_contains() {
    local path="$1"
    local pattern="$2"
    if grep -q "$pattern" "$path" 2>/dev/null; then
        echo "  [OK]  $path contains '${pattern}'"
    else
        echo "  [FAIL] $path does not contain '${pattern}'" >&2
        FAIL=1
    fi
}

OUT="${RAPIDJSON_DIR}/.cpp2rust/${FEATURE}"

for case_name in "${CASE_NAMES[@]}"; do
    middleware_file="${case_name}.cpp.cpp2rust"
    check_file  "${OUT}/cpp/${middleware_file}"
    check_file  "${OUT}/cpp/${middleware_file}.opts"
    check_file  "${OUT}/rust/src.1/mod_${case_name}/include/mod.rs"
    check_file  "${OUT}/rust/src.1/mod_${case_name}/meta.json"
    check_contains "${OUT}/meta/selected_files.json" "${middleware_file}"
    check_contains "${OUT}/rust/src.1/mod_${case_name}/include/mod.rs" "#include \"${middleware_file}\""
    check_contains "${OUT}/rust/src.1/mod_${case_name}/meta.json" "\"group\": \"mod_${case_name}\""
    check_contains "${OUT}/rust/src.1/mod_${case_name}/meta.json" "${middleware_file}"
    check_contains "${OUT}/rust/src/merged_ffi.rs" "#include \"${middleware_file}\""
done

check_file  "${OUT}/meta/build_cmd.txt"
check_file  "${OUT}/meta/selected_files.json"
check_file  "${OUT}/meta/headers.json"
check_file  "${OUT}/meta/init-interface-report.md"
check_file  "${OUT}/rust/Cargo.toml"
check_file  "${OUT}/rust/build.rs"
check_file  "${OUT}/rust/src/lib.rs"
check_file  "${OUT}/rust/src/merged_ffi.rs"
check_file  "${OUT}/meta/merge-report.md"

check_contains "${OUT}/rust/src/merged_ffi.rs" "import_lib!"
check_contains "${OUT}/rust/src/merged_ffi.rs" 'link_name = "rapidjson"'

echo ""
echo "=== Generated .cpp2rust directory tree ==="
find "${RAPIDJSON_DIR}/.cpp2rust" -type f | sort

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "VALIDATION FAILED – see errors above." >&2
    exit 1
fi

echo ""
echo "✓ All validation checks passed."
