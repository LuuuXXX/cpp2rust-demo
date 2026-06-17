#!/usr/bin/env bash
# =============================================================================
# verify-sqlite3.sh —— sqlite3（系统 C 接口）工作流本地验证
#
# sqlite3 是纯 C 接口库。本脚本写一个最小 wrapper .cpp（extern "C" { #include
# <sqlite3.h> }，与 tests/sqlite3_e2e_test.rs 同构）触发拦截，验证工具对 extern "C"
# 接口的处理（生成 import_lib! 而非 import_class!）。
#
# 依赖系统已安装 sqlite3 开发头：sudo apt-get install -y libsqlite3-dev
#
# 可配置环境变量：
#   FEATURE          feature 名称（默认 sqlite3）
#   SKIP_INSTALL     置 1 跳过 cargo install
#   SQLITE3_HEADER   覆盖 sqlite3.h 路径（默认 /usr/include/sqlite3.h）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

SQLITE3_HEADER="${SQLITE3_HEADER:-/usr/include/sqlite3.h}"
if [ ! -e "${SQLITE3_HEADER}" ]; then
    fail "系统 sqlite3.h 未安装：${SQLITE3_HEADER}（请运行 sudo apt-get install -y libsqlite3-dev）"
fi

FEATURE="${FEATURE:-sqlite3}"
LIB_DISPLAY="sqlite3（C 接口工作流）"
CXX_STD="c++17"
C2R_SRC_FILES=()
C2R_INCLUDES=()
C2R_DRIVER_CONTENT='// sqlite3 C++ wrapper — 验证 extern "C" 接口处理（生成 import_lib!）
extern "C" {
#include <sqlite3.h>
}'

c2r_run_direct
c2r_finish
