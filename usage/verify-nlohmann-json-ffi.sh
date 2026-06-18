#!/usr/bin/env bash
# =============================================================================
# verify-nlohmann-json-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 nlohmann/json C++ 库的 Rust safe FFI 生成能力
#
# 项目特征：
#   - header-only 库（include/nlohmann/json.hpp 单文件 ~23K 行）
#   - 重度使用模板（basic_json<...>）与模板特化
#   - 经典的「超大单头文件」代表，验证工具对巨型预处理输出的处理能力
#
# 与有 .cpp 实现的库不同：header-only 库没有现成的 .cpp 可被 LD_PRELOAD 拦截，
# 本脚本自动生成一个 driver cpp（仅 #include <nlohmann/json.hpp>）作为入口。
# 流程总览与 verify-tinyxml2-ffi.sh 同构。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

# ─── 库特有参数 ──────────────────────────────────────────────────────────────
LIB_NAME="nlohmann-json"
FEATURE="${FEATURE:-${LIB_NAME//-/_}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
INCLUDE_DIRS=( "include" )
HEADER_FILE="nlohmann/json.hpp"   # 相对 INCLUDE_DIRS 的头路径
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
DRIVER_NAME="_cpp2rust_nlohmann_json_driver.cpp"
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" "${PROJECT_ROOT}/${DRIVER_NAME}" 2>/dev/null || true' EXIT
DRIVER_CPP="${PROJECT_ROOT}/${DRIVER_NAME}"
cpp2rust_make_driver_cpp "${DRIVER_CPP}" "${INCLUDE_DIRS[@]}" -- "${HEADER_FILE}"
ABS_INCLUDES=( "${INCLUDE_DIRS[@]/#/${PROJECT_ROOT}/}" )
cpp2rust_compile_units "${OBJ_DIR}" "${ABS_INCLUDES[@]}" -- "${DRIVER_CPP}"

# § 4. init
cpp2rust_step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
BUILD_SCRIPT=$(mktemp)
# build script 内 cwd = WORKDIR = PROJECT_ROOT，driver cpp 用相对路径
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
