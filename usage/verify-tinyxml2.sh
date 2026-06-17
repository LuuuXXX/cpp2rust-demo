#!/usr/bin/env bash
# =============================================================================
# verify-tinyxml2.sh —— tinyxml2 直出（无 shim）工作流本地验证
#
# tinyxml2 是单头文件 + 单 .cpp 的 XML 解析库，含典型 OOP 类层级
# （XMLDocument → XMLElement → XMLNode）。cpp2rust-demo 默认对其命名空间类与
# 自由函数「直出」safe FFI（import_class! / import_lib!），无需手写 extern-C shim。
#
# 流程：环境检查 → 安装工具 → 子模块就绪 → 编译 tinyxml2.cpp 触发拦截 → init →
#       merge → build.rs 校验 → cargo check →（若有）cargo test → 统计 → 汇报
#
# 可配置环境变量：
#   FEATURE        feature 名称（默认 tinyxml2）
#   SKIP_INSTALL   置 1 跳过 cargo install（已安装时复用）
#   TINYXML2_DIR   覆盖 tinyxml2 源码目录（默认 references/tinyxml2）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${TINYXML2_DIR:-${REPO_DIR}/references/tinyxml2}"
if [ ! -e "${LIB_DIR}/tinyxml2.cpp" ]; then
    c2r_ensure_submodule "references/tinyxml2" || fail "tinyxml2 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-tinyxml2}"
LIB_DISPLAY="tinyxml2（直出工作流）"
CXX_STD="c++17"
C2R_SRC_FILES=("${LIB_DIR}/tinyxml2.cpp")
C2R_INCLUDES=("${LIB_DIR}")
C2R_DRIVER_CONTENT=""

c2r_run_direct
c2r_finish
