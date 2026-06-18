#!/usr/bin/env bash
# =============================================================================
# verify-fmtlib-ffi.sh — 用途：验证 cpp2rust-demo 对 fmtlib/fmt 的 FFI 生成能力
# 项目特征：多文件项目（src/format.cc / src/os.cc），重度模板与 extern template
# 流程总览与 verify-tinyxml2-ffi.sh 同构。
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

LIB_NAME="fmtlib"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SUBMODULE_REL="references/${LIB_NAME}"
PROJECT_ROOT="${CPP2RUST_REPO_DIR}/${SUBMODULE_REL}"
SOURCES=( "${SUBMODULE_REL}/src/format.cc" "${SUBMODULE_REL}/src/os.cc" )
INCLUDE_DIRS=( "${SUBMODULE_REL}/include" )
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

# fmt 的 convert_rwcount 是模板辅助函数，cpp2rust-demo 无法生成有效绑定；
# 在 cargo check 前过滤掉该函数。
cpp2rust_filter_bindings "${RUST_SRC}" "convert_rwcount"

# fmt 的 format-inl.h 含非内联函数定义，cc_build.file(format.cc) 与 hicc 生成的
# .rs.cpp 各编译一份，链接时 duplicate symbol。在 build.rs 追加 --allow-multiple-definition
# 让链接器接受第一个定义。
BUILD_RS="${RUST_PROJECT}/build.rs"
if [ -f "${BUILD_RS}" ] && ! grep -q "allow-multiple-definition" "${BUILD_RS}"; then
    awk '
        { lines[NR] = $0 }
        END {
            last_brace = -1
            for (i = NR; i >= 1; i--) {
                if (lines[i] ~ /^[[:space:]]*}[[:space:]]*$/) { last_brace = i; break }
            }
            for (i = 1; i <= NR; i++) {
                if (i == last_brace) {
                    print "    // 容忍 format-inl.h 非内联函数的双重定义（cc_build + hicc .rs.cpp）"
                    print "    println!(\"cargo::rustc-link-arg=-Wl,--allow-multiple-definition\");"
                }
                print lines[i]
            }
        }
    ' "${BUILD_RS}" > "${BUILD_RS}.tmp" && mv "${BUILD_RS}.tmp" "${BUILD_RS}"
    cpp2rust_info "build.rs 已追加 -Wl,--allow-multiple-definition"
fi

cpp2rust_step "§ 5b. cargo check"
cpp2rust_cargo_check "${RUST_PROJECT}"
cpp2rust_step "§ 5c. cargo test"
cpp2rust_cargo_test "${RUST_PROJECT}"

cpp2rust_step "§ 6. FFI 审计"
cpp2rust_ffi_audit "${RUST_SRC}" "${OBJ_DIR}"

cpp2rust_step "§ 7. 报告"
cpp2rust_final_report "${LIB_NAME}" "${FEATURE}" "${CPP2RUST_OUTPUT}" "${RUST_PROJECT}"
cpp2rust_exit_with_error_check
