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
#   2. Clones Tencent/rapidjson into /tmp/rapidjson (re-uses existing clone)
#      and configures it with cmake (examples enabled, tests disabled).
#   3. Runs `cpp2rust-demo init` using `cmake --build` as the build command so
#      that the LD_PRELOAD hook captures every real compilation (one per example
#      .cpp source file).
#   4. Runs `cpp2rust-demo merge`.
#   5. Validates expected output files exist and contain expected content.
#      Uses dynamic discovery of captured .cpp2rust files (1:1 flat .rs layout).
#   6. Runs `cargo check` inside the generated Rust project to verify the
#      scaffold compiles (header-only library, --no-link mode).
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
# Step 2: Clone Tencent/rapidjson and configure cmake build
# ---------------------------------------------------------------------------
echo "=== Step 2: Preparing Tencent/rapidjson ==="
if [ -d "${RAPIDJSON_DIR}/.git" ]; then
    echo "Re-using existing clone at ${RAPIDJSON_DIR}"
else
    echo "Cloning Tencent/rapidjson into ${RAPIDJSON_DIR} ..."
    git clone --depth=1 https://github.com/Tencent/rapidjson.git "${RAPIDJSON_DIR}"
fi

echo "Configuring cmake build (examples ON, tests OFF) ..."
cmake -S "${RAPIDJSON_DIR}" -B "${RAPIDJSON_DIR}/build" \
    -DRAPIDJSON_BUILD_EXAMPLES=ON \
    -DRAPIDJSON_BUILD_TESTS=OFF \
    -DCMAKE_BUILD_TYPE=Debug
echo ""

# ---------------------------------------------------------------------------
# Step 3: Run cpp2rust-demo init via cmake --build
# ---------------------------------------------------------------------------
echo "=== Step 3: Running cpp2rust-demo init ==="
(
    cd "${RAPIDJSON_DIR}"
    # Remove previous output so this script is idempotent.
    rm -rf .cpp2rust
    # Use --no-link so cargo check works without a rapidjson.a static library
    # (rapidjson is header-only; all FFI content compiles from headers alone).
    "${BIN}" init \
        --feature "${FEATURE}" \
        --link rapidjson \
        --no-link \
        -- cmake --build "${RAPIDJSON_DIR}/build" --parallel 2
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

# --- Global artefacts ---
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

# --- Per-file artefacts (dynamically discovered from captured .cpp2rust files) ---
# Each .cpp compiled by cmake produces one .cpp2rust capture -> one flat .rs file.
captured_count=0
for mw_file in "${OUT}/cpp/"*.cpp2rust; do
    # Skip .opts companion files.
    [[ "${mw_file}" == *.opts ]] && continue
    [ -f "${mw_file}" ] || continue

    # Derive the stem: strip directory and .cpp.cpp2rust (or .cpp2rust) suffix.
    mw_basename="$(basename "${mw_file}")"
    # Remove .cpp2rust capture suffix to get original name, then strip extension.
    original_file="${mw_basename%.cpp2rust}"
    stem="${original_file%.*}"          # strip final extension (.cpp, .cc, ...)

    captured_count=$((captured_count + 1))

    check_file     "${OUT}/cpp/${mw_basename}"
    check_file     "${OUT}/cpp/${mw_basename}.opts"
    check_file     "${OUT}/rust/src.1/${stem}.rs"
    check_file     "${OUT}/rust/src.1/${stem}.meta.json"
    check_contains "${OUT}/meta/selected_files.json" "${mw_basename}"
    check_contains "${OUT}/rust/src.1/${stem}.rs" "#include \"${original_file}\""
    check_contains "${OUT}/rust/src.1/${stem}.rs" "import_lib!"
    check_contains "${OUT}/rust/src.1/${stem}.rs" 'link_name = "rapidjson"'
    check_contains "${OUT}/rust/src.1/${stem}.meta.json" "\"group\": \"${stem}\""
    check_contains "${OUT}/rust/src/merged_ffi.rs" "#include \"${original_file}\""
done

if [ "${captured_count}" -eq 0 ]; then
    echo "  [FAIL] No .cpp2rust files captured -- cmake build may have failed or hook not active" >&2
    FAIL=1
else
    echo ""
    echo "  Captured ${captured_count} translation unit(s)."
fi

echo ""
echo "=== Generated .cpp2rust directory tree ==="
find "${RAPIDJSON_DIR}/.cpp2rust" -type f | sort

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "VALIDATION FAILED -- see errors above." >&2
    exit 1
fi

echo ""
echo "All file validation checks passed."

# ---------------------------------------------------------------------------
# Step 6: cargo check on the generated Rust project
# ---------------------------------------------------------------------------
echo ""
echo "=== Step 6: cargo check on generated Rust project ==="
(
    cd "${OUT}/rust"
    # --no-link was used during init so build.rs does not emit
    # cargo::rustc-link-lib; the C++ adapter (rapidjson headers) compiles
    # cleanly without a static library.
    cargo check 2>&1
)
echo ""
echo "cargo check passed -- scaffold is valid Rust."
