#!/usr/bin/env bash
# =============================================================================
# verify-magic-enum-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 magic_enum C++ 库的 Rust safe FFI 生成能力
#
# 项目特征：
#   - header-only（include/magic_enum.hpp 单文件）
#   - 重度使用 constexpr 与模板元编程（编译期枚举反射）
#   - 经典的「constexpr / 模板元编程」代表，验证工具对编译期特性的处理能力
#
# 与 header-only 库一致：使用 driver cpp（#include <magic_enum.hpp>）触发预处理。
# 流程总览与 verify-nlohmann-json-ffi.sh 同构。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

# ─── 库特有参数 ──────────────────────────────────────────────────────────────
LIB_NAME="magic_enum"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
INCLUDE_DIRS=( "include" "example" )
HEADER_FILE="magic_enum/magic_enum.hpp"
CXX_STD="${CXX_STD:-c++17}"

# § 0-1. 环境 & 安装
cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

# § 2. 子模块
cpp2rust_step "§ 2. 初始化子模块 ${SUBMODULE_REL}"
cpp2rust_init_submodule "${SUBMODULE_REL}"
[ -f "${PROJECT_ROOT}/${INCLUDE_DIRS[0]}/${HEADER_FILE}" ] \
    || cpp2rust_fail "头文件不存在：${PROJECT_ROOT}/${INCLUDE_DIRS[0]}/${HEADER_FILE}"
cpp2rust_set_workdir "${PROJECT_ROOT}"

# § 3. 生成 driver cpp + 编译目标文件
# driver cpp 必须放在 WORKDIR（PROJECT_ROOT）内，否则 LD_PRELOAD hook 不会捕获
cpp2rust_step "§ 3. 生成 driver cpp & 编译目标文件"
OBJ_DIR=$(mktemp -d)
DRIVER_NAME="_cpp2rust_magic_enum_driver.cpp"
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" "${PROJECT_ROOT}/${DRIVER_NAME}" 2>/dev/null || true' EXIT
DRIVER_CPP="${PROJECT_ROOT}/${DRIVER_NAME}"
cpp2rust_make_driver_cpp "${DRIVER_CPP}" "${INCLUDE_DIRS[@]}" -- "${HEADER_FILE}"
ABS_INCLUDES=( "${INCLUDE_DIRS[@]/#/${PROJECT_ROOT}/}" )
cpp2rust_compile_units "${OBJ_DIR}" "${ABS_INCLUDES[@]}" -- "${DRIVER_CPP}"

# § 4. init
cpp2rust_step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
BUILD_SCRIPT=$(mktemp)
cpp2rust_make_build_script "${BUILD_SCRIPT}" "${INCLUDE_DIRS[@]}" -- "${DRIVER_NAME}"
cpp2rust_run_init "${FEATURE}" "${BUILD_SCRIPT}"
rm -f "${BUILD_SCRIPT}"

CPP2RUST_OUTPUT="$(cpp2rust_output_dir "${FEATURE}")"
RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
RUST_SRC="${RUST_PROJECT}/src"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
cpp2rust_info "输出目录：${CPP2RUST_OUTPUT}"
cpp2rust_info "捕获预处理文件数：${CAPTURED}"

# § 5. merge
cpp2rust_step "§ 5. cpp2rust-demo merge"
cpp2rust_run_merge "${FEATURE}"

# § 5b-c. cargo check / test
cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"

cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

# § 6. FFI 审计
cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

# § 7. 报告
cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
