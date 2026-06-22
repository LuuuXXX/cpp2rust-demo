#!/usr/bin/env bash
# =============================================================================
# verify-fmtlib-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 {fmt} 库的完整 Rust safe FFI 生成能力
#
# 背景说明：
#   {fmt} 是多源文件 C++ 格式化库，核心实现位于 src/format.cc 与 src/os.cc。
#   本脚本验证 cpp2rust-demo 对「多翻译单元 + .cc 扩展名 + 无 shim 直接抽取」
#   工作流的支持能力：编译源文件 → cpp2rust-demo init → merge → cargo check/test →
#   检查生成的 Rust FFI 脚手架与 build.rs 注入是否完整。
#
# 流程总览：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 检查 fmtlib 子模块
#   § 3. 编译 fmtlib .cc 目标文件（供 nm 符号验证）
#   § 4. cpp2rust-demo init —— 编译拦截 & Rust FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录
#   § 5a. 校验 / 兜底补全 build.rs
#   § 5b. cargo check
#   § 5c. cargo test
#   § 6. FFI 验证（fmt:: 符号、import_lib! / import_class!、预处理体积）
#   § 7. 生成结果汇报
# =============================================================================

set -euo pipefail

FEATURE="${FEATURE:-fmtlib_ffi}"
SKIP_INSTALL="${SKIP_INSTALL:-0}"
CXX_STD="c++17"

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

SCRIPT_ERRORS=0

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null || echo "${SCRIPT_DIR}/..")"
REPO_DIR="$(cd "${REPO_DIR}" && pwd)"
RUNTIME_ROOT="${REPO_DIR}/.cpp2rust-script-work"
RUN_ID="${FEATURE}-$(date +%s)-$$"
RUNTIME_DIR="${RUNTIME_ROOT}/${RUN_ID}"
mkdir -p "${RUNTIME_DIR}"
trap 'rm -rf "${RUNTIME_DIR}" 2>/dev/null || true' EXIT

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
    INSTALL_LOG="${RUNTIME_DIR}/cargo-install.log"
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo \
             >"${INSTALL_LOG}" 2>&1; then
        echo "── cargo install 失败，完整日志：──"
        cat "${INSTALL_LOG}"
        fail "cpp2rust-demo 安装失败，请检查上方日志"
    fi
    tail -5 "${INSTALL_LOG}" || true
    ok "cpp2rust-demo 安装完成：$(which cpp2rust-demo)"
fi

cpp2rust-demo --version 2>/dev/null || true

# =============================================================================
# § 2. 检查 fmtlib 子模块
# =============================================================================
step "§ 2. 检查 fmtlib 子模块"

FMT_DIR="${REPO_DIR}/references/fmtlib"
FMT_INCLUDE="${FMT_DIR}/include"
FMT_SRC="${FMT_DIR}/src"
if [ ! -d "${FMT_DIR}" ] || [ ! -f "${FMT_SRC}/format.cc" ]; then
    fail "fmtlib 子模块未初始化，请运行：git submodule update --init references/fmtlib"
fi
FMT_SOURCES=("${FMT_SRC}/format.cc" "${FMT_SRC}/os.cc")

info "fmtlib 根目录：${FMT_DIR}"
info "include 目录：${FMT_INCLUDE}"
info "src 目录：${FMT_SRC}"
printf '  - %s\n' "${FMT_SOURCES[@]}"
ok "fmtlib 子模块检查通过"

# =============================================================================
# § 3. 编译 fmtlib .cc 目标文件（供 nm 符号验证）
# =============================================================================
step "§ 3. 编译 fmtlib 目标文件"

OBJ_DIR="${RUNTIME_DIR}/obj"
mkdir -p "${OBJ_DIR}"
info "目标文件输出目录：${OBJ_DIR}"

COMPILE_ERRORS=0
for src in "${FMT_SOURCES[@]}"; do
    name="$(basename "${src}" .cc)"
    g++ -c -std="${CXX_STD}" -I"${FMT_INCLUDE}" "${src}" \
        -o "${OBJ_DIR}/${name}.o" 2>"${OBJ_DIR}/${name}.log" || {
        warn "编译失败：${src}"
        cat "${OBJ_DIR}/${name}.log" >&2
        COMPILE_ERRORS=$((COMPILE_ERRORS+1))
    }
done

if [ "${COMPILE_ERRORS}" -gt 0 ]; then
    warn "${COMPILE_ERRORS} 个 fmtlib 源文件编译失败，后续 nm 验证可能不完整"
else
    ok "全部 ${#FMT_SOURCES[@]} 个 fmtlib 源文件编译成功"
fi

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

BUILD_SCRIPT="${RUNTIME_DIR}/build-fmtlib.sh"
cat > "${BUILD_SCRIPT}" <<EOFSCRIPT
#!/bin/bash
set -e
for src in "${FMT_SRC}/format.cc" "${FMT_SRC}/os.cc"; do
    g++ -c -std="${CXX_STD}" \
        -I"${FMT_INCLUDE}" \
        "\${src}" -o /dev/null 2>&1
done
EOFSCRIPT
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"
info "构建命令：bash ${BUILD_SCRIPT}"

cd "${REPO_DIR}"
cpp2rust-demo init \
    --feature "${FEATURE}" \
    -- bash "${BUILD_SCRIPT}"

ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${REPO_DIR}/.cpp2rust/${FEATURE}"
info "输出目录：${CPP2RUST_OUTPUT}"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l | tr -d '[:space:]')
CAPTURED="${CAPTURED:-0}"
info "捕获预处理文件数：${CAPTURED}"

# =============================================================================
# § 5. cpp2rust-demo merge — 整理输出目录
# =============================================================================
step "§ 5. cpp2rust-demo merge（整理输出结构）"

cd "${REPO_DIR}"
cpp2rust-demo merge --feature "${FEATURE}"
ok "merge 完成"

RUST_SRC="${CPP2RUST_OUTPUT}/rust/src"
RS_FILES=$(find -L "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l | tr -d '[:space:]')
RS_FILES="${RS_FILES:-0}"
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
step "§ 5a. 校验 build.rs（方案 A：工具自动注入 include + .cc 实现）"

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
BUILD_RS="${RUST_PROJECT}/build.rs"
LIB_NAME="${FEATURE//-/_}"

if [ -f "${BUILD_RS}" ]; then
    info "工具生成的 build.rs："
    sed 's/^/    /' "${BUILD_RS}"

    BUILD_META="${CPP2RUST_OUTPUT}/meta/build-meta.json"
    if [ -f "${BUILD_META}" ]; then
        info "编译元数据 meta/build-meta.json："
        sed 's/^/    /' "${BUILD_META}"
    fi

    if grep -q 'cc_build.include' "${BUILD_RS}" && grep -q 'cc_build.file' "${BUILD_RS}"; then
        ok "build.rs 已由工具自动注入头路径 + fmtlib .cc 实现（方案 A 生效）"
    else
        warn "工具 build.rs 未同时注入 include/file，退回脚本就地补全（方案 B 兜底）"

        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" ! -name "lib.rs" ! -name "mod.rs" | sort)

        FMT_INCLUDE_BP="$(to_build_path "${FMT_INCLUDE}")"
        FMT_SRC_BP="$(to_build_path "${FMT_SRC}")"

        cat > "${BUILD_RS}" <<EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.std("${CXX_STD}");
    cc_build.include("${FMT_INCLUDE_BP}");
    cc_build.file("${FMT_SRC_BP}/format.cc");
    cc_build.file("${FMT_SRC_BP}/os.cc");
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入 fmt include + .cc 文件 + 逐文件注册）"
    fi
else
    warn "未找到 ${BUILD_RS}，跳过 build.rs 校验/补全"
fi

# =============================================================================
# § 5b. cargo check
# =============================================================================
step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"

if [ -f "${RUST_PROJECT}/Cargo.toml" ]; then
    info "在 ${RUST_PROJECT} 中运行 cargo check ..."
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
        fail "cargo check 失败 — 生成的 FFI 代码存在编译错误，请检查上方输出"
    fi
else
    warn "未找到 ${RUST_PROJECT}/Cargo.toml，跳过 cargo check"
fi

# =============================================================================
# § 5c. cargo test
# =============================================================================
step "§ 5c. cargo test（验证生成的冒烟测试可编译、可链接、可运行）"

SMOKE_FILE="${RUST_PROJECT}/tests/smoke.rs"
if [ -f "${SMOKE_FILE}" ]; then
    info "检测到冒烟测试文件：${SMOKE_FILE}"
    if (cd "${RUST_PROJECT}" && cargo test 2>&1); then
        ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
    else
        warn "cargo test 失败 — 请检查上方日志（fmtlib 为多 .cc 直连场景）"
    fi
else
    info "未生成 tests/smoke.rs（可能无 pub class 类型），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

echo -e "\n${BOLD}6a. fmtlib 目标文件符号（nm 验证）${NC}"
NM_CACHE="${RUNTIME_DIR}/fmtlib-nm.txt"
find "${OBJ_DIR}" -name "*.o" 2>/dev/null \
    | xargs -r nm -C --defined-only -f posix 2>/dev/null > "${NM_CACHE}" || true

FMT_SYMBOL_COUNT=$(grep -c 'fmt::' "${NM_CACHE}" 2>/dev/null || echo 0)
info "nm 输出中匹配 fmt:: 的符号数：${FMT_SYMBOL_COUNT}"
if [ "${FMT_SYMBOL_COUNT}" -gt 0 ]; then
    echo "──── 部分 fmt:: 符号（前 20 条）────"
    grep 'fmt::' "${NM_CACHE}" | awk '{print $1}' | sort -u | head -20 || true
    ok "fmtlib 目标文件中检测到 fmt:: 符号"
else
    warn "未在目标文件中检测到 fmt:: 符号（请检查 .cc 编译结果）"
fi

echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（import_lib! / import_class!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    IMPORT_LIB_FILES="${IMPORT_LIB_FILES:-0}"
    IMPORT_CLASS_FILES="${IMPORT_CLASS_FILES:-0}"
    info "包含 import_lib! 绑定的文件数：${IMPORT_LIB_FILES}"
    info "包含 import_class! 绑定的文件数：${IMPORT_CLASS_FILES}"

    echo "──── import_lib! / import_class! 命中（前 20 条）────"
    grep -rn 'hicc::import_lib!\|hicc::import_class!' "${RUST_SRC}" 2>/dev/null | head -20 || true

    if [ "${IMPORT_LIB_FILES}" -eq 0 ] && [ "${IMPORT_CLASS_FILES}" -eq 0 ]; then
        warn "生成代码中既未找到 import_lib! 也未找到 import_class!"
        SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

echo -e "\n${BOLD}6c. shim 交叉比对说明${NC}"
info "fmtlib 为 hicc-direct 多源文件库，本脚本不做 shim 符号交叉比对（6c 不适用）"

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
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null | sort -rn | head -15 || true
else
    warn "未找到预处理目录：${C_DIR}"
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│            cpp2rust-demo fmtlib FFI 验证结果            │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      fmtlib（多源文件 + .cc 扩展名）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}fmt 目录：${NC}   ${FMT_DIR}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"

if [ -d "${RUST_SRC}" ]; then
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || IMPORT_LIB_FILES=0
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || IMPORT_CLASS_FILES=0
    IMPORT_LIB_FILES="${IMPORT_LIB_FILES:-0}"
    IMPORT_CLASS_FILES="${IMPORT_CLASS_FILES:-0}"
    echo -e "  ${BOLD}import_lib! 文件数：${NC}     ${IMPORT_LIB_FILES}"
    echo -e "  ${BOLD}import_class! 文件数：${NC}   ${IMPORT_CLASS_FILES}"
    if [ "${IMPORT_LIB_FILES}" -gt 0 ] || [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        echo -e "  ${GREEN}✓ 已生成 fmtlib Rust safe FFI 脚手架${NC}"
    else
        echo -e "  ${RED}✗ 未检测到 import_lib! / import_class! 绑定${NC}"
        SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
    fi
fi

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
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -n 'import_lib\|import_class'"
echo ""

ok "验证脚本执行完毕！"

if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
fi
