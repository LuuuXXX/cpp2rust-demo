#!/usr/bin/env bash
# =============================================================================
# verify-sqlite3-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 sqlite3 系统 C 库的 Rust safe FFI 生成能力
#
# 项目特征：
#   - 系统 C 库（/usr/include/sqlite3.h + libsqlite3.so），通过 extern "C" 暴露 API
#   - 不使用 C++ 特性，是验证 cpp2rust-demo 对纯 C 接口处理能力的代表
#   - 与其他子模块库不同：sqlite3 走系统包管理器，无 git submodule
#
# 流程总览与 verify-tinyxml2-ffi.sh 同构，但：
#   § 2 改为检查系统头文件存在性（缺失则 graceful skip）
#   § 3/4 用 driver cpp 包装 #include <sqlite3.h>
#   工作目录为临时目录（系统头文件无 PROJECT_ROOT 概念）
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y libsqlite3-dev
#   （其余依赖同 verify-tinyxml2-ffi.sh）
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

# sqlite3 使用临时目录作为 WORKDIR（见后），不需要 PROJECT_ROOT 概念
# shellcheck disable=SC2034
CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

# ─── 库特有参数 ──────────────────────────────────────────────────────────────
LIB_NAME="sqlite3"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SQLITE3_HEADER="${SQLITE3_HEADER:-/usr/include/sqlite3.h}"
HEADER_FILE="sqlite3.h"
CXX_STD="${CXX_STD:-c++17}"

# § 0-1. 环境 & 安装
cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

# § 2. 系统头文件检查（缺失则 graceful skip）
cpp2rust_step "§ 2. 检查系统头文件 ${SQLITE3_HEADER}"
cpp2rust_require_system_header_or_skip "${SQLITE3_HEADER}" \
    "Ubuntu/Debian：sudo apt-get install -y libsqlite3-dev"

# sqlite3.h 是系统头，无需 PROJECT_ROOT；用临时目录作为工作目录
WORKDIR=$(mktemp -d)
trap 'rm -rf "${WORKDIR}" "${OBJ_DIR:-}" "${BUILD_SCRIPT:-}" 2>/dev/null || true' EXIT
cpp2rust_set_workdir "${WORKDIR}"

# § 3. 生成 driver cpp + 编译目标文件
# driver cpp 必须放在 WORKDIR 内，否则 LD_PRELOAD hook 不会捕获
cpp2rust_step "§ 3. 生成 driver cpp & 编译目标文件"
OBJ_DIR=$(mktemp -d)
DRIVER_NAME="sqlite3_driver.cpp"
DRIVER_CPP="${WORKDIR}/${DRIVER_NAME}"
cpp2rust_make_driver_cpp "${DRIVER_CPP}" -- "${HEADER_FILE}"
cpp2rust_compile_units "${OBJ_DIR}" -- "${DRIVER_CPP}"

# § 4. init
cpp2rust_step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
BUILD_SCRIPT=$(mktemp)
cpp2rust_make_build_script "${BUILD_SCRIPT}" -- "${DRIVER_NAME}"
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
