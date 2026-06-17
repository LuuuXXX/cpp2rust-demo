#!/usr/bin/env bash
# =============================================================================
# verify-pugixml.sh —— pugixml 直出（无 shim）工作流本地验证
#
# pugixml 是轻量 XML 库（src/pugixml.cpp + src/pugixml.hpp）。cpp2rust-demo 对其
# 命名空间类（pugi::xml_document 等）直出 safe FFI，无需手写 extern-C shim。
#
# 可配置环境变量：
#   FEATURE        feature 名称（默认 pugixml）
#   SKIP_INSTALL   置 1 跳过 cargo install
#   PUGIXML_DIR    覆盖 pugixml 源码目录（默认 references/pugixml）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${PUGIXML_DIR:-${REPO_DIR}/references/pugixml}"
if [ ! -e "${LIB_DIR}/src/pugixml.cpp" ]; then
    c2r_ensure_submodule "references/pugixml" || fail "pugixml 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-pugixml}"
LIB_DISPLAY="pugixml（直出工作流）"
CXX_STD="c++17"
C2R_SRC_FILES=("${LIB_DIR}/src/pugixml.cpp")
C2R_INCLUDES=("${LIB_DIR}/src")
C2R_DRIVER_CONTENT=""

c2r_run_direct
c2r_finish
