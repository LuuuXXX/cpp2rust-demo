#!/usr/bin/env bash
# scripts/run_l3_local.sh
#
# L3 运行测试本地化脚本（Linux / macOS）
#
# 功能：
#   1. 遍历 examples/ 下所有 NNN_*/cpp/ 目录
#   2. 按平台编译 C++ 共享库（.so / .dylib）
#   3. 设置动态库搜索路径（LD_LIBRARY_PATH / DYLD_LIBRARY_PATH）
#   4. 运行 L3 集成测试
#
# 用法：
#   ./scripts/run_l3_local.sh                   # 运行所有 L3 测试
#   ./scripts/run_l3_local.sh --filter 001       # 只运行包含 "001" 的示例
#   ./scripts/run_l3_local.sh --filter hello     # 只运行包含 "hello" 的示例
#   ./scripts/run_l3_local.sh --compile-only     # 只编译共享库，不运行测试
#   ./scripts/run_l3_local.sh --clean            # 删除所有编译产物后退出
#
# 前提条件：
#   - Linux：g++ 或 clang++（默认使用 g++）
#   - macOS：clang++（Xcode Command Line Tools）
#   - Rust 工具链：rustup（用于 cargo test）

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
#  参数解析
# ─────────────────────────────────────────────────────────────────────────────
FILTER=""
COMPILE_ONLY=0
CLEAN=0
TEST_THREADS=4

while [[ $# -gt 0 ]]; do
    case "$1" in
        --filter|-f)
            FILTER="$2"
            shift 2
            ;;
        --compile-only|-c)
            COMPILE_ONLY=1
            shift
            ;;
        --clean)
            CLEAN=1
            shift
            ;;
        --threads|-j)
            TEST_THREADS="$2"
            shift 2
            ;;
        --help|-h)
            head -30 "$0" | grep '^#' | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "未知参数：$1" >&2
            exit 1
            ;;
    esac
done

# ─────────────────────────────────────────────────────────────────────────────
#  检测操作系统和编译器
# ─────────────────────────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
    Linux*)
        CXX="${CXX:-g++}"
        LIB_EXT="so"
        LIB_FLAG="-shared -fPIC"
        LIB_PATH_VAR="LD_LIBRARY_PATH"
        ;;
    Darwin*)
        CXX="${CXX:-clang++}"
        LIB_EXT="dylib"
        LIB_FLAG="-dynamiclib"
        LIB_PATH_VAR="DYLD_LIBRARY_PATH"
        ;;
    *)
        echo "不支持的操作系统：$OS（请使用 Linux 或 macOS）" >&2
        echo "Windows 用户请使用：scripts/run_l3_local.ps1" >&2
        exit 1
        ;;
esac

# 检查编译器是否可用
if ! command -v "$CXX" &>/dev/null; then
    echo "错误：未找到 C++ 编译器 '$CXX'" >&2
    echo "  Linux：sudo apt-get install g++" >&2
    echo "  macOS：xcode-select --install" >&2
    exit 1
fi

# ─────────────────────────────────────────────────────────────────────────────
#  定位仓库根目录
# ─────────────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"

if [[ ! -d "$EXAMPLES_DIR" ]]; then
    echo "错误：未找到 examples 目录：$EXAMPLES_DIR" >&2
    exit 1
fi

# ─────────────────────────────────────────────────────────────────────────────
#  --clean 模式：删除所有编译产物
# ─────────────────────────────────────────────────────────────────────────────
if [[ $CLEAN -eq 1 ]]; then
    echo "清理所有编译产物..."
    find "$EXAMPLES_DIR" -name "*.so" -o -name "*.dylib" | while read -r f; do
        echo "  删除：$f"
        rm -f "$f"
    done
    echo "清理完成。"
    exit 0
fi

# ─────────────────────────────────────────────────────────────────────────────
#  编译阶段：遍历所有 examples/NNN_*/cpp/ 目录
# ─────────────────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo " L3 运行测试本地化（$OS）"
echo " 编译器：$CXX"
if [[ -n "$FILTER" ]]; then
    echo " 过滤器：$FILTER"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

LIB_DIRS=()   # 所有编译输出目录（用于拼接 LIB_PATH_VAR）
FAILED=()     # 编译失败的目录
COMPILED=0

for example_dir in "$EXAMPLES_DIR"/*/; do
    example_name="$(basename "$example_dir")"

    # 应用过滤器
    if [[ -n "$FILTER" ]] && [[ "$example_name" != *"$FILTER"* ]]; then
        continue
    fi

    cpp_dir="$example_dir/cpp"
    if [[ ! -d "$cpp_dir" ]]; then
        continue
    fi

    # 从目录名提取库名（去掉 NNN_ 数字前缀）
    # 例如：001_hello_world -> hello_world
    lib_name="${example_name#*_}"
    lib_file="lib${lib_name}.${LIB_EXT}"
    lib_output="$cpp_dir/$lib_file"

    # 收集所有 .cpp 文件（排除含 main() 的文件可能更好，但通常 shim 没有 main）
    mapfile -t cpp_files < <(find "$cpp_dir" -maxdepth 1 -name "*.cpp" -not -name "*.test.cpp" | sort)

    if [[ ${#cpp_files[@]} -eq 0 ]]; then
        echo "  [跳过] $example_name：cpp 目录中无 .cpp 文件"
        continue
    fi

    # 编译共享库
    echo -n "  [编译] $example_name -> $lib_file ..."
    if "$CXX" -std=c++17 $LIB_FLAG \
            "${cpp_files[@]}" \
            -o "$lib_output" \
            -I"$cpp_dir" \
            2>/dev/null; then
        echo " ✓"
        LIB_DIRS+=("$cpp_dir")
        (( COMPILED++ )) || true
    else
        echo " ✗"
        FAILED+=("$example_name")
    fi
done

echo ""
echo "编译结果：成功 $COMPILED 个，失败 ${#FAILED[@]} 个"

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo "编译失败的示例（可能存在编译依赖问题，仍继续测试）："
    for f in "${FAILED[@]}"; do
        echo "  - $f"
    done
fi

if [[ $COMPILE_ONLY -eq 1 ]]; then
    echo ""
    echo "（--compile-only 模式，跳过测试）"
    exit 0
fi

# ─────────────────────────────────────────────────────────────────────────────
#  设置动态库路径并运行 L3 测试
# ─────────────────────────────────────────────────────────────────────────────
# 拼接所有 cpp 目录到库路径
NEW_LIB_PATHS=""
for d in "${LIB_DIRS[@]}"; do
    if [[ -z "$NEW_LIB_PATHS" ]]; then
        NEW_LIB_PATHS="$d"
    else
        NEW_LIB_PATHS="$NEW_LIB_PATHS:$d"
    fi
done

# 追加现有库路径（避免覆盖系统库）
EXISTING_LIB_PATH="${!LIB_PATH_VAR:-}"
if [[ -n "$EXISTING_LIB_PATH" ]]; then
    NEW_LIB_PATHS="$NEW_LIB_PATHS:$EXISTING_LIB_PATH"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo " 运行 L3 测试"
echo " $LIB_PATH_VAR（共 ${#LIB_DIRS[@]} 个目录）"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cd "$REPO_ROOT"
export "$LIB_PATH_VAR"="$NEW_LIB_PATHS"

# 构建 cargo test 命令
CARGO_ARGS=("test" "--test" "l3_run_tests" "--" "--include-ignored" "--test-threads=$TEST_THREADS")
if [[ -n "$FILTER" ]]; then
    CARGO_ARGS+=("$FILTER")
fi

cargo "${CARGO_ARGS[@]}"
