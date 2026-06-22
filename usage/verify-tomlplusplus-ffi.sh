#!/usr/bin/env bash
# =============================================================================
# verify-tomlplusplus-ffi.sh
#
# 用途：验证 cpp2rust-demo 对 toml++ 库的完整 Rust safe FFI 生成能力
#       （header-only、大型单头、重度模板）
# =============================================================================
set -euo pipefail

FEATURE="${FEATURE:-tomlplusplus_ffi}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"
CXX_STD="c++17"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ OK ]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }
step()  { echo -e "\n${BOLD}══════════════════════════════════════════${NC}";           echo -e "${BOLD}  $*${NC}";           echo -e "${BOLD}══════════════════════════════════════════${NC}"; }

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
VERIFY_ROOT="${REPO_DIR}/.cpp2rust-verify"
RUN_DIR="${VERIFY_ROOT}/${FEATURE}-$$"
OBJ_DIR="${RUN_DIR}/obj"
mkdir -p "${OBJ_DIR}"
trap 'rm -rf "${RUN_DIR}" 2>/dev/null || true' EXIT

TOML_DIR="${REPO_DIR}/references/tomlplusplus"
TOML_INCLUDE="${TOML_DIR}/include"
DRIVER_CPP="${RUN_DIR}/toml_driver.cpp"
DRIVER_OBJ="${OBJ_DIR}/toml_driver.o"
BUILD_SCRIPT="${RUN_DIR}/build-tomlplusplus.sh"

# =============================================================================
# § 0. 环境检查
# =============================================================================
step "§ 0. 环境检查"

need_cmd git
need_cmd g++
need_cmd cargo
need_cmd nm

if ! pkg-config --exists libclang 2>/dev/null &&    ! find /usr /lib /usr/local -name "libclang*.so*" 2>/dev/null | grep -q .; then
    warn "未检测到 libclang（可能未安装 libclang-dev）。cpp2rust-demo 依赖 libclang 解析 AST。"
    warn "  Ubuntu/Debian：sudo apt-get install -y clang libclang-dev"
fi

ok "环境检查完成"

# =============================================================================
# § 1. 安装 cpp2rust-demo
# =============================================================================
step "§ 1. 安装 cpp2rust-demo"

INSTALL_LOG="${RUN_DIR}/cargo-install.log"
if [ "${SKIP_INSTALL}" = "1" ] && command -v cpp2rust-demo &>/dev/null; then
    ok "已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）"
else
    info "从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…"
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked --bin cpp2rust-demo              >"${INSTALL_LOG}" 2>&1; then
        echo "── cargo install 失败，完整日志：──"
        cat "${INSTALL_LOG}"
        fail "cpp2rust-demo 安装失败，请检查上方日志"
    fi
    tail -5 "${INSTALL_LOG}" || true
    ok "cpp2rust-demo 安装完成：$(which cpp2rust-demo)"
fi

cpp2rust-demo --version 2>/dev/null || true

# =============================================================================
# § 2. 检查 toml++ 子模块
# =============================================================================
step "§ 2. 检查 toml++ 子模块"

if [ ! -f "${TOML_INCLUDE}/toml++/toml.hpp" ]; then
    fail "未找到 toml.hpp：${TOML_INCLUDE}/toml++/toml.hpp
请先运行：git submodule update --init references/tomlplusplus"
fi

info "仓库根目录：${REPO_DIR}"
info "toml++ include：${TOML_INCLUDE}"
ok "toml++ 子模块就绪"

# =============================================================================
# § 3. 创建驱动文件并编译
# =============================================================================
step "§ 3. 创建驱动文件并编译"

cat > "${DRIVER_CPP}" <<'EOF'
// toml++ 驱动文件 — 用于测试大型单头实库的解析能力
#define TOML_HEADER_ONLY 1
#include <toml++/toml.hpp>
#include <string>
namespace tomlwrap_ns {
class TomlWrapper {
public:
    int int_value(const std::string& key) const;
    std::string string_value(const std::string& key) const;
};
}  // namespace tomlwrap_ns
EOF

COMPILE_LOG="${RUN_DIR}/toml-driver.compile.log"
if g++ -c -std="${CXX_STD}" -DTOML_HEADER_ONLY=1 -I"${TOML_INCLUDE}" "${DRIVER_CPP}" -o "${DRIVER_OBJ}" >"${COMPILE_LOG}" 2>&1; then
    ok "驱动文件编译成功：${DRIVER_OBJ}"
else
    warn "驱动文件编译失败（不阻断后续 init）：${DRIVER_CPP}"
    cat "${COMPILE_LOG}" >&2 || true
fi

# =============================================================================
# § 4. cpp2rust-demo init
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 header-only FFI 脚手架）"

cat > "${BUILD_SCRIPT}" <<EOF
#!/usr/bin/env bash
set -e
g++ -c -std="${CXX_STD}" -DTOML_HEADER_ONLY=1 -I"${TOML_INCLUDE}" "${DRIVER_CPP}" -o /dev/null 2>&1
EOF
chmod +x "${BUILD_SCRIPT}"

info "工作目录：${REPO_DIR}"
info "feature 名称：${FEATURE}"
info "构建命令：bash ${BUILD_SCRIPT}"

cd "${REPO_DIR}"
cpp2rust-demo init     --feature "${FEATURE}"     -- bash "${BUILD_SCRIPT}"

ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${REPO_DIR}/.cpp2rust/${FEATURE}"
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l | tr -d '[:space:]')
info "输出目录：${CPP2RUST_OUTPUT}"
info "捕获预处理文件数：${CAPTURED:-0}"

# =============================================================================
# § 5. cpp2rust-demo merge
# =============================================================================
step "§ 5. cpp2rust-demo merge（整理输出结构）"

cd "${REPO_DIR}"
cpp2rust-demo merge --feature "${FEATURE}"
ok "merge 完成"

RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
RUST_SRC="${RUST_PROJECT}/src"
RS_FILES=$(find -L "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l | tr -d '[:space:]')
info "生成 Rust 文件数：${RS_FILES:-0}"
if [ "${RS_FILES:-0}" -gt 0 ]; then
    echo "──── 生成的 .rs 文件（前 20 条）────"
    find -L "${RUST_SRC}" -name "*.rs" | sort | head -20 || true
fi

echo ""
info "降级标记统计（cpp2rust-todo）："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn || echo "  （无降级标记）"

# =============================================================================
# § 5a. 校验 / 兜底生成 build.rs（header-only：仅注入 include）
# =============================================================================
step "§ 5a. 校验 build.rs（header-only include 注入）"

BUILD_RS="${RUST_PROJECT}/build.rs"
LIB_NAME="${FEATURE//-/_}"

if [ -f "${BUILD_RS}" ]; then
    info "工具生成的 build.rs："
    sed 's/^/    /' "${BUILD_RS}"

    if grep -q 'cc_build.include' "${BUILD_RS}"; then
        ok "build.rs 已包含头文件搜索路径（header-only 场景无需 cc_build.file）"
    else
        warn "build.rs 未注入 include 路径，退回脚本兜底生成（header-only 模式）"
        RUST_FILE_LINES=""
        while IFS= read -r rs; do
            rel="${rs#${RUST_PROJECT}/}"
            RUST_FILE_LINES="${RUST_FILE_LINES}        .rust_file(\"${rel}\")
"
        done < <(find -L "${RUST_PROJECT}/src" -name "*.rs" ! -name "lib.rs" ! -name "mod.rs" | sort)

        TOML_INCLUDE_BP="$(to_build_path "${TOML_INCLUDE}")"
        cat > "${BUILD_RS}" <<EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.std("${CXX_STD}");
    // toml++ 为 header-only；TOML_HEADER_ONLY 宏已在驱动源码中定义，此处仅补 include。
    cc_build.include("${TOML_INCLUDE_BP}");
    build
${RUST_FILE_LINES}        .compile("${LIB_NAME}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF
        info "兜底补全后的 build.rs："
        sed 's/^/    /' "${BUILD_RS}"
        ok "build.rs 已就地补全（注入 include + 逐文件注册）"
    fi
else
    warn "未找到 ${BUILD_RS}，跳过 build.rs 校验/补全"
fi

# =============================================================================
# § 5b. cargo check
# =============================================================================
step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"

if [ -f "${RUST_PROJECT}/Cargo.toml" ]; then
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
        warn "cargo test 失败 — 请检查上方日志"
    fi
else
    info "未生成 tests/smoke.rs（header-only 声明型驱动可能无冒烟测试），跳过 cargo test"
fi

# =============================================================================
# § 6. FFI 验证
# =============================================================================
step "§ 6. FFI 验证"

echo -e "
${BOLD}6a. 驱动目标文件符号（nm）${NC}"
if [ -f "${DRIVER_OBJ}" ]; then
    DRIVER_NM="${RUN_DIR}/driver.nm"
    nm --defined-only -f posix "${DRIVER_OBJ}" > "${DRIVER_NM}" 2>/dev/null || true
    DRIVER_SYMBOLS=$(wc -l < "${DRIVER_NM}" | tr -d '[:space:]')
    info "驱动目标文件定义符号数：${DRIVER_SYMBOLS:-0}"
    if [ "${DRIVER_SYMBOLS:-0}" -gt 0 ]; then
        head -20 "${DRIVER_NM}" || true
    else
        info "驱动文件仅用于触发大型单头模板解析；无导出符号属于预期"
    fi
else
    warn "未找到驱动目标文件：${DRIVER_OBJ}"
fi

echo -e "
${BOLD}6b. 生成 Rust 代码中的绑定块${NC}"
CPP_BLOCK_FILES=0
IMPORT_CLASS_FILES=0
IMPORT_LIB_FILES=0
INCLUDE_HITS=0
MACRO_HITS=0
if [ -d "${RUST_SRC}" ]; then
    CPP_BLOCK_FILES=$(grep -rl 'hicc::cpp!' "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    IMPORT_CLASS_FILES=$(grep -rl 'hicc::import_class!' "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    IMPORT_LIB_FILES=$(grep -rl 'hicc::import_lib!' "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    INCLUDE_HITS=$(grep -r '#include <toml++/toml.hpp>' "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    MACRO_HITS=$(grep -r 'TOML_HEADER_ONLY' "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]')
    info "包含 hicc::cpp! 的文件数：${CPP_BLOCK_FILES:-0}"
    info "包含 import_class! 的文件数：${IMPORT_CLASS_FILES:-0}"
    info "包含 import_lib! 的文件数：${IMPORT_LIB_FILES:-0}"
    info "toml++ 头文件 include 命中数：${INCLUDE_HITS:-0}"
    info "TOML_HEADER_ONLY 命中数：${MACRO_HITS:-0}"
    if [ "${CPP_BLOCK_FILES:-0}" -gt 0 ] && [ "${INCLUDE_HITS:-0}" -gt 0 ] && [ "${MACRO_HITS:-0}" -gt 0 ]; then
        ok "生成代码已包含 toml++ 头文件与 header-only 宏痕迹（大型单头模板解析通过）"
    else
        warn "生成代码中未找到预期的 toml++ include / TOML_HEADER_ONLY / hicc::cpp! 块"
        SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
    fi
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
    SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
fi

echo -e "
${BOLD}6c. 交叉比对${NC}"
info "toml++ 为 header-only 驱动验证场景，无 extern-C shim，本项不适用"

echo -e "
${BOLD}6d. 捕获的 .cpp2rust 预处理文件大小统计${NC}"
C_DIR="${CPP2RUST_OUTPUT}/c"
if [ -d "${C_DIR}" ]; then
    TOTAL_LINES=0
    while IFS= read -r f; do
        lines=$(wc -l < "${f}")
        TOTAL_LINES=$((TOTAL_LINES + lines))
    done < <(find "${C_DIR}" -name "*.cpp2rust" 2>/dev/null)
    info "预处理文件总行数：${TOTAL_LINES}"
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null | sort -rn | head -15 || true
fi

echo -e "
${BOLD}6e. struct/class 前缀 & restrict 清理验证${NC}"
if [ -d "${RUST_SRC}" ]; then
    STRUCT_HITS=$(grep -rn 'struct ' "${RUST_SRC}" 2>/dev/null | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l | tr -d '[:space:]') || STRUCT_HITS=0
    CLASS_HITS=$(grep -rn 'class ' "${RUST_SRC}" 2>/dev/null | grep -v '^\s*//' | grep -v 'hicc::cpp!' | wc -l | tr -d '[:space:]') || CLASS_HITS=0
    RESTRICT_HITS=$(grep -rn '__restrict\|[^_]restrict[^_]' "${RUST_SRC}" 2>/dev/null | grep -v '^\s*//' | wc -l | tr -d '[:space:]') || RESTRICT_HITS=0
    if [ "${STRUCT_HITS:-0}" -eq 0 ] && [ "${CLASS_HITS:-0}" -eq 0 ]; then
        ok "Rust 绑定中无多余的 struct/class 前缀"
    else
        warn "Rust 绑定中仍有 struct/class 前缀：struct=${STRUCT_HITS:-0} class=${CLASS_HITS:-0}"
    fi
    if [ "${RESTRICT_HITS:-0}" -eq 0 ]; then
        ok "Rust 绑定中无 restrict 限定符"
    else
        warn "Rust 绑定中仍有 restrict 限定符：${RESTRICT_HITS:-0} 处"
    fi
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│      cpp2rust-demo toml++ header-only 验证结果          │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      toml++（header-only，大型单头 + 重度模板）"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}include：${NC}   ${TOML_INCLUDE}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED:-0}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES:-0}"
echo -e "  ${BOLD}hicc::cpp! 文件数：${NC}    ${CPP_BLOCK_FILES:-0}"
echo -e "  ${BOLD}头文件 include 命中：${NC}  ${INCLUDE_HITS:-0}"
echo -e "  ${BOLD}TOML_HEADER_ONLY 命中：${NC} ${MACRO_HITS:-0}"
echo -e "  ${BOLD}import_class! 文件数：${NC} ${IMPORT_CLASS_FILES:-0}"
echo -e "  ${BOLD}import_lib! 文件数：${NC}   ${IMPORT_LIB_FILES:-0}"
if [ "${INCLUDE_HITS:-0}" -gt 0 ] && [ "${MACRO_HITS:-0}" -gt 0 ]; then
    echo -e "  ${GREEN}✓ 已验证 header-only 大型单头模板文件被成功捕获与解析${NC}"
else
    echo -e "  ${RED}✗ 未在生成代码中看到 toml++ 头文件解析结果${NC}"
    SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
fi

TODO_COUNT=$(grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || TODO_COUNT=0
TODO_COUNT="${TODO_COUNT:-0}"
if [ "${TODO_COUNT}" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ 降级标记（需手动完善）：${TODO_COUNT} 处${NC}"
else
    echo -e "  ${GREEN}✓ 无降级标记${NC}"
fi

ok "验证脚本执行完毕！"
if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
    fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方输出并修复"
fi
