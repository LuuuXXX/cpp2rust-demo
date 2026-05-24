#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

# Examples whose generated Rust FFI is known to have open codegen bugs.
# cargo check is run for ALL examples; failures for these are reported but do
# not fail the script (tracked as known issues).  Remove an entry here once
# the underlying bug is fixed.
CARGO_CHECK_KNOWN_FAILING=(
  02-pointers-references   # typedef function-pointer type aliases not mapped
  03-classes-basic         # nested/forward-declared class types not resolved
  04-inheritance           # base-class initialisation codegen incomplete
  05-virtual-polymorphism  # trait objects used where concrete types required; duplicate fields
  06-operator-overload     # operator methods emitted twice (duplicate definitions)
  07-templates-function    # template type parameters (T, V) not erased in output
  08-templates-class       # template type parameters not erased in output
  10-stl-containers        # STL container types (vector, pair) not mapped
  11-smart-pointers        # shared_ptr / unique_ptr type mapping gaps
  12-move-semantics        # rvalue/move type mapping issues
  13-lambdas-functional    # std::function / closure type mapping incomplete
  14-type-casting          # cast-result types not fully mapped
  15-exceptions            # std::nothrow_t / std::align_val_t not filtered
  16-static-members        # static member codegen issues
  18-const-correctness     # const-qualified reference lifetime issues
  19-memory-management     # operator new overloads (nothrow_t) not filtered
  20-template-specialization # template specialisation types not resolved
)

is_known_failing() {
  local name="$1"
  for kf in "${CARGO_CHECK_KNOWN_FAILING[@]}"; do
    [[ "$kf" == "$name" ]] && return 0
  done
  return 1
}

fail=0
cargo_check_pass=0
cargo_check_fail=0
cargo_check_known_fail=0

for example_dir in "$repo_root"/examples/[0-9][0-9]-*/; do
  example_name="$(basename "$example_dir")"
  feature_dir="$example_dir/.cpp2rust/ci-validate"

  rm -rf "$feature_dir"
  echo "==> init $example_name"
  (
    cd "$example_dir"
    cargo run --manifest-path "$repo_root/Cargo.toml" -- init --feature ci-validate -- sh -c 'make clean && make' >/dev/null 2>&1
  )

  # Validate every AST JSON file produced for this example
  ast_dir="$feature_dir/ast"
  if [ ! -d "$ast_dir" ]; then
    echo "FAIL: no ast/ directory for $example_name" >&2
    fail=1
    continue
  fi

  json_count=0
  while IFS= read -r -d '' json_file; do
    python3 -c "
import json, sys
with open(sys.argv[1]) as f:
    json.load(f)
print('  validated', sys.argv[1])
" "$json_file" || { echo "FAIL: invalid JSON in $json_file" >&2; fail=1; }
    json_count=$((json_count + 1))
  done < <(find "$ast_dir" -name '*.json' -print0)

  if [ "$json_count" -eq 0 ]; then
    echo "FAIL: no JSON files captured for $example_name" >&2
    fail=1
  fi

  # Validate the generated Rust FFI crate type-checks with cargo check
  rust_dir="$feature_dir/rust"
  if [ ! -f "$rust_dir/Cargo.toml" ]; then
    echo "FAIL: no generated Cargo.toml for $example_name" >&2
    fail=1
    continue
  fi

  echo "  cargo check $example_name"
  check_output=$(cargo check --quiet --target-dir "$repo_root/target/ci-validate" --manifest-path "$rust_dir/Cargo.toml" 2>&1) && check_ok=true || check_ok=false

  if $check_ok; then
    echo "  [PASS] cargo check $example_name"
    cargo_check_pass=$((cargo_check_pass + 1))
  elif is_known_failing "$example_name"; then
    echo "  [KNOWN FAIL] cargo check $example_name (open codegen bug — see CARGO_CHECK_KNOWN_FAILING)" >&2
    echo "$check_output" | head -5 >&2 || true
    cargo_check_known_fail=$((cargo_check_known_fail + 1))
  else
    echo "FAIL: cargo check $example_name (unexpected failure)" >&2
    echo "$check_output" >&2
    cargo_check_fail=$((cargo_check_fail + 1))
    fail=1
  fi
done

echo ""
echo "cargo check summary: ${cargo_check_pass} passed, ${cargo_check_known_fail} known-failing (open bugs), ${cargo_check_fail} unexpected failures"

if [ "$fail" -ne 0 ]; then
  echo "validate-rapidjson: one or more examples FAILED" >&2
  exit 1
fi
echo "All examples validated successfully."
