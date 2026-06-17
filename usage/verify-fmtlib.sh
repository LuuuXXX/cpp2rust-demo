#!/usr/bin/env bash
# =============================================================================
# verify-fmtlib.sh —— {fmt} 直出（无 shim）工作流本地验证
#
# {fmt} 是现代 C++ 格式化库；本脚本编译其实现单元 src/format.cc 与 src/os.cc
# 触发拦截，对命名空间类与自由函数直出 safe FFI。
#
# 可配置环境变量：
#   FEATURE        feature 名称（默认 fmtlib）
#   SKIP_INSTALL   置 1 跳过 cargo install
#   FMTLIB_DIR     覆盖 fmtlib 源码目录（默认 references/fmtlib）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${FMTLIB_DIR:-${REPO_DIR}/references/fmtlib}"
if [ ! -e "${LIB_DIR}/src/format.cc" ]; then
    c2r_ensure_submodule "references/fmtlib" || fail "fmtlib 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-fmtlib}"
LIB_DISPLAY="fmtlib（直出工作流）"
CXX_STD="c++17"
C2R_SRC_FILES=("${LIB_DIR}/src/format.cc" "${LIB_DIR}/src/os.cc")
C2R_INCLUDES=("${LIB_DIR}/include" "${LIB_DIR}/src")
C2R_DRIVER_CONTENT=""

c2r_run_direct
c2r_finish
