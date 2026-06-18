#!/usr/bin/env bash
# =============================================================================
# verify-sqlite3-ffi.sh — 用途：验证 cpp2rust-demo 对 sqlite3 系统 C 库的 FFI 生成能力
# 项目特征：系统 C 库（/usr/include/sqlite3.h），SQLITE_API 宏暴露 extern "C" API
# 用 driver cpp 包装 #include "sqlite3.h"。流程与 verify-tinyxml2-ffi.sh 同构。
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y libsqlite3-dev
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"

# shellcheck disable=SC2034
CPP2RUST_REPO_DIR="$(cpp2rust_find_repo_root)"

LIB_NAME="sqlite3"
FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
SQLITE3_HEADER="${SQLITE3_HEADER:-/usr/include/sqlite3.h}"
HEADER_FILE="sqlite3.h"
CXX_STD="${CXX_STD:-c++17}"
DRIVER_NAME="_cpp2rust_sqlite3_driver.cpp"

cpp2rust_step "§ 0-1. 环境检查 & 安装 cpp2rust-demo"
cpp2rust_require_cmds git g++ cargo nm
cpp2rust_check_libclang
cpp2rust_install_tool "${SKIP_INSTALL:-0}"
cpp2rust-demo --version 2>/dev/null || true

cpp2rust_step "§ 2. 检查系统头文件 ${SQLITE3_HEADER}"
cpp2rust_require_system_header_or_skip "${SQLITE3_HEADER}" \
    "Ubuntu/Debian：sudo apt-get install -y libsqlite3-dev"

# 把系统 sqlite3.h 复制到仓库根，driver cpp 用 #include "sqlite3.h" 引用
cpp2rust_step "§ 3. 复制系统头 + 生成 driver cpp & 编译"
LOCAL_HEADER="${CPP2RUST_REPO_DIR}/sqlite3.h"
DRIVER_CPP="${CPP2RUST_REPO_DIR}/${DRIVER_NAME}"
OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${BUILD_SCRIPT:-}" "${LOCAL_HEADER}" "${DRIVER_CPP}" 2>/dev/null || true' EXIT
cp -f "${SQLITE3_HEADER}" "${LOCAL_HEADER}"
# include 路径就是仓库根（"."），driver 直接 #include "sqlite3.h"
cpp2rust_make_driver_cpp "${DRIVER_CPP}" "." -- "${HEADER_FILE}"
cpp2rust_compile_units "${OBJ_DIR}" "${CPP2RUST_REPO_DIR}" -- "${DRIVER_CPP}"

cpp2rust_step "§ 4. cpp2rust-demo init"
BUILD_SCRIPT=$(mktemp)
cpp2rust_make_build_script "${BUILD_SCRIPT}" "." -- "${DRIVER_NAME}"
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

# sqlite3 函数实现在系统 libsqlite3（Linux: libsqlite3.so / Windows: sqlite3.dll）。
# 问题：sqlite3.h 含平台条件函数（sqlite3_win32_* / sqlite3_snapshot_cmp 等），
# 在当前平台的 libsqlite3 中无符号，链接报 undefined。
# 解决：扫描 libsqlite3 实际导出的符号集合，注释掉生成 .rs 文件中找不到
# 实现的 extern-C 函数绑定；同时在 build.rs 追加 -lsqlite3 链接指令。
if cpp2rust_is_windows; then
    # Windows: libsqlite3.dll 可能在 system32 / System32 或 MinGW bin 下
    LIBSQLITE3_PATH="${LIBSQLITE3_PATH:-}"
    if [ -z "${LIBSQLITE3_PATH}" ]; then
        for p in /c/Windows/System32/sqlite3.dll \
                 /c/msys64/mingw64/bin/sqlite3.dll \
                 "$(pwd)/sqlite3.dll"; do
            if [ -f "$p" ]; then LIBSQLITE3_PATH="$p"; break; fi
        done
    fi
else
    LIBSQLITE3_PATH="${LIBSQLITE3_PATH:-/usr/lib/x86_64-linux-gnu/libsqlite3.so}"
    if [ ! -f "${LIBSQLITE3_PATH}" ]; then
        LIBSQLITE3_PATH="$(ldconfig -p 2>/dev/null | grep 'libsqlite3.so ' | head -1 | awk '{print $NF}')"
    fi
fi
cpp2rust_info "libsqlite3 路径：${LIBSQLITE3_PATH:-（未找到，跳过符号过滤）}"

BUILD_RS="${RUST_PROJECT}/build.rs"
if [ -f "${BUILD_RS}" ] && ! grep -q "rustc-link-lib=sqlite3" "${BUILD_RS}"; then
    awk '
        { lines[NR] = $0 }
        END {
            last_brace = -1
            for (i = NR; i >= 1; i--) {
                if (lines[i] ~ /^[[:space:]]*}[[:space:]]*$/) { last_brace = i; break }
            }
            for (i = 1; i <= NR; i++) {
                if (i == last_brace) {
                    print "    println!(\"cargo::rustc-link-lib=sqlite3\");"
                }
                print lines[i]
            }
        }
    ' "${BUILD_RS}" > "${BUILD_RS}.tmp" && mv "${BUILD_RS}.tmp" "${BUILD_RS}"
    cpp2rust_info "build.rs 已追加 -lsqlite3"
fi

# 过滤生成 .rs 中找不到实现的符号（注释掉对应绑定）。
# 用单次 awk 扫描整个文件，遇到 #[cpp(func = "...fn...")] 行就检查 fn 是否在
# libsqlite3 符号集中；不在则同时注释本行和紧随的 pub fn 行。
if [ -n "${LIBSQLITE3_PATH}" ] && [ -f "${LIBSQLITE3_PATH}" ]; then
    cpp2rust_info "扫描 libsqlite3.so 并过滤生成代码中无实现的符号..."
    # 取得 libsqlite3.so 中所有 T 段定义的 sqlite3_ 符号
    SYMBOLS_FILE=$(mktemp)
    nm -D --defined-only "${LIBSQLITE3_PATH}" 2>/dev/null \
        | awk '$2 == "T" && $3 ~ /^sqlite3_/ {print $3}' | sort -u > "$SYMBOLS_FILE"
    cpp2rust_info "libsqlite3.so 中 sqlite3_* T 段符号数：$(wc -l < "$SYMBOLS_FILE")"

    FILTERED_COUNT=0
    while IFS= read -r rs_file; do
        [ -f "$rs_file" ] || continue
        # 单次 awk 扫描：先收集符号集（GNU awk 支持的 FILENAME 间共享）
        # 标记需要跳过的行（func 行 + 紧随的 pub fn 行）
        awk -v syms="$SYMBOLS_FILE" '
            BEGIN {
                while ((getline line < syms) > 0) sym_set[line] = 1
                close(syms)
                skip_next = 0
            }
            {
                if (skip_next) {
                    # 上一行是过滤的 func，本行是 pub fn → 注释掉
                    sub(/^[[:space:]]*/, "")
                    print "// " $0
                    skip_next = 0
                    next
                }
                if ($0 ~ /#\[cpp\(func = "[^"]*"\)\]/) {
                    # 提取函数名（最后一个紧跟 ( 的 identifier）
                    sig = $0
                    sub(/.*func = "/, "", sig)
                    sub(/".*/, "", sig)
                    n = split(sig, parts, /[()]/)
                    last = parts[1]
                    gsub(/^[[:space:]]+|[[:space:]]+$/, "", last)
                    m = split(last, toks, /[[:space:]*]+/)
                    fn = toks[m]
                    if (fn != "" && !(fn in sym_set)) {
                        filtered++
                        print "    // cpp2rust-filtered: " fn " 不在 libsqlite3 中（平台条件函数）"
                        sub(/^[[:space:]]*/, "")
                        print "// " $0
                        skip_next = 1
                        next
                    }
                }
                print
            }
            END { if (filtered > 0) print "    // 已过滤 " filtered " 个无实现绑定" > "/dev/stderr" }
        ' "$rs_file" > "$rs_file.tmp" && mv "$rs_file.tmp" "$rs_file"
        local_count=$(awk '/cpp2rust-filtered/{c++} END{print c+0}' "$rs_file")
        FILTERED_COUNT=$((FILTERED_COUNT + local_count))
    done < <(find "${RUST_SRC}" -name "*.rs" -not -name "lib.rs" -not -name "mod.rs" 2>/dev/null)
    rm -f "$SYMBOLS_FILE"
    cpp2rust_info "已过滤 ${FILTERED_COUNT} 个无实现的 sqlite3 函数"
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
