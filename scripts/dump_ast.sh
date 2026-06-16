#!/usr/bin/env bash
# dump_ast.sh <cpp_dir> [<file_stem>]
#
# 「AST → hicc」可追溯辅助工具：对某个示例的 C++ 源做宏展开与 clang JSON AST 转储，
# 并过滤出「仅用户自有头/实现文件」的精简 AST，便于人工核对工具抽取的 IR 是否正确。
#
# 输出（写入 <cpp_dir>/../ast/，该目录已被 .gitignore 忽略，绝不入库）：
#   - {stem}.i        宏展开后的源（保留 #include 但展开宏）
#   - ast.json        完整 clang JSON AST（仅语法解析，不实际编译）
#   - user-ast.json   过滤后只含用户自有声明的精简 AST
#
# 用法: bash scripts/dump_ast.sh examples/006_class_basic/cpp class_basic
set -euo pipefail

CPP_DIR="${1:?missing cpp dir}"
# 默认取目录下第一个非 main.cpp 的实现单元作为 stem；显式传入 $2 可覆盖。
if [ -z "${2:-}" ]; then
    CPP_CANDIDATE="$(ls "$CPP_DIR"/*.cpp 2>/dev/null | grep -v main.cpp | head -1 || true)"
    if [ -z "$CPP_CANDIDATE" ]; then
        echo "[dump_ast] error: 在 $CPP_DIR 下未找到非 main.cpp 的 .cpp 文件，请显式传入 stem" >&2
        exit 1
    fi
    STEM="$(basename "$CPP_CANDIDATE" | sed 's/.cpp$//')"
else
    STEM="$2"
fi

AST_DIR="$(cd "$CPP_DIR/.." && pwd)/ast"
mkdir -p "$AST_DIR"

CXX="${CXX:-clang++}"

# 1. 宏展开后的源（保留 #include 但展开宏）
"$CXX" -std=c++17 -E -dD -P -I"$CPP_DIR" \
    "$CPP_DIR/$STEM.cpp" \
    -o "$AST_DIR/$STEM.i"

# 2. JSON AST（仅语法解析，不实际编译；ast-dump 写到 stdout）
"$CXX" -std=c++17 -Xclang -ast-dump=json -fsyntax-only -I"$CPP_DIR" \
    "$CPP_DIR/$STEM.cpp" \
    > "$AST_DIR/ast.json"

# 3. 过滤后的「只含用户自己头文件节点」的精简 AST（用于人工提取关键信息）
HEADER_BASENAME="$(basename "$(ls "$CPP_DIR"/*.h | head -1)")"
CPP_BASENAME="$(basename "$(ls "$CPP_DIR"/*.cpp | grep -v main.cpp | head -1)")"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 "$SCRIPT_DIR/filter_ast.py" "$AST_DIR/ast.json" "$HEADER_BASENAME" "$CPP_BASENAME" "$AST_DIR/user-ast.json"

echo "[dump_ast] wrote: $AST_DIR/$STEM.i ($(wc -c < "$AST_DIR/$STEM.i") bytes)"
echo "[dump_ast] wrote: $AST_DIR/ast.json ($(wc -c < "$AST_DIR/ast.json") bytes — full)"
echo "[dump_ast] wrote: $AST_DIR/user-ast.json ($(wc -c < "$AST_DIR/user-ast.json") bytes — filtered to $HEADER_BASENAME)"
echo "[dump_ast] stem = $STEM"
