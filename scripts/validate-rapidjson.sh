#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
example_dir="$repo_root/examples/01-basic-types"
feature_dir="$example_dir/.cpp2rust/ci-validate"

rm -rf "$feature_dir"
(
  cd "$example_dir"
  cargo run --manifest-path "$repo_root/Cargo.toml" -- init --feature ci-validate -- sh -c 'make clean && make' >/dev/null
)
REPO_ROOT="$repo_root" python - <<'PY'
import json, os, pathlib
repo = pathlib.Path(os.environ["REPO_ROOT"])
json_path = repo / "examples/01-basic-types/.cpp2rust/ci-validate/ast/main.cpp.json"
json.loads(json_path.read_text())
print("validated", json_path)
PY
