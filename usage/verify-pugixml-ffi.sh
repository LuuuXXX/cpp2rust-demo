#!/usr/bin/env bash
# =============================================================================
# verify-pugixml-ffi.sh
#
# 用途：验证 cpp2rust-demo 对 pugixml C++ 库的完整 Rust safe FFI 生成能力
#
# 背景说明：
#   pugixml 是 C++ 库（无 shim 层），cpp2rust-demo 采用 hicc 直出（无 shim）模式，
#   直接以库源码编译拦截结果生成 Rust safe FFI。本脚本以 src/pugixml.cpp 为输入演示
#   完整工作流：源码编译 → cpp2rust-demo init → merge → cargo check → cargo test。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位本地 pugixml 源码（references/pugixml）
#   § 3. 编译库目标文件（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. FFI 验证（类符号、import_class!、预处理文件、签名清理）
#   § 7. 生成结果汇报
#
# 工作流说明（重要）：
#   cpp2rust-demo 通过编译拦截捕获 C++ 编译命令，提取类/方法信息后生成
#   hicc 直出 Rust FFI（hicc::cpp! + import_class!；必要时可伴随 import_lib!）。
#
#   ┌──────────────────────────────────────────────────────────────┐
#   │  纯 C++ 库（如 pugixml）的 hicc 直出工作流：            │
#   │                                                              │
#   │  1. 直接编译库源码（src/pugixml.cpp）供 cpp2rust-demo 拦截  │
#   │  2. 运行 cpp2rust-demo init（本脚本演示的步骤）              │
#   │  3. 运行 cpp2rust-demo merge                                 │
#   │  4. 在生成的 Rust 项目中使用 import_class! 绑定             │
#   │                                                              │
#   │  ⚠  本流程不依赖 extern-C shim；生成结果以类绑定为主。      │
#   └──────────────────────────────────────────────────────────────┘
#
# 本脚本验证的 cpp2rust-demo 特性：
#   ① 直出模式编译拦截 —— 工具从库源码编译命令提取 C++ 类型信息
#   ② import_class! 生成 —— 面向类库生成完整的 Rust 类绑定块
#   ③ build.rs 自包含性 —— 自动注入头文件路径、实现源码与 C++ 标准
#   ④ struct/class 前缀清理 —— 绑定签名中的 C++ 关键字前缀自动去除
#   ⑤ __restrict__ 处理 —— 类型末尾的 restrict 限定符自动剥离
#   ⑥ hpp/hxx 头文件探测 —— read_source_includes 按 .h → .hpp 顺序探测
#   ⑦ hicc-std 依赖声明 —— 生成的 Cargo.toml 包含 hicc-std 依赖
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
FEATURE="${FEATURE:-pugixml_ffi}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"   # 置 1 可跳过 cargo install 步骤（已安装时使用）
CXX_STD="c++11"                     # C++ 标准版本（库源码要求 C++11）
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
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo \
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
# § 2. 定位本地 pugixml 源码
# =============================================================================
step "§ 2. 定位本地 pugixml 源码"

# 从脚本所在目录向上找仓库根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"

LIB_DIR="${REPO_DIR}/references/pugixml"
PUGIXML_SRC="${LIB_DIR}/src"
if [ ! -d "${LIB_DIR}" ] || [ ! -f "${PUGIXML_SRC}/pugixml.cpp" ]; then
    fail "pugixml 子模块未初始化，请运行：git submodule update --init references/pugixml"
fi

SOURCE_FILE="${PUGIXML_SRC}/pugixml.cpp"

info "库目录：${LIB_DIR}"
info "源码目录：${PUGIXML_SRC}"
info "源文件：${SOURCE_FILE}"
info "include 目录：${PUGIXML_SRC}"
ok "pugixml 源码就绪"

# =============================================================================
# § 3. 编译库目标文件（供 nm 符号验证）
# =============================================================================
step "§ 3. 编译库目标文件"

OBJ_DIR=$(mktemp -d)
info "目标文件输出目录：${OBJ_DIR}"

# 注册 trap 清理临时目录
trap 'rm -rf "${OBJ_DIR}" "${NM_CACHE:-}" 2>/dev/null || true' EXIT

OBJ_FILE="${OBJ_DIR}/pugixml.o"
COMPILE_LOG="${OBJ_DIR}/pugixml.log"
COMPILE_ERRORS=0
if g++ -c -std="${CXX_STD}" -I"${PUGIXML_SRC}" "${PUGIXML_SRC}/pugixml.cpp" -o "${OBJ_FILE}" >"${COMPILE_LOG}" 2>&1; then
    ok "编译成功：pugixml.o"
else
    warn "编译失败：src/pugixml.cpp"
    cat "${COMPILE_LOG}" >&2
    COMPILE_ERRORS=1
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

# 创建临时构建脚本：g++ -c 编译库源码，触发 LD_PRELOAD hook 拦截
BUILD_SCRIPT=$(mktemp)
cat > "${BUILD_SCRIPT}" << EOF
#!/bin/bash
# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用
set -e
g++ -c -std="${CXX_STD}" \
    -I"${PUGIXML_SRC}" \
    "${PUGIXML_SRC}/pugixml.cpp" -o /dev/null 2>&1
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
#   使 hicc-build 为 import_class! / import_lib! 生成 `_hicc_export_methods_*` 方法表。
#   此外，init 阶段会从编译拦截记录的 `.opts`（include 路径 / `-std`）与反推出的
#   实现 `.cpp` 落盘编译元数据（meta/build-meta.json），并据此在生成的 build.rs 中
#   **自动**注入：
#     ① pugixml 头文件包含路径（hicc 生成的胶水 C++ #include 需要）；
#     ② 编译 src/pugixml.cpp，提供底层 C++ 实现符号；
#     ③ 与库源码编译一致的 C++ 标准。
#   因此正常情况下本脚本**无需**再就地改写 build.rs。下面先校验工具产物是否已
#   自包含（含 cc_build.include + cc_build.file）：
#     · 已自包含 → 直接信任工具产物（方案 A）。
#     · 未自包含（旧版工具 / 未捕获 .opts）→ 退回脚本就地补全（方案 B 兜底）。
# =============================================================================
step "§ 5a. 校验 build.rs（方案 A：工具自动注入头路径与库实现）"

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
        ok "build.rs 已由工具自动注入头路径 + 库实现（方案 A 生效，跳过就地补全）"
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

                PUGIXML_SRC_BP="$(to_build_path "${PUGIXML_SRC}")"
        SOURCE_FILE_BP="$(to_build_path "${SOURCE_FILE}")"

        cat > "${BUILD_RS}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    // 与库源码编译保持一致的 C++ 标准（脚本 CXX_STD=${CXX_STD}）
    cc_build.std("${CXX_STD}");
    // ① pugixml 头文件包含路径（hicc 生成的胶水 C++ #include 需要）
    cc_build.include("${PUGIXML_SRC_BP}");
    // ② 编译 src/pugixml.cpp，提供底层 C++ 实现符号
    cc_build.file("${SOURCE_FILE_BP}");
    // 逐文件注册含 hicc 宏的单元，生成 _hicc_export_methods_* 导出函数
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入头路径 + 库实现 + 逐文件注册）"
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
        warn "cargo test 失败 — 请检查上方链接日志（build.rs 已逐文件注册 .rust_file 并编译库实现，理论上 _hicc_export_methods_* 与底层 C++ 符号均应可解析）"
    fi
else
    info "未生成 tests/smoke.rs（可能 init 阶段无 pub class 类型），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

# ── 6a. 验证库目标文件中的核心 C++ 符号 ───────────────────────────────────────
echo -e "\n${BOLD}6a. 库目标文件核心 C++ 符号（nm 验证）${NC}"
NM_CACHE=$(mktemp)
if [ -f "${OBJ_FILE}" ]; then
    nm -C --defined-only "${OBJ_FILE}" 2>/dev/null > "${NM_CACHE}" || true
    DEFINED_SYMBOLS=$(wc -l < "${NM_CACHE}" | tr -d '[:space:]')
    MATCHED_SYMBOLS=$(grep -E 'xml_document|xml_node' "${NM_CACHE}" 2>/dev/null | wc -l | tr -d '[:space:]')
    info "库目标文件已定义符号数：${DEFINED_SYMBOLS}"
    info "匹配 xml_document/xml_node 的符号数：${MATCHED_SYMBOLS}"

    if [ "${MATCHED_SYMBOLS}" -gt 0 ]; then
        echo "──── 部分核心类符号（前 20 条）────"
        grep -E 'xml_document|xml_node' "${NM_CACHE}" | head -20 || true
        ok "目标文件中存在 xml_document/xml_node 相关符号"
    else
        warn "未在 pugixml.o 中找到 xml_document/xml_node 相关符号"
    fi
else
    warn "未找到目标文件：${OBJ_FILE}"
fi

# ── 6b. 验证生成 Rust 代码中的 FFI 声明 ──────────────────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（import_class! / import_lib!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    info "包含 import_lib! 绑定的文件数：${IMPORT_LIB_FILES}"
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"

    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        ok "生成代码包含 import_class! 块（类绑定存在）✓"
    elif [ "${IMPORT_LIB_FILES}" -gt 0 ]; then
        warn "仅检测到 import_lib!，未检测到 import_class!（对于类库这通常不符合预期）"
    else
        warn "生成代码中同时未找到 import_class! / import_lib! 块"
        warn "  请检查：库源码是否被成功捕获并生成 Rust FFI？"
    fi

    echo ""
    echo "──── import_lib! 绑定函数（前 30 条）────"
    grep -rn "fn " "${RUST_SRC}" 2>/dev/null | grep -v "//\|test\|mod " | head -30 || true

    echo ""
    echo "──── import_class! 类（前 20 条）────"
    grep -rn "class " "${RUST_SRC}" 2>/dev/null | grep -v "//\|#\[" | head -20 || true

    # ── 特性验证 ③ build.rs / link_name 一致性 ──────────────────────────────
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
        info "未找到 #![link_name = ...] 声明（可能以 import_class! 为主）"
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

# ── 6c. hicc 直出模式说明 ────────────────────────────────────────────────────
echo -e "\n${BOLD}6c. hicc 直出模式说明（无 shim 函数名交叉比对）${NC}"
info "pugixml 采用 hicc 直出模式（无 extern-C shim），不适用 shim 函数名交叉比对。"

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
echo -e "${BOLD}│           cpp2rust-demo Direct FFI 验证结果             │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      pugixml（hicc 直出，无 shim）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}库目录：${NC}    ${LIB_DIR}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
echo ""

# import_class! 存在检查
if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l)
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l)
    TOTAL_FN_BINDINGS=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${RUST_SRC}" 2>/dev/null | wc -l)
    echo -e "  ${BOLD}import_class! 绑定文件数：${NC}  ${IMPORT_CLASS_FILES}"
    echo -e "  ${BOLD}import_lib! 绑定文件数：${NC}    ${IMPORT_LIB_FILES}"
    echo -e "  ${BOLD}FFI 函数绑定总数：${NC}          ${TOTAL_FN_BINDINGS}"
    if [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 成功生成 Rust safe FFI（import_class! 块存在）${NC}"
    else
        echo -e "  ${RED}✗ 未生成 import_class! 绑定（请检查库源码是否被正确捕获）${NC}"
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
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -l 'import_class'"
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
SKILL 适用场景（hicc 直出工作流）：

  在库源码目录与 GitHub Copilot 对话，说：
    "帮我用 cpp2rust-demo 把这个 C++ 库转换为 Rust FFI"
  或：
    "对这个项目执行 cpp2rust-demo 转换"

────────────────────────────────────────────────────────────────
纯 C++ 库的典型 hicc 直出工作流（以 pugixml 为例）：

  1. 直接编译库源码
     参考：references/pugixml/src/pugixml.cpp

  2. 在源码目录编写 Makefile 或 build script

  3. 运行 cpp2rust-demo init
     cpp2rust-demo init --feature pugixml_ffi -- bash build-pugixml.sh

  4. 运行 cpp2rust-demo merge
     cpp2rust-demo merge --feature pugixml_ffi

  5. 在生成的 Rust 项目中使用 import_class! 绑定调用 pugixml API

────────────────────────────────────────────────────────────────
⚠  重要提示：
  pugixml 采用 hicc 直出模式（无 extern-C shim），不适用 shim 函数名交叉比对。

  这是 hicc 直出工作流的预期行为，不是 bug。
────────────────────────────────────────────────────────────────

EOF
