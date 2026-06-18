#!/usr/bin/env bash
# =============================================================================
# verify-tinyxml2-ffi.sh
# 用途：本地验证 cpp2rust-demo 对 tinyxml2 C++ 库的 Rust safe FFI 生成能力
#
# 项目特征：单头 + 单 cpp XML 解析库（tinyxml2.cpp ~4K 行 + tinyxml2.h），OOP 类层级
# 流程（与其他 verify-<lib>-ffi.sh 一致）：
#   § 0-1. 环境检查 & 安装 cpp2rust-demo
#   § 2.   初始化 references/tinyxml2 子模块
#   § 3.   编译目标文件（供 nm 符号验证）
#   § 4.   cpp2rust-demo init
#   § 5.   cpp2rust-demo merge
#   § 5b-c. cargo check + cargo test
#   § 6.   FFI 审计
#   § 7.   报告
#
# 工作目录：init/merge 从仓库根（CPP2RUST_REPO_DIR）运行，与 verify-rapidjson-ffi.sh
# 同策略。link_name 由 extractor 的 basename 规范化保证干净（不污染 hicc 命名空间拼接）。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

LIB_NAME="tinyxml2"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
# 源文件 + include 用相对仓库根的路径
SOURCES=( "${SUBMODULE_REL}/tinyxml2.cpp" )
INCLUDE_DIRS=( "${SUBMODULE_REL}" )
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

cpp2rust_step "§ 3. 编译目标文件（nm 验证用）"
OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" 2>/dev/null || true' EXIT
cpp2rust_info "目标文件目录：${OBJ_DIR}"
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

cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"

cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
