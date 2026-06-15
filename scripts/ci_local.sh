#!/usr/bin/env bash
# ci_local.sh — Local CI regression verification script
# Usage:
#   bash scripts/ci_local.sh           # Full verification
#   bash scripts/ci_local.sh --quick   # Quick (build + lint + unit only)

set -euo pipefail

QUICK=0
if [[ "${1:-}" == "--quick" ]]; then
    QUICK=1
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

step() {
    echo -e "${YELLOW}[STEP] $1${NC}"
}

ok() {
    echo -e "${GREEN}[PASS] $1${NC}"
}

fail() {
    echo -e "${RED}[FAIL] $1${NC}"
}

FAILED=0

run_step() {
    local name="$1"
    local cmd="$2"
    step "$name"
    if eval "$cmd"; then
        ok "$name"
    else
        fail "$name"
        FAILED=1
    fi
}

# Phase 1: Build + Lint
run_step "Build (cargo build --all-targets)" "cargo build --all-targets"
run_step "Check formatting (cargo fmt --check --all)" "cargo fmt --check --all"
run_step "Check clippy (cargo clippy --all-targets -- -D warnings)" "cargo clippy --all-targets -- -D warnings"

# Phase 2: Unit tests
run_step "Unit tests (lib)" "cargo test --lib"
run_step "Unit tests (bin)" "cargo test --bin cpp2rust-demo"

if [[ $QUICK -eq 1 ]]; then
    echo ""
    if [[ $FAILED -eq 0 ]]; then
        ok "Quick verification passed!"
    else
        fail "Quick verification FAILED!"
    fi
    exit $FAILED
fi

# Phase 3: L1 + L2
run_step "L1 golden tests" "cargo test --test l1_golden_tests --features full-test -- --include-ignored --test-threads=1"
run_step "L2 compile tests" "cargo test --test l2_compile_tests"

# Phase 4: E2E
run_step "tinyxml2 E2E" "git submodule update --init references/tinyxml2 && cargo test --test tinyxml2_e2e_test -- --test-threads=1"

echo ""
if [[ $FAILED -eq 0 ]]; then
    ok "All verification steps passed!"
else
    fail "Some verification steps FAILED!"
fi
exit $FAILED
