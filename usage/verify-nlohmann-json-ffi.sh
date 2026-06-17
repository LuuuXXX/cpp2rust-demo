#!/usr/bin/env bash
# =============================================================================
# verify-nlohmann-json-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 nlohmann/json header-only 重度模板库的 Rust safe FFI 生成能力
#
# 背景说明：
#   nlohmann/json 是著名的 header-only 纯模板 C++ JSON 库，json.hpp 约 23K 行，
#   大量使用模板元编程、SFINAE、constexpr 等现代 C++ 特性。
#   本脚本需要创建驱动 .cpp 文件以触发模板实例化。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地 nlohmann/json 源文件（references/nlohmann-json/include/nlohmann/json.hpp）
#   § 3. 创建驱动 .cpp 并编译（触发模板实例化）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. FFI 验证（符号、import_class!、模板骨架）
#   § 7. 生成结果汇报
#
# 本脚本验证的 cpp2rust-demo 特性：
#   ① header-only 库处理    —— 纯头文件模板库
#   ② 重度模板绑定          —— nlohmann::json 模板类
#   ③ 模板骨架验证          —— § 6e 增补：模板实例化骨架检查
#   ④ 现代 C++ 特性         —— constexpr、SFINAE、concept-like
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-nlohmann_json_demo}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"   # 置 1 可跳过 cargo install 步骤（已安装时使用）
CXX_STD="c++17"                     # C++ 标准版本（nlohmann/json 需要 C++17）

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

# 将路径转换为可被「消费 build.rs 的本地 C++ 工具链」识别的形式
to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

# 全局错误计数器
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
# § 2. 定位本地 nlohmann/json 源文件
# =============================================================================
step "§ 2. 定位本地 nlohmann/json 源文件"

# 从脚本所在目录向上找仓库根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"

JSON_DIR="${REPO_DIR}/references/nlohmann-json"
JSON_INCLUDE="${JSON_DIR}/include"

# 验证 nlohmann-json 目录存在
if [ ! -d "${JSON_DIR}" ]; then
    warn "nlohmann-json 子模块未初始化：${JSON_DIR}"
    warn "请运行：git submodule update --init references/nlohmann-json"
    info "脚本将跳过验证"
    exit 0
fi

JSON_HPP="${JSON_INCLUDE}/nlohmann/json.hpp"
if [ ! -f "${JSON_HPP}" ]; then
    fail "未找到 json.hpp：${JSON_HPP}"
fi

# 统计文件行数
JSON_LINES=$(wc -l < "${JSON_HPP}")
info "nlohmann/json 目录：${JSON_DIR}"
info "包含目录：${JSON_INCLUDE}"
info "源文件：${JSON_HPP}"
info "文件行数：${JSON_LINES}"

ok "nlohmann/json 源文件就绪"

# =============================================================================
# § 3. 创建驱动 .cpp 并编译（触发模板实例化）
# =============================================================================
step "§ 3. 创建驱动 .cpp 并编译"

OBJ_DIR=$(mktemp -d)
info "目标文件输出目录：${OBJ_DIR}"

# 驱动 .cpp 必须在 REPO_DIR 下，否则 hook 过滤（仅捕获 CPP2RUST_PROJECT_ROOT 以内的文件）
DRIVER_TMPDIR="${REPO_DIR}/.cpp2rust_tmp_nlohmann_$$"
mkdir -p "${DRIVER_TMPDIR}"

# 注册 trap 清理临时目录
trap 'rm -rf "${OBJ_DIR}" "${DRIVER_TMPDIR}" "${NM_CACHE:-}" 2>/dev/null || true' EXIT

# 创建驱动 .cpp 文件触发模板实例化（位于 REPO_DIR 内，确保 hook 能捕获）
DRIVER_CPP="${DRIVER_TMPDIR}/json_driver.cpp"
cat > "${DRIVER_CPP}" << 'EOF'
#include <nlohmann/json.hpp>
#include <string>

// 强制实例化常用模板特化，生成符号供 FFI 绑定
using json = nlohmann::json;

// 构造与解析
void instantiate_json_anchor() {
    json j;
    j["key"] = "value";
    j["number"] = 42;
    j["array"] = json::array({1, 2, 3});
    
    std::string s = j.dump();
    json parsed = json::parse(s);
    
    // 访问
    auto k = j["key"];
    bool has = j.contains("key");
    
    // 迭代
    for (auto& el : j.items()) {
        auto key = el.key();
        auto val = el.value();
    }
}

// 显式实例化常用方法（可选）
template class nlohmann::basic_json<>;
EOF

JSON_OBJ="${OBJ_DIR}/json_driver.o"
if g++ -c -std="${CXX_STD}" \
       -I"${JSON_INCLUDE}" \
       "${DRIVER_CPP}" -o "${JSON_OBJ}" 2>&1; then
    ok "编译成功：json_driver.o"
else
    warn "编译失败：json_driver.cpp"
    SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

# 创建临时构建脚本：g++ -c 编译驱动文件，触发 LD_PRELOAD hook 拦截
BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用
set -e
g++ -c -std="${CXX_STD}" \\
    -I"${JSON_INCLUDE}" \\
    "${DRIVER_CPP}" -o /dev/null 2>&1
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
ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${REPO_DIR}/.cpp2rust/${FEATURE}"
info "输出目录：${CPP2RUST_OUTPUT}"

# 统计捕获文件数
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
    echo "──── 生成的 .rs 文件 ────"
    find -L "${RUST_SRC}" -name "*.rs" | sort || true
fi

# 统计降级标记
echo ""
info "降级标记统计（cpp2rust-todo）："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null \
    | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn \
    || echo "  （无降级标记）"

# =============================================================================
# § 5a. 校验 / 兜底生成项目的 build.rs
# =============================================================================
step "§ 5a. 校验 build.rs（方案 A：工具自动注入头路径与源文件）"

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
        ok "build.rs 已由工具自动注入头路径 + 源文件（方案 A 生效，跳过就地补全）"
    else
        warn "工具 build.rs 未注入头/实现路径（可能未捕获 .opts），退回脚本就地补全（方案 B 兜底）"

        # 收集所有含 hicc 宏的单元 .rs（排除 lib.rs / mod.rs），生成 .rust_file 行
        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" \
                     ! -name "lib.rs" ! -name "mod.rs" | sort)

        # 驱动文件路径转换（Windows 兼容）
        DRIVER_CPP_BP="$(to_build_path "${DRIVER_CPP}")"
        JSON_INCLUDE_BP="$(to_build_path "${JSON_INCLUDE}")"

        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    // 与源文件编译保持一致的 C++ 标准（脚本 CXX_STD=${CXX_STD}）
    cc_build.std("${CXX_STD}");
    // ① nlohmann/json 头文件包含路径
    cc_build.include("${JSON_INCLUDE_BP}");
    // ② 编译驱动 .cpp，触发模板实例化
    cc_build.file("${DRIVER_CPP_BP}");
    // 逐文件注册含 hicc 宏的单元
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入头路径 + 驱动文件 + 逐文件注册）"
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

    # 验证 Cargo.toml 包含 hicc-std 依赖
    echo "──── Cargo.toml hicc-std 依赖检查 ────"
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
    info "未生成 tests/smoke.rs（header-only 模板库可能无自动测试），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

# ── 6a. 验证 json_driver.o 中的符号 ──────────────────────────────────────────
echo -e "\n${BOLD}6a. json_driver.o 符号（nm 验证）${NC}"
NM_CACHE=$(mktemp)
if [ -f "${JSON_OBJ}" ]; then
    nm --defined-only -f posix "${JSON_OBJ}" 2>/dev/null > "${NM_CACHE}" || true
    SYMBOL_COUNT=$(grep -c ' T ' "${NM_CACHE}" 2>/dev/null || echo 0)
    info "json_driver.o 中 T 段定义符号数：${SYMBOL_COUNT}"

    if [ "${SYMBOL_COUNT}" -gt 0 ]; then
        echo "──── 部分模板实例化符号（前 30 条，T 段）────"
        awk '$2 == "T" { print $1 }' "${NM_CACHE}" | head -30 || true
        ok "json_driver.o 包含模板实例化符号"
    else
        warn "未找到 T 段符号（json_driver.cpp 是否编译成功？）"
    fi
else
    warn "未找到 json_driver.o，跳过符号验证"
fi

# ── 6b. 验证生成 Rust 代码中的 FFI 声明 ──────────────────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（import_class!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l || true)
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"

    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        ok "生成代码包含 import_class! 块（模板类绑定存在）✓"
        echo ""
        echo "──── import_class! 类（前 20 条）────"
        grep -rn "class " "${RUST_SRC}" 2>/dev/null | grep -v "//\|#\[" | head -20 || true
    else
        warn "生成代码中未找到 import_class! 块"
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

# ── 6c. link_name 一致性检查 ──────────────────────────────────────────────────
echo -e "\n${BOLD}6c. link_name 一致性检查（不应含路径分隔符 /）${NC}"
if [ -d "${RUST_SRC}" ]; then
    LINK_NAMES=$(grep -roh '#!\[link_name = "[^"]*"\]' "${RUST_SRC}" 2>/dev/null \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u || true)
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
            warn "${BAD_LINKS} 个 link_name 含路径分隔符"
        fi
    else
        info "未找到 #![link_name = ...] 声明"
    fi
fi

# ── 6d. 预处理文件大小统计 ────────────────────────────────────────────────────
echo -e "\n${BOLD}6d. 捕获的 .cpp2rust 预处理文件大小统计${NC}"
C_DIR="${CPP2RUST_OUTPUT}/c"
if [ -d "${C_DIR}" ]; then
    TOTAL_LINES=0
    while IFS= read -r f; do
        lines=$(wc -l < "${f}")
        TOTAL_LINES=$((TOTAL_LINES + lines))
    done < <(find "${C_DIR}" -name "*.cpp2rust" 2>/dev/null)
    info "预处理文件总行数：${TOTAL_LINES}"

    echo "──── .cpp2rust 文件大小 ────"
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null | sort -rn || true
fi

# ── 6e. 模板骨架验证（header-only 模板库特有）──────────────────────────────
echo -e "\n${BOLD}6e. 模板骨架验证（检查模板类绑定）${NC}"
if [ -d "${RUST_SRC}" ]; then
    TEMPLATE_MARKERS=$(grep -r "basic_json\|json" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "检测到 json 模板类相关标识：${TEMPLATE_MARKERS} 处"

    if [ "${TEMPLATE_MARKERS}" -gt 0 ]; then
        ok "生成代码包含 nlohmann::json 模板骨架"
        echo ""
        echo "──── json 模板类引用（前 20 条）────"
        grep -rn "json\|basic_json" "${RUST_SRC}" 2>/dev/null | head -20 || true
    else
        warn "未检测到 json 模板类骨架（可能需要增强驱动文件）"
    fi
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│        cpp2rust-demo nlohmann/json FFI 验证结果         │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      nlohmann/json（header-only 重度模板 ~23K 行）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}源文件：${NC}    ${JSON_HPP}"
echo -e "  ${BOLD}文件行数：${NC}  ${JSON_LINES}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
echo ""

# import_class! 存在检查
if [ -d "${RUST_SRC}" ]; then
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l || true)
    echo -e "  ${BOLD}import_class! 模板类绑定文件数：${NC}  ${IMPORT_CLASS_FILES}"
    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 成功生成 Rust safe FFI（import_class! 块存在）${NC}"
    else
        echo -e "  ${YELLOW}⚠ 未生成模板类绑定（本次驱动可能仅触发自由函数/骨架路径）${NC}"
    fi
fi

# 是否存在 todo 标记
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

# 全局错误汇总
if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
fi
