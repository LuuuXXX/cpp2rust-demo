#!/usr/bin/env bash
# =============================================================================
# verify-nlohmann-json-ffi.sh — 用途：验证 cpp2rust-demo 对 nlohmann/json 的 FFI 生成能力
# 项目特征：header-only（include/nlohmann/json.hpp ~23K 行），重度模板
# 用 driver cpp 触发预处理拦截。流程与 verify-tinyxml2-ffi.sh 同构。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

LIB_NAME="nlohmann-json"
FEATURE="${FEATURE:-${LIB_NAME//-/_}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
INCLUDE_DIRS=( "${SUBMODULE_REL}/include" )
HEADER_FILE="nlohmann/json.hpp"
CXX_STD="${CXX_STD:-c++17}"
DRIVER_NAME="_cpp2rust_nlohmann_json_driver.cpp"

cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

cpp2rust_step "§ 2. 初始化子模块 ${SUBMODULE_REL}"
cpp2rust_init_submodule "${SUBMODULE_REL}"
[ -f "${CPP2RUST_REPO_DIR}/${INCLUDE_DIRS[0]}/${HEADER_FILE}" ] \
    || cpp2rust_fail "头文件不存在：${CPP2RUST_REPO_DIR}/${INCLUDE_DIRS[0]}/${HEADER_FILE}"

# driver cpp 必须放在仓库根（hook 才会捕获，且不会被 realpath 解析到 stage 外）
cpp2rust_step "§ 3. 生成 driver cpp & 编译目标文件"
DRIVER_CPP="${CPP2RUST_REPO_DIR}/${DRIVER_NAME}"
OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" "${CPP2RUST_REPO_DIR}/${DRIVER_NAME}" 2>/dev/null || true' EXIT
cpp2rust_make_driver_cpp "${DRIVER_CPP}" "${INCLUDE_DIRS[@]}" -- "${HEADER_FILE}"
cpp2rust_compile_units "${OBJ_DIR}" "${INCLUDE_DIRS[@]/#/${CPP2RUST_REPO_DIR}\//}" -- "${DRIVER_CPP}"

cpp2rust_step "§ 4. cpp2rust-demo init"
BUILD_SCRIPT=$(mktemp)
# driver cpp 用绝对路径（cargo 在 <final>/.cpp2rust/<feature>/rust/ 跑，
# 相对路径会找不到 driver cpp；hook 也会用 realpath 解析，绝对路径无歧义）
cpp2rust_make_build_script "${BUILD_SCRIPT}" "${INCLUDE_DIRS[@]}" -- "${DRIVER_CPP}"
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

cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"
cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
