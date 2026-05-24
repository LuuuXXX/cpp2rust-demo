#!/bin/bash
# Script to generate AST JSON for all C++ examples

set -e

EXAMPLES_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Generating AST JSON files for all examples..."

for dir in "$EXAMPLES_DIR"/[0-9]*-*; do
    if [ -d "$dir" ] && [ -f "$dir/main.cpp" ]; then
        example_name=$(basename "$dir")
        echo "  Processing: $example_name"

        cd "$dir"
        if clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json 2>/dev/null; then
            echo "    Generated: $dir/ast.json"
        else
            echo "    Warning: AST generation had issues for $example_name"
        fi
    fi
done

echo "Done!"
