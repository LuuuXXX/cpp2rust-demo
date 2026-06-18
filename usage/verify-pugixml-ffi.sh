#!/usr/bin/env bash
# =============================================================================
# verify-pugixml-ffi.sh — 用途：验证 cpp2rust-demo 对 pugixml 的 FFI 生成能力
# 项目特征：单 src/pugixml.cpp（含完整实现 + pugixml.hpp），重成员函数类层级
# 流程总览与 verify-tinyxml2-ffi.sh 同构。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

LIB_NAME="pugixml"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
SOURCES=( "${SUBMODULE_REL}/src/pugixml.cpp" )
INCLUDE_DIRS=( "${SUBMODULE_REL}/src" )
CXX_STD="${CXX_STD:-c++17}"

cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

cpp2rust_step "§ 2. 初始化子模块 ${SUBMODULE_REL}"
cpp2rust_init_submodule "${SUBMODULE_REL}"
for s in "${SOURCES[@]}"; do
    [ -f "${CPP2RUST_REPO_DIR}/${s}" ] || cpp2rust_fail "源文件不存在：${CPP2RUST_REPO_DIR}/${s}"
done

cpp2rust_step "§ 3. 编译目标文件"
OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" 2>/dev/null || true' EXIT
ABS_SOURCES=( "${SOURCES[@]/#/${CPP2RUST_REPO_DIR}\//}" )
cpp2rust_compile_units "${OBJ_DIR}" "${INCLUDE_DIRS[@]/#/${CPP2RUST_REPO_DIR}\//}" -- "${ABS_SOURCES[@]}"

cpp2rust_step "§ 4. cpp2rust-demo init"
BUILD_SCRIPT=$(mktemp)
cpp2rust_make_build_script "${BUILD_SCRIPT}" "${INCLUDE_DIRS[@]}" -- "${SOURCES[@]}"
cpp2rust_run_init "${FEATURE}" "${BUILD_SCRIPT}"
rm -f "${BUILD_SCRIPT}"

cpp2rust_step "§ 5. cpp2rust-demo merge"
cpp2rust_run_merge "${FEATURE}"

CPP2RUST_OUTPUT="$(cpp2rust_output_dir "${FEATURE}")"
RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
RUST_SRC="${RUST_PROJECT}/src"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l || echo 0)
cpp2rust_info "输出目录：${CPP2RUST_OUTPUT}"
cpp2rust_info "捕获预处理文件数：${CAPTURED}"

# pugixml 的 as_wide / as_utf8 / as_utf16 等是字符串重载函数（char / wchar_t），
# hicc 无法消歧重载（"address of overloaded function"）。在 cargo check 前过滤。
cpp2rust_filter_bindings "${RUST_SRC}" "as_wide" "as_utf8" "as_utf16" "from_wide"

cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"
cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
