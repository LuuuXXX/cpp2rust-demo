#!/usr/bin/env bash
set -euo pipefail

TINYXML2_DIR="${TINYXML2_DIR:-references/tinyxml2}"
FEATURE="${FEATURE:-tinyxml2_ffi}"
SKIP_INSTALL="${SKIP_INSTALL:-0}"
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

echo "=== cpp2rust-demo verify-tinyxml2-ffi ==="
echo "TINYXML2_DIR : $TINYXML2_DIR"
echo "FEATURE      : $FEATURE"
echo "PROJECT_ROOT : $PROJECT_ROOT"

# § 0. 环境检查
for cmd in git g++ cargo nm; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "❌ 未找到命令：$cmd"
        exit 1
    fi
done
echo "✅ 环境检查通过"

# § 1. 安装 cpp2rust-demo
if [ "$SKIP_INSTALL" = "1" ]; then
    echo "⏭  跳过安装（SKIP_INSTALL=1）"
else
    echo "§ 1. 安装 cpp2rust-demo..."
    cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked
fi

# § 2. 确认子模块
if [ ! -d "${PROJECT_ROOT}/${TINYXML2_DIR}/tinyxml2.cpp" ]; then
    echo "§ 2. 初始化 tinyxml2 子模块..."
    git -C "$PROJECT_ROOT" submodule update --init "$TINYXML2_DIR"
fi
echo "✅ tinyxml2 源码就绪"

# § 3. init
echo "§ 3. cpp2rust-demo init..."
cd "${PROJECT_ROOT}/${TINYXML2_DIR}"
cpp2rust-demo init --feature "$FEATURE" -- g++ -shared -fPIC tinyxml2.cpp -o libtinyxml2.so

# § 4. merge
echo "§ 4. cpp2rust-demo merge..."
cpp2rust-demo merge --feature "$FEATURE"

# § 5. cargo check
echo "§ 5. cargo check..."
RUST_DIR=".cpp2rust/${FEATURE}/rust"
if [ -d "$RUST_DIR" ]; then
    cd "$RUST_DIR"
    cargo check --quiet
    echo "✅ cargo check 通过"
else
    echo "❌ 未找到生成目录：$RUST_DIR"
    exit 1
fi

echo ""
echo "=== 验证完成 ==="
echo "生成目录：${PROJECT_ROOT}/${TINYXML2_DIR}/$RUST_DIR"
