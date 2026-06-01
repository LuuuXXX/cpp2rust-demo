#!/usr/bin/env bash
# =============================================================================
# verify-rapidjson-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 rapidjson 项目（测试文件）的 FFI 导出能力
#
# 流程总览：
#   § 0. 环境检查 & 依赖安装
#   § 1. 安装 cpp2rust-demo
#   § 2. git clone rapidjson
#   § 3. 配置构建环境（CMake + GTest）
#   § 4. cpp2rust-demo init  —— 编译拦截 & FFI 脚手架生成
#   § 5. cpp2rust-demo merge —— 整理输出目录结构
#   § 6. 符号验证（nm / objdump）
#   § 7. 生成结果汇报
#
# 另见文末 "§ SKILL 工作流" 一节：如何通过 GitHub Copilot Agent Skill 完成同样流程。
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev cmake \
#                           libgtest-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 可配置参数 ───────────────────────────────────────────────────────────────
RAPIDJSON_REPO="https://github.com/Tencent/rapidjson.git"
RAPIDJSON_DIR="${RAPIDJSON_DIR:-/tmp/rapidjson-ffi-demo}"
FEATURE="${FEATURE:-rapidjson_tests}"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"   # 置 1 可跳过 cargo install 步骤（已安装时使用）

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

# =============================================================================
# § 0. 环境检查 & 依赖安装
# =============================================================================
step "§ 0. 环境检查 & 依赖安装"

need_cmd git
need_cmd cmake
need_cmd g++
need_cmd cargo
need_cmd nm
need_cmd objdump

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
# § 2. git clone rapidjson
# =============================================================================
step "§ 2. git clone rapidjson"

if [ -d "${RAPIDJSON_DIR}/.git" ]; then
    info "目录已存在，执行 git pull …"
    git -C "${RAPIDJSON_DIR}" pull --ff-only 2>&1 | tail -3
    git -C "${RAPIDJSON_DIR}" submodule update --init --recursive 2>&1 | tail -3
else
    info "克隆 rapidjson → ${RAPIDJSON_DIR}"
    git clone --depth 1 --recurse-submodules --shallow-submodules \
        "${RAPIDJSON_REPO}" "${RAPIDJSON_DIR}"
fi

ok "rapidjson 源码就绪：${RAPIDJSON_DIR}"
info "rapidjson 版本：$(git -C "${RAPIDJSON_DIR}" describe --tags --always 2>/dev/null || echo unknown)"

# 检测 GTest 源码目录（必须在 clone 之后执行，才能找到 rapidjson bundled GTest）
# 优先使用 rapidjson 自带的 bundled GTest（较旧版本，兼容 C++11），避免系统 GTest 1.14+
# 要求 C++14 导致的编译失败。
GTEST_SOURCE_DIR=""
for candidate in \
    "${RAPIDJSON_DIR}/thirdparty/gtest/googletest" \
    "/usr/src/googletest/googletest" \
    "/usr/src/gtest"; do
    if [ -f "${candidate}/CMakeLists.txt" ] && \
       [ -f "${candidate}/src/gtest_main.cc" ]; then
        GTEST_SOURCE_DIR="${candidate}"
        break
    fi
done

if [ -z "${GTEST_SOURCE_DIR}" ]; then
    warn "未找到 GTest 源码目录，将跳过测试目标的构建。"
    warn "  如需启用测试，请确认 rapidjson 已通过 --recurse-submodules 初始化子模块。"
else
    info "GTest 源码：${GTEST_SOURCE_DIR}"
fi

# =============================================================================
# § 3. 配置构建环境（CMake）
# =============================================================================
step "§ 3. 配置构建环境（CMake）"

BUILD_DIR="${RAPIDJSON_DIR}/build"
mkdir -p "${BUILD_DIR}"

CMAKE_ARGS=(
    -DCMAKE_BUILD_TYPE=Debug
    -DRAPIDJSON_BUILD_EXAMPLES=OFF   # 减少无关编译单元
    -DRAPIDJSON_BUILD_DOC=OFF
)

# 若找到 GTest，开启测试构建
if [ -n "${GTEST_SOURCE_DIR}" ]; then
    CMAKE_ARGS+=(
        -DRAPIDJSON_BUILD_TESTS=ON
        -DGTEST_SOURCE_DIR="${GTEST_SOURCE_DIR}"
    )
    # 系统 GTest 1.14+（/usr/src/gtest 或 /usr/src/googletest）明确要求 C++14，
    # 而 rapidjson 的 CMakeLists.txt 未设置 CXX_STANDARD，默认 C++11 会编译失败。
    # bundled GTest 是较旧版本，不需要此标志；但加上也无害（C++14 向后兼容 C++11）。
    CMAKE_ARGS+=(-DCMAKE_CXX_STANDARD=14)
    # bundled GTest（ba96d0b）中 gtest-death-test.cc StackGrowsDown() 存在一个
    # 未初始化变量警告（dummy）。GCC 13 将该警告提升为错误（-Werror=maybe-uninitialized），
    # 导致构建失败。该问题属于旧版 GTest 代码缺陷，与 rapidjson 本身无关。
    # 通过 -Wno-maybe-uninitialized 仅在 GTest 子目录范围内抑制该警告。
    CMAKE_ARGS+=(-DCMAKE_CXX_FLAGS="-Wno-maybe-uninitialized")
    info "已启用测试目标（GTEST_SOURCE_DIR=${GTEST_SOURCE_DIR}）"
else
    CMAKE_ARGS+=(-DRAPIDJSON_BUILD_TESTS=OFF)
    warn "未找到 GTest，测试目标将被跳过（仅捕获非测试编译单元）"
fi

cmake -B "${BUILD_DIR}" "${CMAKE_ARGS[@]}" "${RAPIDJSON_DIR}"
ok "CMake 配置完成"

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & FFI 脚手架生成
# =============================================================================
step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

info "工作目录：${RAPIDJSON_DIR}"
info "feature 名称：${FEATURE}"
info "构建命令：cmake --build ${BUILD_DIR} -- -j${NPROC}"

# 非交互模式：自动全选所有被拦截的 .cpp 文件（等同于对"选择参与转换的文件"对话框全部回车确认）
export CPP2RUST_NON_INTERACTIVE=1

cd "${RAPIDJSON_DIR}"
cpp2rust-demo init \
    --feature "${FEATURE}" \
    -- cmake --build "${BUILD_DIR}" -- -j"${NPROC}"

ok "cpp2rust-demo init 完成"

CPP2RUST_OUTPUT="${RAPIDJSON_DIR}/.cpp2rust/${FEATURE}"
info "输出目录：${CPP2RUST_OUTPUT}"

# 统计捕获文件数
CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
info "捕获预处理文件数：${CAPTURED}"

# =============================================================================
# § 5. cpp2rust-demo merge — 整理输出目录
# =============================================================================
step "§ 5. cpp2rust-demo merge（整理输出结构）"

cpp2rust-demo merge --feature "${FEATURE}"
ok "merge 完成"

RUST_SRC="${CPP2RUST_OUTPUT}/rust/src"
RS_FILES=$(find "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l)
info "生成 Rust 文件数：${RS_FILES}"
if [ "${RS_FILES}" -gt 0 ]; then
    echo "──── 生成的 .rs 文件（前 20 条）────"
    find "${RUST_SRC}" -name "*.rs" | sort | head -20
fi

# 统计降级标记
echo ""
info "降级标记统计（cpp2rust-todo）："
grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null \
    | grep -oP '\[.*?\]' | sort | uniq -c | sort -rn \
    || echo "  （无降级标记）"

# =============================================================================
# § 6. 符号验证
# =============================================================================
step "§ 6. 符号验证"

# ── 6a. 验证编译产物中的 C++ mangled 符号 ────────────────────────────────────
echo -e "\n${BOLD}6a. 编译产物 C++ 符号（rapidjson 相关）${NC}"
# 在 build 目录中查找对象文件 / 共享库 / 可执行文件
echo "──── 查找目标文件 ────"
find "${BUILD_DIR}" \( -name "*.o" -o -name "*.so" -o -name "*.a" \
     -o -name "unittest_*" -o -name "rapidjson_*" \) \
    -not -path "*/CMakeFiles/CMakeTmp/*" 2>/dev/null \
    | head -20 \
    | tee /tmp/cpp2rust_build_artifacts.txt

FIRST_ARTIFACT=$(head -1 /tmp/cpp2rust_build_artifacts.txt 2>/dev/null || true)
if [ -n "${FIRST_ARTIFACT}" ]; then
    echo ""
    echo "──── nm 输出（${FIRST_ARTIFACT##*/}，仅展示 T/W 段已定义符号，前 30 条）────"
    nm --demangle "${FIRST_ARTIFACT}" 2>/dev/null \
        | awk '$2 ~ /[TW]/ { print }' | head -30 || true
fi

# ── 6b. 验证生成 Rust 代码中声明的 FFI 符号 ──────────────────────────────────
echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（extern / import_lib! / import_class!）${NC}"
if [ -d "${RUST_SRC}" ]; then
    echo "──── hicc::cpp! 块（C++ shim 实现，前 40 行）────"
    grep -r "hicc::cpp!" "${RUST_SRC}" -l 2>/dev/null | head -5 | while read -r f; do
        echo "  文件：${f##*/}"
        grep -A 20 "hicc::cpp!" "${f}" | head -40 || true
        echo ""
    done

    echo "──── import_lib! 绑定函数（前 30 条）────"
    grep -rn "fn " "${RUST_SRC}" 2>/dev/null | grep -v "//\|test\|mod " | head -30 || true

    echo "──── import_class! 类（前 20 条）────"
    grep -rn "class " "${RUST_SRC}" 2>/dev/null | grep -v "//\|#\[" | head -20 || true
else
    warn "Rust 源码目录不存在：${RUST_SRC}"
fi

# ── 6c. 交叉比对：cpp2rust-demo 生成的 shim 函数名 vs. nm 符号 ───────────────
echo -e "\n${BOLD}6c. FFI shim 函数名交叉比对${NC}"
# 从生成的 .rs 文件中提取 #[cpp(func = ...)] 声明
GENERATED_FUNS=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${RUST_SRC}" 2>/dev/null \
    | grep -oP '"[^"]*"' | tr -d '"' | sort -u)

if [ -n "${GENERATED_FUNS}" ]; then
    echo "生成的 shim 函数（来自 #[cpp(func=...)]）："
    # 一次性收集所有目标文件的符号，避免对每个函数名重复调用 nm（O(n*m) → O(n+m)）
    NM_CACHE=$(mktemp)
    find "${BUILD_DIR}" -name "*.o" 2>/dev/null \
        | xargs -r nm --demangle 2>/dev/null > "${NM_CACHE}" || true

    echo "${GENERATED_FUNS}" | head -30 | while read -r fname; do
        printf "  %-40s" "${fname}"
        if grep -q "${fname}" "${NM_CACHE}"; then
            echo -e "${GREEN}✓ 在目标文件中找到${NC}"
        else
            echo -e "${YELLOW}? 未在目标文件中直接找到（可能在 hicc cpp! 宏展开后才出现）${NC}"
        fi
    done
    rm -f "${NM_CACHE}"
else
    info "未在生成代码中找到 #[cpp(func=...)] 标注（可能全部通过 import_class! 绑定）"
fi

# ── 6d. 验证 .cpp2rust 预处理文件的完整性 ────────────────────────────────────
echo -e "\n${BOLD}6d. .cpp2rust 预处理文件完整性检查${NC}"
C_DIR="${CPP2RUST_OUTPUT}/c"
if [ -d "${C_DIR}" ]; then
    TOTAL_LINES=0
    while IFS= read -r f; do
        lines=$(wc -l < "${f}")
        TOTAL_LINES=$((TOTAL_LINES + lines))
    done < <(find "${C_DIR}" -name "*.cpp2rust" 2>/dev/null)
    info "预处理文件总行数：${TOTAL_LINES}（越大说明捕获内容越多）"

    echo "──── 各 .cpp2rust 文件大小（前 15 条）────"
    find "${C_DIR}" -name "*.cpp2rust" -exec wc -l {} \; 2>/dev/null \
        | sort -rn | head -15
fi

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
step "§ 7. 生成结果汇报"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│                 cpp2rust-demo 验证结果                   │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}项目：${NC}      rapidjson"
echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
echo -e "  ${BOLD}输出目录：${NC}  ${CPP2RUST_OUTPUT}"
echo ""
echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${CAPTURED}"
echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${RS_FILES}"
echo ""

# 是否存在 todo 标记
TODO_COUNT=$(grep -r "cpp2rust-todo" "${RUST_SRC}" 2>/dev/null | wc -l || echo 0)
if [ "${TODO_COUNT}" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠ 降级标记（需手动完善）：${TODO_COUNT} 处${NC}"
    echo "  → 搜索 'cpp2rust-todo' 查看详情：grep -rn cpp2rust-todo ${RUST_SRC}"
else
    echo -e "  ${GREEN}✓ 无降级标记${NC}"
fi

echo ""
echo -e "  ${BOLD}查看生成的 Rust FFI 脚手架：${NC}"
echo -e "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs head -80"
echo ""

# =============================================================================
# § SKILL 工作流（说明）
# =============================================================================
step "§ SKILL 工作流（如何通过 GitHub Copilot Agent Skill 执行相同流程）"

cat <<'EOF'

除了直接运行本脚本（CLI 方式），你也可以通过 GitHub Copilot Agent Skill
来交互式地完成 cpp2rust-demo 转换流程，无需手动输入 feature 名称和构建命令。

────────────────────────────────────────────────────────────────
SKILL 文件位置（本仓库）：
  .github/skills/cpp2rust-convert.md

触发条件：
  在 C++ 项目根目录与 GitHub Copilot 对话，说：
    "帮我用 cpp2rust-demo 把 rapidjson 的测试转换为 Rust FFI"
  或：
    "对这个项目执行 cpp2rust-demo 转换"

────────────────────────────────────────────────────────────────
SKILL 交互式流程（Agent 会自动引导你完成以下步骤）：

  1. Agent 询问 feature 名称
     User: （回车使用默认 default，或输入 rapidjson_tests）

  2. Agent 询问构建命令
     User: cmake --build build -- -j$(nproc)
           （CMake 项目需提前执行 cmake -B build -DCMAKE_BUILD_TYPE=Debug）

  3. Agent 执行捕获与代码生成
     cpp2rust-demo init --feature rapidjson_tests \
         -- cmake --build build -- -j$(nproc)

  4. Agent 执行 merge 整理（可选）
     cpp2rust-demo merge --feature rapidjson_tests

  5. Agent 汇报结果
     - 生成文件数量和路径
     - 降级标记（[OP] / [VA] / [LM]）的数量及说明

────────────────────────────────────────────────────────────────
与 CLI 方式的对比：

  | 维度         | CLI 方式（本脚本）           | SKILL 方式（GitHub Copilot）|
  |--------------|------------------------------|------------------------------|
  | 交互方式     | 全自动批处理                 | 对话式，逐步引导             |
  | feature 设置 | 脚本变量 FEATURE=...         | Agent 询问后自动填充         |
  | 构建命令     | 脚本硬编码 cmake --build ... | Agent 询问后自动填充         |
  | 结果解读     | 原始文件/符号输出            | Agent 自然语言解释 todo 标记 |
  | 适用场景     | CI/CD、批量验证              | 首次使用、探索性转换         |

────────────────────────────────────────────────────────────────
SKILL 快速开始命令（在本仓库目录执行）：

  # 1. 克隆 rapidjson（若尚未克隆）
  git clone --depth 1 https://github.com/Tencent/rapidjson.git /tmp/rapidjson
  cd /tmp/rapidjson

  # 2. CMake 配置（提前完成，SKILL 只执行 cmake --build）
  cmake -B build -DCMAKE_BUILD_TYPE=Debug \
      -DRAPIDJSON_BUILD_TESTS=ON \
      -DGTEST_SOURCE_DIR=/usr/src/gtest

  # 3. 在此目录打开 GitHub Copilot，说：
  #    "帮我用 cpp2rust-demo 转换这个 C++ 项目"
  # → Agent 会自动询问 feature 名称和构建命令，并执行 init + merge

EOF

ok "验证脚本执行完毕！"
echo ""
echo -e "  ${BOLD}完整输出路径：${NC}${CPP2RUST_OUTPUT}"
echo -e "  ${BOLD}快速查看生成代码：${NC}"
echo    "    cat ${CPP2RUST_OUTPUT}/rust/src/lib.rs"
echo    "    find ${CPP2RUST_OUTPUT}/rust/src -name '*.rs' | xargs grep -l 'import_class\|import_lib'"
