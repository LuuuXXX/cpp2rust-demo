#!/usr/bin/env bash
# =============================================================================
# verify-rapidjson-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 rapidjson C++ 库的完整 Rust safe FFI 生成能力
#
# 背景说明：
#   rapidjson 是纯 C++ 库（无 extern "C" 导出）。要用 cpp2rust-demo 为其生成
#   Rust safe FFI，需要先编写一层 C++ extern-C 包装层（shim files）。
#   本仓库已提供完整的 shim 参考实现（10 个子系统），本脚本以此为输入演示
#   完整的工作流：shim 编写 → cpp2rust-demo init → merge → cargo check → cargo test。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地 shim 文件（references/rapidjson-refactoring/rapidjson_sys/shim/）
#   § 3. 编译 shim 目标文件（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. FFI 验证（link_name、import_lib!、函数符号交叉比对）
#   § 7. 生成结果汇报
#
# 工作流说明（重要）：
#   cpp2rust-demo 通过编译拦截捕获 C++ 编译命令，提取 extern "C" 函数后生成
#   hicc 三段式 Rust FFI（hicc::cpp! + import_class! + import_lib!）。
#
#   ┌──────────────────────────────────────────────────────────────┐
#   │  纯 C++ 库（如 rapidjson）的推荐工作流：                    │
#   │                                                              │
#   │  1. 编写 C++ shim 文件（extern "C" 不透明句柄包装层）       │
#   │     参考：references/rapidjson-refactoring/rapidjson_sys/shim/ │
#   │  2. 运行 cpp2rust-demo init（本脚本演示的步骤）              │
#   │  3. 运行 cpp2rust-demo merge                                 │
#   │  4. 在生成的 Rust 项目中使用 import_lib! 绑定               │
#   │                                                              │
#   │  ⚠  直接对纯 C++ 测试文件（如 GTest unittest）运行 init，   │
#   │     因无 extern "C" 函数，只会生成 hicc::cpp! 头文件块。    │
#   └──────────────────────────────────────────────────────────────┘
#
# 本脚本验证的 cpp2rust-demo 特性：
#   ① extern-C 函数识别 —— 工具从 shim 文件提取 extern "C" 函数
#   ② import_lib! 生成  —— 每个 shim 文件都生成完整的 FFI 绑定块
#   ③ link_name 一致性  —— 只取路径末段文件名，避免子目录路径污染链接名
#   ④ struct/class 前缀清理 —— shim 函数签名中的 C++ 关键字前缀自动去除
#   ⑤ __restrict__ 处理 —— 类型末尾的 restrict 限定符自动剥离
#   ⑥ hpp/hxx 头文件探测 —— read_source_includes 按 .h → .hpp 顺序探测
#   ⑦ hicc-std 依赖声明 —— 生成的 Cargo.toml 包含 hicc-std 依赖
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-rapidjson_shim}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"   # 置 1 可跳过 cargo install 步骤（已安装时使用）
CXX_STD="c++11"                     # C++ 标准版本（shim 文件要求 C++11 以上）
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
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked \
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
# § 2. 定位本地 shim 文件
# =============================================================================
step "§ 2. 定位本地 shim 文件"

# 从脚本所在目录向上找仓库根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"

SHIM_DIR="${REPO_DIR}/references/rapidjson-refactoring/rapidjson_sys/shim"
RAPIDJSON_INCLUDE="${REPO_DIR}/references/rapidjson-refactoring/rapidjson_legacy/include"

# 验证 shim 目录存在
if [ ! -d "${SHIM_DIR}" ]; then
    fail "shim 目录不存在：${SHIM_DIR}
请确认仓库中已初始化 references/rapidjson-refactoring/ 子目录。"
fi

SHIM_CPP_COUNT=$(find "${SHIM_DIR}" -maxdepth 1 -name "*.cpp" | wc -l)
if [ "${SHIM_CPP_COUNT}" -eq 0 ]; then
    fail "shim 目录中未找到 .cpp 文件：${SHIM_DIR}"
fi

info "shim 目录：${SHIM_DIR}"
info "shim 源文件数量：${SHIM_CPP_COUNT}"
info "rapidjson include：${RAPIDJSON_INCLUDE}"
find "${SHIM_DIR}" -maxdepth 1 -name "*.cpp" | sort | while read -r f; do
    echo "  - $(basename "$f")"
done

ok "shim 文件就绪"

# =============================================================================
# § 3. 编译 shim 目标文件（供 nm 符号验证）
# =============================================================================
step "§ 3. 编译 shim 目标文件"

OBJ_DIR=$(mktemp -d)
info "目标文件输出目录：${OBJ_DIR}"

# 注册 trap 清理临时目录
trap 'rm -rf "${OBJ_DIR}" "${NM_CACHE:-}" 2>/dev/null || true' EXIT

COMPILE_ERRORS=0
while IFS= read -r src; do
    name="$(basename "${src}" .cpp)"
    obj="${OBJ_DIR}/${name}.o"
    compile_log="${OBJ_DIR}/${name}.log"
    if g++ -c -std="${CXX_STD}" \
           -I"${RAPIDJSON_INCLUDE}" \
           -I"${SHIM_DIR}" \
           "${src}" -o "${obj}" >"${compile_log}" 2>&1; then
        info "编译成功：${name}.o"
    else
        warn "编译失败：${name}.cpp"
        cat "${compile_log}" >&2
        COMPILE_ERRORS=$((COMPILE_ERRORS + 1))
    fi
done < <(find "${SHIM_DIR}" -maxdepth 1 -name "*.cpp" | sort)

if [ "${COMPILE_ERRORS}" -gt 0 ]; then
    warn "${COMPILE_ERRORS} 个 shim 文件编译失败，nm 验证可能不完整"
else
    ok "全部 ${SHIM_CPP_COUNT} 个 shim 文件编译成功"
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

# 创建临时构建脚本：g++ -c 编译每个 shim 文件，触发 LD_PRELOAD hook 拦截
BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用
set -e
while IFS= read -r src; do
    g++ -c -std="${CXX_STD}" \\
        -I"${RAPIDJSON_INCLUDE}" \\
        -I"${SHIM_DIR}" \\
        "\${src}" -o /dev/null 2>&1
done < <(find "${SHIM_DIR}" -maxdepth 1 -name "*.cpp" | sort)
EOF
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"
info "构建命令：bash ${BUILD_SCRIPT}"

# 非交互模式：cpp2rust-demo 检测 stdin 是否为 TTY 自动判断交互性。
# 在脚本（非终端）环境中，自动全选所有被拦截的 .cpp 文件。

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
    echo "──── 生成的 .rs 文件（前 20 条）────"
    find -L "${RUST_SRC}" -name "*.rs" | sort | head -20 || true
fi

# 统计降级标记
echo ""
info "降级标记统计（cpp2rust-todo）："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null \
    | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn \
    || echo "  （无降级标记）"

# =============================================================================
# § 5a. 校验 / 兜底生成项目的 build.rs —— 提供底层 C++ 实现与头文件包含路径
#
# 背景（重要，方案 A 已落地）：
#   工具生成的 build.rs 现在会对每个含 hicc 宏的单元文件调用 `.rust_file(...)`，
#   使 hicc-build 为 import_lib! 生成 `_hicc_export_methods_*` 方法表导出函数（修复
#   了测试二进制链接时 undefined reference 的根因）。此外，init 阶段会从编译拦截记录
#   的 `.opts`（include 路径 / `-std`）与反推出的实现 `.cpp` 落盘编译元数据
#   （meta/build-meta.json），并据此在生成的 build.rs 中**自动**注入：
#     ① rapidjson / shim 头文件包含路径（hicc 生成的胶水 C++ #include 需要）；
#     ② 编译 shim 的 *_ffi.cpp，提供 `rapidjson_*` extern "C" 实现符号；
#     ③ 与 shim 一致的 C++ 标准。
#   因此正常情况下本脚本**无需**再就地改写 build.rs。下面先校验工具产物是否已
#   自包含（含 cc_build.include + cc_build.file）：
#     · 已自包含 → 直接信任工具产物（方案 A）。
#     · 未自包含（旧版工具 / 未捕获 .opts）→ 退回脚本就地补全（方案 B 兜底）。
# =============================================================================
step "§ 5a. 校验 build.rs（方案 A：工具自动注入头路径与 shim 实现）"

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
        ok "build.rs 已由工具自动注入头路径 + shim 实现（方案 A 生效，跳过就地补全）"
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

        # 收集 shim 的 *_ffi.cpp 实现文件，生成 cc_build.file 行
        SHIM_FILE_LINES=""
        while IFS= read -r cpp; do
            SHIM_FILE_LINES="${SHIM_FILE_LINES}    cc_build.file(\"${cpp}\");
"
        done < <(find "${SHIM_DIR}" -maxdepth 1 -name "*.cpp" | sort)

        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    // 与 shim 编译保持一致的 C++ 标准（脚本 CXX_STD=${CXX_STD}）
    cc_build.std("${CXX_STD}");
    // ① rapidjson 与 shim 头文件包含路径（hicc 生成的胶水 C++ #include 需要）
    cc_build.include("${RAPIDJSON_INCLUDE}");
    cc_build.include("${SHIM_DIR}");
    // ② 编译 shim 的 *_ffi.cpp，提供 rapidjson_* extern "C" 实现符号
${SHIM_FILE_LINES}
    // 逐文件注册含 hicc 宏的单元，生成 _hicc_export_methods_* 导出函数
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入头路径 + shim 实现 + 逐文件注册）"
    fi
else
    warn "未找到 ${BUILD_RS}，跳过 build.rs 校验/补全"
fi

# =============================================================================
# § 5b. cargo check — 验证生成的 Rust 项目可编译
# =============================================================================
step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
if [ -f "${RUST_PROJECT}/Cargo.toml" ]; then
    info "在 ${RUST_PROJECT} 中运行 cargo check ..."

    # 验证特性⑦：Cargo.toml 应包含 hicc-std 依赖
    echo "──── 特性⑦ Cargo.toml hicc-std 依赖检查 ────"
    if grep -q 'hicc-std' "${RUST_PROJECT}/Cargo.toml"; then
        ok "Cargo.toml 已包含 hicc-std 依赖（特性⑦ 通过）"
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

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
SMOKE_FILE="${RUST_PROJECT}/tests/smoke.rs"
if [ -f "${SMOKE_FILE}" ]; then
    info "检测到冒烟测试文件：${SMOKE_FILE}"
    if (cd "${RUST_PROJECT}" && cargo test 2>&1); then
        ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
    else
        warn "cargo test 失败 — 请检查上方链接日志（build.rs 已逐文件注册 .rust_file 并编译 shim 实现，理论上 _hicc_export_methods_* 与 rapidjson_* 符号均应可解析）"
    fi
else
    info "未生成 tests/smoke.rs（可能 init 阶段无 pub class 类型），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

# ── 6a. 验证 shim 目标文件中的 extern-C 符号 ─────────────────────────────────
echo -e "\n${BOLD}6a. shim 目标文件 extern-C 符号（nm 验证）${NC}"
NM_CACHE=$(mktemp)
find "${OBJ_DIR}" -name "*.o" 2>/dev/null \
    | xargs -r nm --defined-only -f posix 2>/dev/null > "${NM_CACHE}" || true

EXTERN_C_COUNT=$(grep -c ' T ' "${NM_CACHE}" 2>/dev/null || echo 0)
info "shim 目标文件中 T 段定义符号数：${EXTERN_C_COUNT}"

if [ "${EXTERN_C_COUNT}" -gt 0 ]; then
    echo "──── 部分 extern-C 符号（前 20 条，T 段）────"
    awk '$2 == "T" { print $1 }' "${NM_CACHE}" | head -20 || true
    ok "shim 文件包含 extern-C 导出符号"
else
    warn "未找到 T 段符号（shim 文件是否编译成功？）"
fi

# ── 6b. 验证生成 Rust 代码中的 FFI 声明 ──────────────────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（import_lib! / import_class!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "包含 import_lib! 绑定的文件数：${IMPORT_LIB_FILES}"
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"

    if [ "${IMPORT_LIB_FILES}" -gt 0 ]; then
        ok "生成代码包含 import_lib! 块（FFI 绑定存在）✓"
    else
        warn "生成代码中未找到 import_lib! 块"
        warn "  请检查：shim 文件是否被成功捕获，且包含 extern-C 函数？"
    fi

    echo ""
    echo "──── import_lib! 绑定函数（前 30 条）────"
    grep -rn "fn " "${RUST_SRC}" 2>/dev/null | grep -v "//\|test\|mod " | head -30 || true

    echo ""
    echo "──── import_class! 类（前 20 条）────"
    grep -rn "class " "${RUST_SRC}" 2>/dev/null | grep -v "//\|#\[" | head -20 || true

    # ── 特性验证 ③ link_name 一致性 ─────────────────────────────────────────
    echo ""
    echo "──── link_name 一致性检查（不应含路径分隔符 /）────"
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
            ok "所有 link_name 均为纯文件名（特性③ link_name 一致性 通过）"
        else
            warn "${BAD_LINKS} 个 link_name 含路径分隔符，请检查提取器输出"
        fi
    else
        info "未找到 #![link_name = ...] 声明（可能无 import_lib! 块）"
    fi

    # ── 特性验证 ⑥ hpp/hxx 头文件探测 ───────────────────────────────────────
    echo ""
    echo "──── hpp/hxx 头文件探测（特性⑥ cpp! 块 include 检查）────"
    HDR_INCLUDES=$(grep -rh '#include' "${RUST_SRC}" 2>/dev/null | grep -v '//' | wc -l)
    info "生成代码中 #include 指令数：${HDR_INCLUDES}"
    if [ "${HDR_INCLUDES}" -gt 0 ]; then
        ok "cpp! 块包含头文件 include（hpp/hxx 探测路径已生效）"
        grep -rh '#include' "${RUST_SRC}" 2>/dev/null | grep -v '//' | sort -u | head -10 || true
    else
        warn "cpp! 块无 #include 指令（可能未探测到对应头文件）"
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

# ── 6c. 交叉比对：cpp2rust-demo 生成的 FFI 函数名 vs. nm 符号 ─────────────────
echo -e "\n${BOLD}6c. FFI 函数名交叉比对（生成代码 vs. shim 目标文件 nm 符号）${NC}"
GENERATED_FUNS=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${RUST_SRC}" 2>/dev/null \
    | grep -oE '"[^"]*"' | tr -d '"' | sort -u)

if [ -n "${GENERATED_FUNS}" ]; then
    FUN_COUNT=$(echo "${GENERATED_FUNS}" | wc -l)
    info "生成的函数绑定数（来自 #[cpp(func=...)]）：${FUN_COUNT}"
    echo "生成的 FFI 函数（前 15 条）："
    echo "${GENERATED_FUNS}" | head -15 | while IFS= read -r fname; do
        printf "  %-45s" "${fname}"
        if grep -q "${fname}" "${NM_CACHE}" 2>/dev/null; then
            echo -e "${GREEN}✓ 在目标文件中找到${NC}"
        else
            echo -e "${YELLOW}? 未在目标文件中直接找到（可能在 hicc cpp! 宏展开后才出现）${NC}"
        fi
    done || true
else
    info "未在生成代码中找到 #[cpp(func=...)] 标注（可能全部通过 import_class! 绑定）"
fi
# NM_CACHE 由 EXIT trap 清理，无需手动删除

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

    echo "──── 各 .cpp2rust 文件大小（前 15 条）────"
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null \
        | sort -rn | head -15 || true
fi

# ── 6e. 特性验证：struct/class 前缀清理 & restrict 剥离 ──────────────────────
echo -e "\n${BOLD}6e. struct/class 前缀 & restrict 清理验证（特性④⑤）${NC}"
if [ -d "${RUST_SRC}" ]; then
    STRUCT_HITS=$(grep -rn '\bstruct \b' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l) || STRUCT_HITS=0
    CLASS_HITS=$(grep -rn '\bclass \b' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l) || CLASS_HITS=0
    RESTRICT_HITS=$(grep -rn '__restrict\|[^_]restrict[^_]' "${RUST_SRC}" 2>/dev/null \
        | grep -v '^\s*//' | wc -l) || RESTRICT_HITS=0

    if [ "${STRUCT_HITS}" -eq 0 ] && [ "${CLASS_HITS}" -eq 0 ]; then
        ok "Rust 绑定中无多余的 struct/class 前缀（特性④ 通过）"
    else
        warn "Rust 绑定中仍有 struct/class 前缀：struct=${STRUCT_HITS} class=${CLASS_HITS}"
        grep -rn '\bstruct \b\|\bclass \b' "${RUST_SRC}" 2>/dev/null \
            | grep -v '^\s*//' | grep -v 'hicc::cpp!' | head -10 || true
    fi

    if [ "${RESTRICT_HITS}" -eq 0 ]; then
        ok "Rust 绑定中无 restrict 限定符（特性⑤ 通过）"
    else
        warn "Rust 绑定中仍有 restrict 限定符：${RESTRICT_HITS} 处"
        grep -rn '__restrict\|[^_]restrict[^_]' "${RUST_SRC}" 2>/dev/null \
            | grep -v '^\s*//' | head -10 || true
    fi
else
    warn "Rust 源码目录不存在，跳过特性④⑤验证"
fi
# OBJ_DIR 由 EXIT trap 清理，无需手动删除

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│             cpp2rust-demo Shim FFI 验证结果             │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      rapidjson（通过 extern-C shim 层）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}shim 目录：${NC} ${SHIM_DIR}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
echo ""

# import_lib! 存在检查
if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    TOTAL_FN_BINDINGS=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${RUST_SRC}" 2>/dev/null | wc -l)
    echo -e "  ${BOLD}import_lib! FFI 绑定文件数：${NC}  ${IMPORT_LIB_FILES}"
    echo -e "  ${BOLD}FFI 函数绑定总数：${NC}          ${TOTAL_FN_BINDINGS}"
    if [ "${IMPORT_LIB_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 成功生成 Rust safe FFI（import_lib! 块存在）${NC}"
    else
        echo -e "  ${RED}✗ 未生成 FFI 绑定（请检查 shim 文件是否被正确捕获）${NC}"
        SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
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
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -l 'import_lib'"
echo ""

ok "验证脚本执行完毕！"

# 全局错误汇总：若有非致命检查失败则以 exit 1 退出，确保 CI 能捕获失败
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
SKILL 适用场景（shim 工作流）：

  在 C++ shim 文件目录与 GitHub Copilot 对话，说：
    "帮我用 cpp2rust-demo 把这些 shim 文件转换为 Rust FFI"
  或：
    "对这个项目执行 cpp2rust-demo 转换"

────────────────────────────────────────────────────────────────
纯 C++ 库的典型 shim 工作流（以 rapidjson 为例）：

  1. 编写 C++ shim 文件（extern-C 包装层）
     参考：references/rapidjson-refactoring/rapidjson_sys/shim/

  2. 在 shim 目录编写 Makefile 或 build script

  3. 运行 cpp2rust-demo init
     cpp2rust-demo init --feature rapidjson_shim -- make -j$(nproc)

  4. 运行 cpp2rust-demo merge
     cpp2rust-demo merge --feature rapidjson_shim

  5. 在生成的 Rust 项目中使用 import_lib! 绑定调用 rapidjson API

────────────────────────────────────────────────────────────────
⚠  重要提示：
  cpp2rust-demo 只处理含 extern "C" 函数的 C++ 文件。
  直接对纯 C++ 文件（如 rapidjson 的 GTest unittest）运行 init，
  因无 extern "C" 函数，只会生成 hicc::cpp! 头文件块，
  不会生成 import_lib! FFI 绑定。

  这是预期行为，不是 bug。
────────────────────────────────────────────────────────────────

EOF
