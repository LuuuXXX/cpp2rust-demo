#!/usr/bin/env bash
# =============================================================================
# verify-magic-enum-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 magic_enum header-only 库的解析鲁棒性
#
# 背景说明：
#   magic_enum 是 header-only 库（magic_enum.hpp，C++17 重度 constexpr + 模板元编程），用于验证工具对「极端模板/constexpr 头文件」的解析鲁棒性（不崩溃/不超时）。以「驱动文件」模式演示工作流。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地源文件
#   § 3. 编译源文件（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. FFI 验证
#   § 7. 生成结果汇报
#
# 工作流说明（重要）：
#   cpp2rust-demo 通过编译拦截捕获 C/C++ 编译命令，结合 libclang AST 提取
#   extern-C 函数 / C++ 类定义，并生成 hicc 三段式 Rust FFI 脚手架
#   （hicc::cpp! + import_class! + import_lib!）。
#
# 本脚本验证的 cpp2rust-demo 特性：
#   ① 驱动文件提取 —— 工具从包含 magic_enum.hpp 的驱动文件提取类绑定
#   ② import_class! 生成 —— ColorWrapper 等类生成完整的 FFI 绑定块
#   ③ constexpr / 模板鲁棒性 —— 极端模板头文件场景可被稳定解析
#   ④ hicc::cpp! 块 —— 生成必要的 C++ 内联 shim
#   ⑤ hicc-std 依赖声明 —— 生成的 Cargo.toml 包含 hicc-std 依赖
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================
set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-magic_enum_ffi}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"   # 置 1 可跳过 cargo install 步骤（已安装时使用）
CXX_STD="c++17"                     # C++ 标准版本
TMPDIR="${TMPDIR:-/tmp}"            # 系统临时目录（Linux）

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

# 将路径转换为可被「消费 build.rs 的本地 C++ 工具链」识别的形式。
# 背景：在 MSYS2 / Cygwin 上本脚本以 POSIX 路径（/d/a/...）运行，但生成项目的
#   cargo check / cargo test 默认使用原生 Windows 工具链（MSVC cl.exe）。若把
#   POSIX 路径原样写入 build.rs 的 cc_build.include()/cc_build.file()，cl.exe 会：
#     · 对 -I /d/a/... 报 “Cannot open include file”（找不到目录）；
#     · 对前导 “/” 的源文件报 D9002（被当作未知选项而忽略）。
#   用 `cygpath -m` 转为带盘符的正斜杠混合路径（D:/a/...），MSVC cl.exe 与
#   MinGW gcc 均可识别。非 Windows 环境无 cygpath，原样返回，保持 Linux 行为不变。
to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

# 全局错误计数器：非致命但重要的检查失败时递增，脚本末尾统一 exit 1
SCRIPT_ERRORS=0

# =============================================================================
# § 0. 环境检查
# =============================================================================
step "§ 0. 环境检查"

need_cmd git
need_cmd g++
need_cmd cargo
need_cmd nm

# 检查 libclang
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
# § 2. 定位本地源文件
# =============================================================================
step "§ 2. 定位本地源文件"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"
SOURCE_DIR="${REPO_DIR}/references/magic_enum"
MAGIC_ENUM_INCLUDE="${SOURCE_DIR}/include"

if [ ! -d "${SOURCE_DIR}" ] || [ ! -d "${MAGIC_ENUM_INCLUDE}" ]; then
    info "references/magic_enum 子模块未初始化，正在初始化..."
    git -C "${REPO_DIR}" submodule update --init references/magic_enum || \
        fail "子模块初始化失败：references/magic_enum；请检查网络或手动执行 git submodule update --init references/magic_enum"
fi

if [ ! -d "${SOURCE_DIR}" ] || [ ! -d "${MAGIC_ENUM_INCLUDE}" ]; then
    fail "未找到 magic_enum include 目录：${MAGIC_ENUM_INCLUDE}"
fi

DRIVER_TMP=$(mktemp "${REPO_DIR}/tmpXXXXXX.cpp")
cat > "${DRIVER_TMP}" << 'EOF'
// magic_enum 驱动文件 — 测试重度 constexpr/模板元编程头文件的解析鲁棒性
#include <magic_enum/magic_enum.hpp>

namespace enumwrap_ns {

enum class Color { Red, Green, Blue };

class ColorWrapper {
public:
    ColorWrapper();
    int count() const;
    bool is_valid() const;
    void reset();
};

}  // namespace enumwrap_ns
EOF

info "magic_enum include：${MAGIC_ENUM_INCLUDE}"
info "临时 driver 文件：${DRIVER_TMP}"
ok "magic_enum 驱动文件就绪"

# =============================================================================
# § 3. 编译源文件（供 nm 符号验证）
# =============================================================================
step "§ 3. 编译源文件（供 nm 符号验证）"

OBJ_DIR=$(mktemp -d)
info "目标文件输出目录：${OBJ_DIR}"
trap 'rm -rf "${OBJ_DIR}" "${NM_CACHE:-}" "${DRIVER_TMP:-}" "${WRAPPER_TMP:-}" "${BUILD_SCRIPT:-}" 2>/dev/null || true' EXIT
if g++ -c -std="${CXX_STD}" -I"${MAGIC_ENUM_INCLUDE}" "${DRIVER_TMP}" -o "${OBJ_DIR}/magic_enum_driver.o" 2>&1; then
    ok "magic_enum driver 编译成功"
else
    warn "编译失败（模板 / constexpr 展开较重），nm 验证可能不完整"
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用
set -e
g++ -c -std="${CXX_STD}" -I"${MAGIC_ENUM_INCLUDE}" "${DRIVER_TMP}" -o /dev/null 2>&1
EOF
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"
info "构建命令：bash ${BUILD_SCRIPT}"

cd "${REPO_DIR}"
cpp2rust-demo init \
    --feature "${FEATURE}" \
    -- bash "${BUILD_SCRIPT}"

rm -f "${BUILD_SCRIPT}"
BUILD_SCRIPT=""
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
step "§ 5a. 校验 build.rs（方案 A：工具自动注入构建元数据）"

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

    if grep -q 'cc_build.include' "${BUILD_RS}"; then
        ok "build.rs 已包含所需 include / file 元数据（方案 A 生效，跳过就地补全）"
    else
        warn "工具 build.rs 未包含所需构建元数据，退回脚本就地补全（方案 B 兜底）"

        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" \
                     ! -name "lib.rs" ! -name "mod.rs" | sort)
        MAGIC_ENUM_INCLUDE_BP="$(to_build_path "${MAGIC_ENUM_INCLUDE}")"
        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.std("${CXX_STD}");
    cc_build.include("${MAGIC_ENUM_INCLUDE_BP}");
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（方案 B）"
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

    echo "──── 特性⑤ Cargo.toml hicc-std 依赖检查 ────"
    if grep -q 'hicc-std' "${RUST_PROJECT}/Cargo.toml"; then
        ok "Cargo.toml 已包含 hicc-std 依赖（特性⑤ 通过）"
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
        warn "cargo test 失败 — 请检查上方链接日志..."
    fi
else
    info "未生成 tests/smoke.rs，跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

# ── 6a. 目标文件 C++ 符号（nm 验证） ─────────────────────────────────────────
echo -e "\n${BOLD}6a. 目标文件 C++ 符号（nm 验证）${NC}"
NM_CACHE=$(mktemp)
find "${OBJ_DIR}" -name "*.o" 2>/dev/null \
    | xargs -r nm -C --defined-only 2>/dev/null > "${NM_CACHE}" || true

CPP_SYMBOLS=$(grep -c ' [TWBV] ' "${NM_CACHE}" 2>/dev/null || echo 0)
info "目标文件中已定义符号数：${CPP_SYMBOLS}"
if [ "${CPP_SYMBOLS}" -gt 0 ]; then
    echo "──── 部分已定义符号（前 20 条）────"
    head -20 "${NM_CACHE}" || true
    ok "已捕获 magic_enum 驱动文件 的 C++ 符号"
else
    warn "未找到已定义符号（编译失败时 nm 验证可能为空）"
fi

# ── 6b. 生成 Rust 代码中的 OOP 绑定 ─────────────────────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 OOP 绑定（import_class! / hicc::cpp!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    CPP_BLOCK_FILES=$(grep -rl "hicc::cpp!" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"
    info "包含 import_lib! 绑定的文件数：${IMPORT_LIB_FILES}"
    info "包含 hicc::cpp! 块的文件数：${CPP_BLOCK_FILES}"

    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        ok "生成代码包含 import_class! 块（OOP 类绑定存在）✓"
    else
        warn "生成代码中未找到 import_class! 块（可能无 C++ 类被提取？）"
    fi

    echo "──── import_class! 绑定类（前 20 条）────"
    grep -rn "class " "${RUST_SRC}" 2>/dev/null | grep -v "//\|#\[" | head -20 || true

    echo ""
    echo "──── 关键类名检查 ────"
    if grep -q "ColorWrapper" "${RUST_SRC}" 2>/dev/null; then
        echo -e "  ${GREEN}✓ 生成代码提取到类：ColorWrapper${NC}"
    else
        echo -e "  ${YELLOW}? 生成代码中未直接找到类：ColorWrapper${NC}"
    fi
    if grep -q "Color" "${RUST_SRC}" 2>/dev/null; then
        echo -e "  ${GREEN}✓ 生成代码提取到类：Color${NC}"
    else
        echo -e "  ${YELLOW}? 生成代码中未直接找到类：Color${NC}"
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

# ── 6c. 关键类名交叉比对（生成代码 vs. nm 符号） ────────────────────────────
echo -e "\n${BOLD}6c. 关键类名交叉比对（生成代码 vs. nm 符号）${NC}"
if [ -s "${NM_CACHE}" ]; then
    if grep -q "ColorWrapper" "${NM_CACHE}" 2>/dev/null; then
        echo -e "  ${GREEN}✓ nm 符号中包含：ColorWrapper${NC}"
    else
        echo -e "  ${YELLOW}? nm 符号中未直接看到：ColorWrapper${NC}"
    fi
    if grep -q "Color" "${NM_CACHE}" 2>/dev/null; then
        echo -e "  ${GREEN}✓ nm 符号中包含：Color${NC}"
    else
        echo -e "  ${YELLOW}? nm 符号中未直接看到：Color${NC}"
    fi
else
    warn "nm 结果为空，跳过类名交叉比对"
fi

# ── 6d. 捕获的 .cpp2rust 预处理文件大小统计 ────────────────────────────────────
echo -e "\n${BOLD}6d. 捕获的 .cpp2rust 预处理文件大小统计${NC}"
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

# ── 6e. struct/class 前缀清理 & restrict 剥离 ──────────────────────────────────
echo -e "\n${BOLD}6e. struct/class 前缀 & restrict 清理验证${NC}"
if [ -d "${RUST_SRC}" ]; then
    STRUCT_HITS=$(grep -rn '\bstruct \b' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l) || STRUCT_HITS=0
    CLASS_HITS=$(grep -rn '\bclass \b' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l) || CLASS_HITS=0
    RESTRICT_HITS=$(grep -rn '__restrict\|[^_]restrict[^_]' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | wc -l) || RESTRICT_HITS=0

    if [ "${STRUCT_HITS}" -eq 0 ] && [ "${CLASS_HITS}" -eq 0 ]; then
        ok "Rust 绑定中无多余的 struct/class 前缀 ✓"
    else
        warn "Rust 绑定中仍有 struct/class 前缀：struct=${STRUCT_HITS} class=${CLASS_HITS}"
        grep -rn '\bstruct \b\|\bclass \b' "${RUST_SRC}" 2>/dev/null \
            | grep -v '^\s*//' | grep -v 'hicc::cpp!' | head -10 || true
    fi

    if [ "${RESTRICT_HITS}" -eq 0 ]; then
        ok "Rust 绑定中无 restrict 限定符 ✓"
    else
        warn "Rust 绑定中仍有 restrict 限定符：${RESTRICT_HITS} 处"
        grep -rn '__restrict\|[^_]restrict[^_]' "${RUST_SRC}" 2>/dev/null \
            | grep -v '^\s*//' | head -10 || true
    fi
else
    warn "Rust 源码目录不存在，跳过 struct/class/restrict 清理验证"
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│             cpp2rust-demo FFI 验证结果                  │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      magic_enum（header-only 解析鲁棒性测试）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}源文件：${NC}    ${DRIVER_TMP}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    echo -e "  ${BOLD}import_class! 绑定文件数：${NC}  ${IMPORT_CLASS_FILES}"
    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 成功生成 Rust safe FFI（import_class! 块存在）${NC}"
    else
        echo -e "  ${RED}✗ 未生成 import_class! 绑定（请检查类提取流程）${NC}"
        SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
    fi
fi

echo ""
TODO_COUNT=$(grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || TODO_COUNT=0
TODO_COUNT="${TODO_COUNT:-0}"
if [ "${TODO_COUNT}" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ 降级标记（需手动完善）：${TODO_COUNT} 处${NC}"
    echo "  → 搜索 'cpp2rust-todo' 查看详情：grep -rn cpp2rust-todo ${RUST_SRC}"
else
    echo -e "  ${GREEN}✓ 无降级标记${NC}"
fi

echo ""
echo -e "  ${BOLD}查看生成的 Rust FFI 脚手架：${NC}"
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs head -80"
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -l 'import_class'"
echo ""

ok "验证脚本执行完毕！"

if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
fi

# =============================================================================
# § SKILL 工作流（说明）
# =============================================================================
step "§ SKILL 工作流（如何通过 GitHub Copilot Agent Skill 执行相同流程）"

cat <<'EOF'

除了直接运行本脚本（CLI 方式），你也可以通过 GitHub Copilot Agent Skill
来交互式地完成 cpp2rust-demo 转换流程。

────────────────────────────────────────────────────────────────
SKILL 适用场景（header-only 鲁棒性验证工作流）：

  在 magic_enum 驱动文件目录 与 GitHub Copilot 对话，说：
    "帮我用 cpp2rust-demo 验证 magic_enum 驱动文件的 FFI 生成"
  或：
    "对这个 magic_enum driver 执行 cpp2rust-demo 转换"

────────────────────────────────────────────────────────────────
header-only constexpr 库的典型驱动文件工作流（以 magic_enum 为例）：

  1. 编写包含 magic_enum.hpp 的驱动 .cpp，并声明待提取的包装类 / 枚举

  2. 在驱动文件目录准备可复现的 build script

  3. 运行 cpp2rust-demo init
     cpp2rust-demo init --feature magic_enum_ffi -- bash build.sh

  4. 运行 cpp2rust-demo merge
     cpp2rust-demo merge --feature magic_enum_ffi

  5. 在生成的 Rust 项目中使用 import_class! 绑定调用 ColorWrapper / 枚举 API

────────────────────────────────────────────────────────────────
⚠  重要提示：
  cpp2rust-demo 对重度 constexpr / 模板库的验证重点是“能稳定解析且不中断”。
  若驱动文件过于空泛，可能只生成 hicc::cpp! 片段，
  不会生成期望的 import_class! 绑定。

  这是预期行为，不是 bug。
────────────────────────────────────────────────────────────────

EOF
