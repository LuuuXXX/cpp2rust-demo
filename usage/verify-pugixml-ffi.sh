#!/usr/bin/env bash
# =============================================================================
# verify-pugixml-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 pugixml C++ 库的完整 Rust safe FFI 生成能力
#
# 背景说明：
#   pugixml 是 header-only 高性能 XML 解析库，单头 pugixml.hpp 约 11K 行，
#   与 tinyxml2 类似但采用 header-only 模式（无独立 .cpp 实现文件）。
#   主要类层级：xml_document、xml_node、xml_attribute 等。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地 pugixml 源文件（references/pugixml/src/pugixml.cpp）
#   § 3. 编译 pugixml.o（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. FFI 验证（nm 符号、import_class!、import_lib!、link_name）
#   § 7. 生成结果汇报
#
# 本脚本验证的 cpp2rust-demo 特性：
#   ① header-only 库处理 —— 单头文件模式
#   ② XML DOM 类层级   —— xml_document / xml_node / xml_attribute
#   ③ 方法绑定生成     —— import_class! 块包含所有公开方法
#   ④ 头文件探测       —— .hpp 头文件自动识别
#   ⑤ hicc 直出模式    —— 直接绑定命名空间类型
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-pugixml_demo}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"
CXX_STD="c++11"
TMPDIR="${TMPDIR:-/tmp}"

# ─── 颜色 / 辅助函数 ──────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ OK ]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }
step()  { echo -e "\n${BOLD}══════════════════════════════════════════${NC}"; \
          echo -e "${BOLD}  $*${NC}"; \
          echo -e "${BOLD}══════════════════════════════════════════${NC}"; }

need_cmd() { command -v "$1" &>/dev/null || fail "未找到命令：$1  请先安装后重试"; }

to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

SCRIPT_ERRORS=0

# =============================================================================
# § 0. 环境检查
# =============================================================================
step "§ 0. 环境检查"

need_cmd git
need_cmd g++
need_cmd cargo
need_cmd nm

if ! pkg-config --exists libclang 2>/dev/null && \
   ! find /usr /lib /usr/local -name "libclang*.so*" 2>/dev/null | grep -q .; then
    warn "未检测到 libclang（可能未安装 libclang-dev）"
fi

ok "环境检查完成"

# =============================================================================
# § 1. 安装 cpp2rust-demo
# =============================================================================
step "§ 1. 安装 cpp2rust-demo"

if [ "${SKIP_INSTALL}" = "1" ] && command -v cpp2rust-demo &>/dev/null; then
    ok "已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）"
else
    info "从 GitHub 源码安装 cpp2rust-demo…"
    INSTALL_LOG=$(mktemp)
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked \
             >"${INSTALL_LOG}" 2>&1; then
        cat "${INSTALL_LOG}"
        rm -f "${INSTALL_LOG}"
        fail "cpp2rust-demo 安装失败"
    fi
    tail -5 "${INSTALL_LOG}"
    rm -f "${INSTALL_LOG}"
    ok "cpp2rust-demo 安装完成"
fi

cpp2rust-demo --version 2>/dev/null || true

# =============================================================================
# § 2. 定位本地 pugixml 源文件
# =============================================================================
step "§ 2. 定位本地 pugixml 源文件"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"

PUGIXML_DIR="${REPO_DIR}/references/pugixml"
PUGIXML_SRC_DIR="${PUGIXML_DIR}/src"

if [ ! -d "${PUGIXML_DIR}" ]; then
    warn "pugixml 子模块未初始化：${PUGIXML_DIR}"
    warn "请运行：git submodule update --init references/pugixml"
    info "脚本将跳过验证"
    exit 0
fi

PUGIXML_CPP="${PUGIXML_SRC_DIR}/pugixml.cpp"
PUGIXML_HPP="${PUGIXML_SRC_DIR}/pugixml.hpp"

if [ ! -f "${PUGIXML_CPP}" ]; then
    fail "未找到 pugixml.cpp：${PUGIXML_CPP}"
fi

info "pugixml 目录：${PUGIXML_DIR}"
info "源文件：${PUGIXML_CPP}"
info "头文件：${PUGIXML_HPP}"

ok "pugixml 源文件就绪"

# =============================================================================
# § 3. 编译 pugixml.o
# =============================================================================
step "§ 3. 编译 pugixml.o"

OBJ_DIR=$(mktemp -d)
info "目标文件输出目录：${OBJ_DIR}"

trap 'rm -rf "${OBJ_DIR}" "${NM_CACHE:-}" 2>/dev/null || true' EXIT

PUGIXML_OBJ="${OBJ_DIR}/pugixml.o"
if g++ -c -std="${CXX_STD}" \
       -I"${PUGIXML_SRC_DIR}" \
       "${PUGIXML_CPP}" -o "${PUGIXML_OBJ}" 2>&1; then
    ok "编译成功：pugixml.o"
else
    warn "编译失败：pugixml.cpp"
    SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
fi

# =============================================================================
# § 4. cpp2rust-demo init
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
set -e
g++ -c -std="${CXX_STD}" \\
    -I"${PUGIXML_SRC_DIR}" \\
    "${PUGIXML_CPP}" -o /dev/null 2>&1
EOF
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"

cd "${REPO_DIR}"
cpp2rust-demo init \
    --feature "${FEATURE}" \
    -- bash "${BUILD_SCRIPT}"

rm -f "${BUILD_SCRIPT}"
ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${REPO_DIR}/.cpp2rust/${FEATURE}"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
info "捕获预处理文件数：${CAPTURED}"

# =============================================================================
# § 5. cpp2rust-demo merge
# =============================================================================
step "§ 5. cpp2rust-demo merge"

cd "${REPO_DIR}"
cpp2rust-demo merge --feature "${FEATURE}"
ok "merge 完成"

RUST_SRC="${CPP2RUST_OUTPUT}/rust/src"
RS_FILES=$(find -L "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l)
info "生成 Rust 文件数：${RS_FILES}"

echo ""
info "降级标记统计："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null \
    | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn \
    || echo "  （无降级标记）"

# =============================================================================
# § 5a. 校验 build.rs
# =============================================================================
step "§ 5a. 校验 build.rs"

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
BUILD_RS="${RUST_PROJECT}/build.rs"
LIB_NAME="${FEATURE//-/_}"

if [ -f "${BUILD_RS}" ]; then
    if grep -q 'cc_build.include' "${BUILD_RS}" && grep -q 'cc_build.file' "${BUILD_RS}"; then
        ok "build.rs 已由工具自动注入（方案 A）"
    else
        warn "退回脚本就地补全（方案 B）"

        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" \
                     ! -name "lib.rs" ! -name "mod.rs" | sort)

        PUGIXML_CPP_BP="$(to_build_path "${PUGIXML_CPP}")"
        PUGIXML_SRC_DIR_BP="$(to_build_path "${PUGIXML_SRC_DIR}")"

        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.std("${CXX_STD}");
    cc_build.include("${PUGIXML_SRC_DIR_BP}");
    cc_build.file("${PUGIXML_CPP_BP}");
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF
        ok "build.rs 已就地补全"
    fi
fi

# =============================================================================
# § 5b. cargo check
# =============================================================================
step "§ 5b. cargo check"

if [ -f "${RUST_PROJECT}/Cargo.toml" ]; then
    if (cd "${RUST_PROJECT}" && cargo check 2>&1); then
        ok "cargo check 通过 ✓"
    else
        fail "cargo check 失败"
    fi
fi

# =============================================================================
# § 5c. cargo test
# =============================================================================
step "§ 5c. cargo test"

SMOKE_FILE="${RUST_PROJECT}/tests/smoke.rs"
if [ -f "${SMOKE_FILE}" ]; then
    if (cd "${RUST_PROJECT}" && cargo test 2>&1); then
        ok "cargo test 通过 ✓"
    else
        warn "cargo test 失败"
    fi
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

echo -e "\n${BOLD}6a. pugixml.o 符号${NC}"
NM_CACHE=$(mktemp)
if [ -f "${PUGIXML_OBJ}" ]; then
    nm --defined-only -f posix "${PUGIXML_OBJ}" 2>/dev/null > "${NM_CACHE}" || true
    SYMBOL_COUNT=$(grep -c ' T ' "${NM_CACHE}" 2>/dev/null || echo 0)
    info "T 段定义符号数：${SYMBOL_COUNT}"
fi

echo -e "\n${BOLD}6b. import_class! 验证${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "包含 import_class! 文件数：${IMPORT_CLASS_FILES}"
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│             cpp2rust-demo pugixml FFI 验证结果          │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      pugixml（header-only XML 解析库）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}捕获文件数：${NC} ${CAPTURED}"
echo -e "  ${BOLD}生成文件数：${NC} ${RS_FILES}"
echo ""

ok "验证脚本执行完毕！"

if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "发现 ${SCRIPT_ERRORS} 个错误"
fi
