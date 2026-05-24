#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

fail=0

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
done

if [ "$fail" -ne 0 ]; then
  echo "validate-rapidjson: one or more examples FAILED" >&2
  exit 1
fi
echo "All examples validated successfully."
