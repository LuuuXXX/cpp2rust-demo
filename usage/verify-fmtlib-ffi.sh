#!/usr/bin/env bash
# =============================================================================
# verify-fmtlib-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 {fmt} 格式化库的完整 Rust safe FFI 生成能力
#
# 背景说明：
#   {fmt} 是现代 C++ 格式化库，包含多个 .cc 源文件（format.cc、os.cc），
#   fmt:: 命名空间，format_string / formatter 模板类，以及 wide string 支持。
#   工具可直接对其源文件运行，通过 import_class! 生成类绑定，并验证
#   多翻译单元合并能力与非标准扩展名（.cc）的预处理支持。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位 fmtlib 源码（references/fmtlib 子模块 或 临时克隆）
#   § 3. 逐文件编译 format.cc / os.cc（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 5a. 校验 build.rs（方案 A 自动注入 / 方案 B 兜底补全）
#   § 5b. cargo check        —— 验证生成代码可编译
#   § 5c. cargo test         —— 验证冒烟测试通过
#   § 6. FFI 验证（nm / import_class! / link_name / 预处理文件）
#   § 7. 生成结果汇报
#
# 可配置环境变量：
#   FEATURE       （默认 fmtlib）      cpp2rust-demo feature 名称
#   SKIP_INSTALL  （默认 0）           置 1 跳过 cargo install
#   CXX_STD       （默认 c++17）       C++ 标准版本
#
# 用法示例：
#   bash usage/verify-fmtlib-ffi.sh
#   SKIP_INSTALL=1 bash usage/verify-fmtlib-ffi.sh
#   CXX_STD=c++14 FEATURE=my_fmtlib bash usage/verify-fmtlib-ffi.sh
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-fmtlib}"
SKIP_INSTALL="${SKIP_INSTALL:-0}"
CXX_STD="${CXX_STD:-c++17}"
TMPDIR="${TMPDIR:-/tmp}"

# fmtlib 源文件列表（相对 FMTLIB_DIR）
FMT_SOURCES=("src/format.cc" "src/os.cc")

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
    warn "未检测到 libclang（可能未安装 libclang-dev）。cpp2rust-demo 依赖 libclang 解析 AST。"
    warn "  Ubuntu/Debian：sudo apt-get install -y clang libclang-dev"
fi

ok "环境检查完成"

# =============================================================================
# § 1. 安装 cpp2rust-demo
# =============================================================================
step "§ 1. 安装 cpp2rust-demo"

if [ "${SKIP_INSTALL}" = "1" ] && command -v cpp2rust-demo &>/dev/null; then
    ok "已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）"
else
    info "从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…"
    INSTALL_LOG=$(mktemp)
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked cpp2rust-demo \
             >"${INSTALL_LOG}" 2>&1; then
        echo "── cargo install 失败，完整日志：──"
        cat "${INSTALL_LOG}"
        rm -f "${INSTALL_LOG}"
        fail "cpp2rust-demo 安装失败，请检查上方日志"
    fi
    tail -5 "${INSTALL_LOG}"
    rm -f "${INSTALL_LOG}"
    ok "cpp2rust-demo 安装完成：$(which cpp2rust-demo)"
fi

cpp2rust-demo --version 2>/dev/null || true

# =============================================================================
# § 2. 定位 fmtlib 源码
# =============================================================================
step "§ 2. 定位 fmtlib 源码"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"

SUBMODULE_DIR="${REPO_DIR}/references/fmtlib"
CLONE_DIR="${TMPDIR}/fmtlib-ffi-demo"

if [ -f "${SUBMODULE_DIR}/src/format.cc" ]; then
    FMTLIB_DIR="${SUBMODULE_DIR}"
    info "使用仓库子模块：${FMTLIB_DIR}"
else
    info "子模块未初始化，克隆 fmtlib 到 ${CLONE_DIR}…"
    if [ -d "${CLONE_DIR}" ]; then
        info "目录已存在，尝试 git pull…"
        git -C "${CLONE_DIR}" pull --ff-only 2>/dev/null || true
    else
        git clone --depth 1 https://github.com/fmtlib/fmt.git "${CLONE_DIR}"
    fi
    FMTLIB_DIR="${CLONE_DIR}"
fi

if [ ! -f "${FMTLIB_DIR}/src/format.cc" ]; then
    fail "未找到 src/format.cc：${FMTLIB_DIR}/src/format.cc"
fi

FMT_INCLUDE="${FMTLIB_DIR}/include"
FMT_SRC="${FMTLIB_DIR}/src"
info "fmtlib 源码目录：${FMTLIB_DIR}"
info "fmtlib include：${FMT_INCLUDE}"
ok "fmtlib 源码就绪"

# =============================================================================
# § 3. 逐文件编译 format.cc / os.cc（供 nm 符号验证）
# =============================================================================
step "§ 3. 逐文件编译 fmtlib 源文件"

OBJ_DIR=$(mktemp -d)
trap 'rm -rf "${OBJ_DIR}" "${NM_CACHE:-}" 2>/dev/null || true' EXIT

COMPILE_ERRORS=0
for src_rel in "${FMT_SOURCES[@]}"; do
    src="${FMTLIB_DIR}/${src_rel}"
    name="$(basename "${src}" .cc)"
    obj="${OBJ_DIR}/${name}.o"
    compile_log="${OBJ_DIR}/${name}.log"
    if [ ! -f "${src}" ]; then
        warn "源文件不存在，跳过：${src}"
        continue
    fi
    if g++ -c -std="${CXX_STD}" \
           -I"${FMT_INCLUDE}" \
           "${src}" -o "${obj}" >"${compile_log}" 2>&1; then
        info "编译成功：${src_rel} → ${name}.o"
    else
        warn "编译失败：${src_rel}"
        cat "${compile_log}" >&2
        COMPILE_ERRORS=$((COMPILE_ERRORS + 1))
    fi
done

if [ "${COMPILE_ERRORS}" -gt 0 ]; then
    warn "${COMPILE_ERRORS} 个文件编译失败，nm 验证可能不完整"
else
    ok "全部 fmtlib 源文件编译成功"
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

# 创建临时构建脚本：逐文件调用 g++ -c，触发 LD_PRELOAD hook 拦截
BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用
set -e
for src_rel in ${FMT_SOURCES[*]}; do
    src="${FMTLIB_DIR}/\${src_rel}"
    if [ -f "\${src}" ]; then
        g++ -c -std="${CXX_STD}" \\
            -I"${FMT_INCLUDE}" \\
            "\${src}" -o /dev/null 2>&1
    fi
done
EOF
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"
info "构建脚本：${BUILD_SCRIPT}"

cd "${REPO_DIR}"
cpp2rust-demo init \
    --feature "${FEATURE}" \
    -- bash "${BUILD_SCRIPT}"

rm -f "${BUILD_SCRIPT}"
ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${REPO_DIR}/.cpp2rust/${FEATURE}"
info "输出目录：${CPP2RUST_OUTPUT}"

CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
info "捕获预处理文件数：${CAPTURED}"

# =============================================================================
# § 5. cpp2rust-demo merge — 整理输出目录
# =============================================================================
step "§ 5. cpp2rust-demo merge（整理输出结构）"

cd "${REPO_DIR}"
cpp2rust-demo merge --feature "${FEATURE}"
ok "merge 完成"

RUST_SRC="${CPP2RUST_OUTPUT}/rust/src"
RS_FILES=$(find -L "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l)
info "生成 Rust 文件数：${RS_FILES}"
if [ "${RS_FILES}" -gt 0 ]; then
    echo "──── 生成的 .rs 文件（前 20 条）────"
    find -L "${RUST_SRC}" -name "*.rs" | sort | head -20 || true
fi

echo ""
info "降级标记统计（cpp2rust-todo）："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null \
    | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn \
    || echo "  （无降级标记）"

# =============================================================================
# § 5a. 校验 / 兜底生成项目的 build.rs
# =============================================================================
step "§ 5a. 校验 build.rs（方案 A：工具自动注入 / 方案 B：兜底补全）"

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
BUILD_RS="${RUST_PROJECT}/build.rs"
LIB_NAME="${FEATURE//-/_}"

if [ -f "${BUILD_RS}" ]; then
    info "工具生成的 build.rs："
    sed 's/^/    /' "${BUILD_RS}"

    BUILD_META="${CPP2RUST_OUTPUT}/meta/build-meta.json"
    if [ -f "${BUILD_META}" ]; then
        info "编译元数据 meta/build-meta.json（方案 A 落盘）："
        sed 's/^/    /' "${BUILD_META}"
    fi

    if grep -q 'cc_build.include' "${BUILD_RS}" && grep -q 'cc_build.file' "${BUILD_RS}"; then
        ok "build.rs 已由工具自动注入头路径 + 实现文件（方案 A 生效，跳过就地补全）"
    else
        warn "工具 build.rs 未注入头/实现路径，退回脚本就地补全（方案 B 兜底）"

        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" \
                     ! -name "lib.rs" ! -name "mod.rs" | sort)

        # 收集各 .cc 源文件路径
        CC_FILE_LINES=""
        for src_rel in "${FMT_SOURCES[@]}"; do
            src="${FMTLIB_DIR}/${src_rel}"
            if [ -f "${src}" ]; then
                src_bp="$(to_build_path "${src}")"
                CC_FILE_LINES="${CC_FILE_LINES}    cc_build.file(\"${src_bp}\");
"
            fi
        done

        FMT_INCLUDE_BP="$(to_build_path "${FMT_INCLUDE}")"

        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.std("${CXX_STD}");
    // fmtlib 头文件包含路径
    cc_build.include("${FMT_INCLUDE_BP}");
    // 编译 fmtlib 实现文件（format.cc、os.cc）
${CC_FILE_LINES}
    // 逐文件注册含 hicc 宏的单元，生成 _hicc_export_methods_* 导出函数
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入头路径 + fmtlib .cc 文件 + 逐文件注册）"
    fi
else
    warn "未找到 ${BUILD_RS}，跳过 build.rs 校验/补全"
fi

# =============================================================================
# § 5b. cargo check — 验证生成的 Rust 项目可编译
# =============================================================================
step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"

if [ -f "${RUST_PROJECT}/Cargo.toml" ]; then
    info "在 ${RUST_PROJECT} 中运行 cargo check ..."

    if grep -q 'hicc-std' "${RUST_PROJECT}/Cargo.toml"; then
        ok "Cargo.toml 已包含 hicc-std 依赖"
    else
        warn "Cargo.toml 未找到 hicc-std 依赖（可能影响 STL 类型绑定）"
    fi
    cat "${RUST_PROJECT}/Cargo.toml"

    if (cd "${RUST_PROJECT}" && cargo check 2>&1); then
        ok "cargo check 通过 ✓"
    else
        fail "cargo check 失败 — 生成的 FFI 代码存在编译错误，请检查上方 cargo check 输出"
    fi
else
    warn "未找到 ${RUST_PROJECT}/Cargo.toml，跳过 cargo check"
fi

# =============================================================================
# § 5c. cargo test — 验证生成的冒烟测试通过
# =============================================================================
step "§ 5c. cargo test（验证生成的冒烟测试可编译、可链接、可运行）"

SMOKE_FILE="${RUST_PROJECT}/tests/smoke.rs"
if [ -f "${SMOKE_FILE}" ]; then
    info "检测到冒烟测试文件：${SMOKE_FILE}"
    if (cd "${RUST_PROJECT}" && cargo test 2>&1); then
        ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
    else
        warn "cargo test 失败 — 请检查上方链接日志"
    fi
else
    info "未生成 tests/smoke.rs（可能 init 阶段无可往返测试的 pub 类型），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

# ── 6a. nm 查看 .o 中的 T 段符号 ─────────────────────────────────────────────
echo -e "\n${BOLD}6a. fmtlib 目标文件符号（nm 验证，预期 _ZN3fmt* 等）${NC}"
NM_CACHE=$(mktemp)
find "${OBJ_DIR}" -name "*.o" 2>/dev/null \
    | xargs -r nm --defined-only -f posix 2>/dev/null > "${NM_CACHE}" || true

T_COUNT=$(grep -c ' T ' "${NM_CACHE}" 2>/dev/null || echo 0)
info "目标文件 T 段定义符号数：${T_COUNT}"

if [ "${T_COUNT}" -gt 0 ]; then
    echo "──── 部分 T 段符号（前 20 条）────"
    awk '$2 == "T" { print $1 }' "${NM_CACHE}" | head -20 || true
    if grep -q '_ZN3fmt' "${NM_CACHE}" 2>/dev/null; then
        ok "检测到 fmt:: 命名空间符号（_ZN3fmt*）✓"
    else
        warn "未检测到 _ZN3fmt* 符号（可能 fmt 命名空间未编译）"
    fi
else
    warn "未找到 T 段符号（fmtlib 源文件是否编译成功？）"
fi

# ── 6b. 验证生成代码含 import_class!/import_lib! 块 ───────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 import_class!/import_lib! 块验证${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"
    info "包含 import_lib! 绑定的文件数：${IMPORT_LIB_FILES}"

    TOTAL_BINDING_FILES=$((IMPORT_CLASS_FILES + IMPORT_LIB_FILES))
    if [ "${TOTAL_BINDING_FILES}" -gt 0 ]; then
        ok "生成代码包含 FFI 绑定块（import_class! 或 import_lib! 存在）✓"
        if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
            echo "──── import_class! 中的类（前 10 条）────"
            grep -rh "class " "${RUST_SRC}" 2>/dev/null | grep -v '//' | head -10 || true
        fi
    else
        warn "生成代码中未找到 import_class! 或 import_lib! 块"
        warn "（fmtlib 重度模板系统可能导致工具未提取到可绑定符号）"
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

# ── 6c. link_name 不含路径分隔符 ─────────────────────────────────────────────
echo -e "\n${BOLD}6c. link_name 一致性检查（不应含路径分隔符 /）${NC}"
LINK_NAMES=$(grep -roh '#!\[link_name = "[^"]*"\]' "${RUST_SRC}" 2>/dev/null \
    | grep -oE '"[^"]*"' | tr -d '"' | sort -u)
if [ -n "${LINK_NAMES}" ]; then
    BAD_LINKS=0
    while IFS= read -r ln; do
        if echo "${ln}" | grep -q '/'; then
            echo -e "  ${RED}✗ link_name 含路径分隔符：${ln}${NC}"
            BAD_LINKS=$((BAD_LINKS + 1))
        else
            echo -e "  ${GREEN}✓ ${ln}${NC}"
        fi
    done < <(echo "${LINK_NAMES}")
    if [ "${BAD_LINKS}" -eq 0 ]; then
        ok "所有 link_name 均为纯文件名（link_name 一致性通过）"
    else
        warn "${BAD_LINKS} 个 link_name 含路径分隔符，请检查提取器输出"
    fi
else
    info "未找到 #![link_name = ...] 声明（可能全部通过 import_class! 绑定）"
fi

# ── 6d. 预处理文件行数统计 ────────────────────────────────────────────────────
echo -e "\n${BOLD}6d. 捕获的 .cpp2rust 预处理文件行数统计${NC}"
C_DIR="${CPP2RUST_OUTPUT}/c"
if [ -d "${C_DIR}" ]; then
    TOTAL_LINES=0
    while IFS= read -r f; do
        lines=$(wc -l < "${f}")
        TOTAL_LINES=$((TOTAL_LINES + lines))
    done < <(find "${C_DIR}" -name "*.cpp2rust" 2>/dev/null)
    info "预处理文件总行数：${TOTAL_LINES}"

    echo "──── 各 .cpp2rust 文件大小（前 15 条）────"
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null \
        | sort -rn | head -15 || true
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│           cpp2rust-demo fmtlib FFI 验证结果             │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      {fmt}（多文件 C++ 格式化库）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}源码目录：${NC}  ${FMTLIB_DIR}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
echo ""

if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    echo -e "  ${BOLD}import_class! 绑定文件数：${NC}  ${IMPORT_CLASS_FILES}"
    echo -e "  ${BOLD}import_lib!   绑定文件数：${NC}  ${IMPORT_LIB_FILES}"
    TOTAL_BINDING_FILES=$((IMPORT_CLASS_FILES + IMPORT_LIB_FILES))
    if [ "${TOTAL_BINDING_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 成功生成 Rust safe FFI（FFI 绑定块存在）${NC}"
    else
        echo -e "  ${YELLOW}⚠ 未生成 FFI 绑定（fmtlib 模板系统复杂，可能仅生成 hicc::cpp! 块）${NC}"
    fi
fi

TODO_COUNT=$(grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || TODO_COUNT=0
if [ "${TODO_COUNT}" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ 降级标记（需手动完善）：${TODO_COUNT} 处${NC}"
    echo "  → 搜索 'cpp2rust-todo' 查看详情：grep -rn cpp2rust-todo ${RUST_SRC}"
else
    echo -e "  ${GREEN}✓ 无降级标记${NC}"
fi

echo ""
echo -e "  ${BOLD}查看生成的 Rust FFI 脚手架：${NC}"
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs head -80"
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -l 'import_class\|import_lib'"
echo ""

ok "验证脚本执行完毕！"

if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
fi
