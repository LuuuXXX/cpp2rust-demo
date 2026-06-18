#!/usr/bin/env bash
# =============================================================================
# verify-tinyxml2-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 tinyxml2 C++ 库的 Rust safe FFI 生成能力
#
# 项目特征：
#   - 经典的单头文件 + 单 .cpp XML 解析库（tinyxml2.cpp ~4K 行 + tinyxml2.h）
#   - 带完整 OOP 类层级（XMLDocument / XMLElement / XMLNode 等）
#   - 典型的「单文件项目」代表，验证工具对含继承关系的 C++ 类的处理能力
#
# 流程（与其他 verify-<lib>-ffi.sh 一致）：
#   § 0-1. 环境检查 & 安装 cpp2rust-demo
#   § 2.   初始化 references/tinyxml2 子模块
#   § 3.   编译目标文件（供 nm 符号验证）
#   § 4.   cpp2rust-demo init —— 编译拦截 & FFI 脚手架生成
#   § 5.   cpp2rust-demo merge —— 整理输出目录
#   § 5b-c. cargo check + cargo test
#   § 6.   FFI 审计（import_lib!、link_name、nm 交叉比对、降级标记）
#   § 7.   报告
#
# 工作目录策略：
#   init/merge 从 PROJECT_ROOT（references/tinyxml2/）运行，使捕获的相对路径
#   只剩 basename（tinyxml2.cpp.cpp2rust），避免 link_name 含路径分隔符。
#   输出 .cpp2rust/ 位于 references/tinyxml2/ 内，已被根 .gitignore 忽略。
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

# ─── 库特有参数 ──────────────────────────────────────────────────────────────
LIB_NAME="tinyxml2"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
# SOURCES / INCLUDE_DIRS 使用「相对 PROJECT_ROOT」的路径
# （脚本会 cd 到 PROJECT_ROOT 后调用 cpp2rust-demo）
SOURCES=( "tinyxml2.cpp" )
INCLUDE_DIRS=( "." )
CXX_STD="${CXX_STD:-c++17}"

# =============================================================================
# § 0-1. 环境检查 & 安装
# =============================================================================
cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

# =============================================================================
# § 2. 子模块
# =============================================================================
cpp2rust_step "§ 2. 初始化子模块 ${SUBMODULE_REL}"
cpp2rust_init_submodule "${SUBMODULE_REL}"
for s in "${SOURCES[@]}"; do
    [ -f "${PROJECT_ROOT}/${s}" ] || cpp2rust_fail "源文件不存在：${PROJECT_ROOT}/${s}"
done
cpp2rust_info "源文件（相对 PROJECT_ROOT）：${SOURCES[*]}"

# 设定 init/merge 的工作目录为 PROJECT_ROOT
cpp2rust_set_workdir "${PROJECT_ROOT}"

# =============================================================================
# § 3. 编译目标文件（nm 验证用）
# =============================================================================
cpp2rust_step "§ 3. 编译目标文件"
OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" 2>/dev/null || true' EXIT
cpp2rust_info "目标文件输出目录：${OBJ_DIR}"
# 编译时用绝对路径，避免歧义
ABS_SOURCES=( "${SOURCES[@]/#/${PROJECT_ROOT}/}" )
ABS_INCLUDES=( "${INCLUDE_DIRS[@]/#/${PROJECT_ROOT}/}" )
cpp2rust_compile_units "${OBJ_DIR}" "${ABS_INCLUDES[@]}" -- "${ABS_SOURCES[@]}"

# =============================================================================
# § 4. cpp2rust-demo init
# =============================================================================
cpp2rust_step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"
BUILD_SCRIPT=$(mktemp)
# build script 内部 cd 到 PROJECT_ROOT 执行，所以用相对路径
cpp2rust_make_build_script "${BUILD_SCRIPT}" "${INCLUDE_DIRS[@]}" -- "${SOURCES[@]}"
cpp2rust_run_init "${FEATURE}" "${BUILD_SCRIPT}"
rm -f "${BUILD_SCRIPT}"

CPP2RUST_OUTPUT="$(cpp2rust_output_dir "${FEATURE}")"
RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
RUST_SRC="${RUST_PROJECT}/src"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
cpp2rust_info "输出目录：${CPP2RUST_OUTPUT}"
cpp2rust_info "捕获预处理文件数：${CAPTURED}"

# =============================================================================
# § 5. merge
# =============================================================================
cpp2rust_step "§ 5. cpp2rust-demo merge"
cpp2rust_run_merge "${FEATURE}"

# =============================================================================
# § 5b-c. cargo check / test
# =============================================================================
cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"

cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

# =============================================================================
# § 6. FFI 审计
# =============================================================================
cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

# =============================================================================
# § 7. 报告
# =============================================================================
cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
