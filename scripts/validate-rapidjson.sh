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
#   3. Creates one entry-<header>.cpp per major rapidjson header and a
#      CMakeLists.txt that builds each as a separate executable.  The full
#      cmake build is run under cpp2rust-demo init so that every translation
#      unit is captured by the LD_PRELOAD hook.  This produces one .rs file
#      per header (full-build, multi-TU, 1:1 mapping), allowing each header's
#      FFI output to be inspected individually.
#      --no-link is used because rapidjson is a header-only library.
#   4. Runs cpp2rust-demo merge.
#   5. Validates the expected output files exist and contain expected content.
#   6. Runs a full `cargo check` on the generated Rust project to verify that
#      the scaffolding is type-correct (build.rs compiles the C++ adapter,
#      then the Rust code is type-checked against it).
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

# ---------------------------------------------------------------------------
# Step 3: Prepare the project -- one entry-<header>.cpp per major header + CMake
# ---------------------------------------------------------------------------
echo "=== Step 3: Preparing rapidjson project build (full-build, multi-TU) ==="
(
    cd "${RAPIDJSON_DIR}"
    # Remove previous output so this script is idempotent.
    rm -rf .cpp2rust build

    # One translation unit per major rapidjson header so that each header's
    # FFI output is visible as a separate .rs file after cpp2rust-demo init.
    cat > entry-document.cpp << 'CPP_EOF'
#include "rapidjson/document.h"
int main() { return 0; }
CPP_EOF

    cat > entry-reader.cpp << 'CPP_EOF'
#include "rapidjson/reader.h"
int main() { return 0; }
CPP_EOF

    cat > entry-writer.cpp << 'CPP_EOF'
#include "rapidjson/writer.h"
int main() { return 0; }
CPP_EOF

    cat > entry-prettywriter.cpp << 'CPP_EOF'
#include "rapidjson/prettywriter.h"
int main() { return 0; }
CPP_EOF

    cat > entry-pointer.cpp << 'CPP_EOF'
#include "rapidjson/pointer.h"
int main() { return 0; }
CPP_EOF

    cat > entry-schema.cpp << 'CPP_EOF'
#include "rapidjson/schema.h"
int main() { return 0; }
CPP_EOF

    # CMakeLists.txt with one executable per translation unit.
    # cmake --build will compile all six, and the LD_PRELOAD hook in
    # cpp2rust-demo init will capture each compilation separately,
    # producing one .rs file per header.
    cat > CMakeLists.txt << 'CMAKE_EOF'
cmake_minimum_required(VERSION 3.10)
project(cpp2rust_validate LANGUAGES CXX)
set(RAPIDJSON_HEADERS document reader writer prettywriter pointer schema)
foreach(h IN LISTS RAPIDJSON_HEADERS)
    add_executable(cpp2rust_${h} entry-${h}.cpp)
    target_include_directories(cpp2rust_${h} PRIVATE include)
    target_compile_features(cpp2rust_${h} PRIVATE cxx_std_11)
endforeach()
CMAKE_EOF
)
echo ""

# ---------------------------------------------------------------------------
# Step 4: Run cpp2rust-demo init
# ---------------------------------------------------------------------------
echo "=== Step 4: Running cpp2rust-demo init (full cmake build, all headers) ==="
(
    cd "${RAPIDJSON_DIR}"
    # Use --no-link because rapidjson is a header-only library.
    # cmake --build compiles all six entry-<header> executables.  The
    # LD_PRELOAD hook intercepts each compilation and produces one
    # entry-<header>.cpp.cpp2rust middleware file per TU, which cpp2rust-demo
    # then turns into one entry_<header>.rs file.
    "${BIN}" init \
        --feature "${FEATURE}" \
        --link rapidjson \
        --no-link \
        -- sh -c "cmake -S . -B build && cmake --build build -j2" < /dev/null
)
echo ""

# ---------------------------------------------------------------------------
# Step 5: Run cpp2rust-demo merge
# ---------------------------------------------------------------------------
echo "=== Step 5: Running cpp2rust-demo merge ==="
(
    cd "${RAPIDJSON_DIR}"
    "${BIN}" merge --feature "${FEATURE}"
)
echo ""

# ---------------------------------------------------------------------------
# Step 6: Validate output
# ---------------------------------------------------------------------------
echo "=== Step 6: Validating output ==="

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

# Each major header produces its own middleware + .rs file (multi-TU, 1:1 mapping).
# entry-<header>.cpp  ->  entry-<header>.cpp.cpp2rust  ->  entry_<header>.rs
HEADERS=(document reader writer prettywriter pointer schema)

for HEADER in "${HEADERS[@]}"; do
    MIDDLEWARE="entry-${HEADER}.cpp.cpp2rust"
    ORIGINAL="entry-${HEADER}.cpp"
    FLAT_RS="${OUT}/rust/src/entry_${HEADER}.rs"

    echo ""
    echo "--- ${HEADER} ---"
    check_file  "${OUT}/cpp/${MIDDLEWARE}"
    check_file  "${OUT}/cpp/${MIDDLEWARE}.opts"
    check_file  "${FLAT_RS}"
    check_file  "${OUT}/rust/src.1/entry_${HEADER}.meta.json"
    check_contains "${OUT}/meta/selected_files.json" "${MIDDLEWARE}"
    check_contains "${FLAT_RS}" "hicc::cpp!"
    check_contains "${FLAT_RS}" "#include \"${ORIGINAL}\""
    check_contains "${FLAT_RS}" "import_lib!"
    check_contains "${FLAT_RS}" 'link_name = "rapidjson"'
done

echo ""
check_file  "${OUT}/meta/build_cmd.txt"
check_file  "${OUT}/meta/selected_files.json"
check_file  "${OUT}/meta/headers.json"
check_file  "${OUT}/meta/init-interface-report.md"
check_file  "${OUT}/rust/Cargo.toml"
check_file  "${OUT}/rust/build.rs"
check_file  "${OUT}/rust/src/lib.rs"
# Note: no separate merged_ffi.rs – consolidated FFI content lives in lib.rs.
check_file  "${OUT}/meta/merge-report.md"

# lib.rs is the consolidated FFI entry point (replaces the old merged_ffi.rs).
# It must contain the hicc::import_lib! block and all required declarations.
check_contains "${OUT}/rust/src/lib.rs" "import_lib!"
check_contains "${OUT}/rust/src/lib.rs" 'link_name = "rapidjson"'
# lib.rs must reference at least one of the per-header middleware files.
check_contains "${OUT}/rust/src/lib.rs" "#include \"entry-document.cpp\""

# build.rs must reference lib.rs as the consolidated FFI entry point.
check_contains "${OUT}/rust/build.rs" "src/lib.rs"
# build.rs must be in no-link mode (header-only library).
check_contains "${OUT}/rust/build.rs" "Header-only"

# ---------------------------------------------------------------------------
# Step 7: Full cargo check of the generated Rust project
# ---------------------------------------------------------------------------
echo ""
echo "=== Step 7: Running cargo check on the generated Rust project ==="
(
    cd "${OUT}/rust"
    cargo check
)
echo "  [OK]  cargo check passed"

echo ""
echo "=== Generated .cpp2rust directory tree ==="
find "${RAPIDJSON_DIR}/.cpp2rust" -type f | sort

if [ "${FAIL}" -ne 0 ]; then
    echo ""
    echo "VALIDATION FAILED -- see errors above." >&2
    exit 1
fi

echo ""
echo "All validation checks passed."
