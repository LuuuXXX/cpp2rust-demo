#!/usr/bin/env bash
# =============================================================================
# verify-all.sh
#
# 用途：依次运行全部 8 个真实项目的 FFI 验证脚本
#
# 使用方法：
#   bash usage/verify-all.sh
#   SKIP_INSTALL=1 bash usage/verify-all.sh  # 跳过 cargo install（已安装时）
#
# 项目列表：
#   1. rapidjson    —— 纯 C++ + shim 层（extern-C 包装）
#   2. tinyxml2     —— 单头 + 单 .cpp XML 解析库
#   3. pugixml      —— header-only XML 解析库
#   4. sqlite3      —— 超大单文件（~23 万行 amalgamation）
#   5. nlohmann-json—— header-only 重度模板 JSON 库
#   6. fmtlib       —— 现代 C++ 格式化库（CMake 构建）
#   7. magic-enum   —— header-only 枚举反射库（constexpr + 模板元编程）
#   8. tomlplusplus —— toml++ 大型单头模板库
#
# 每个脚本执行 7 段式验证流程：
#   § 0. 环境检查
#   § 1. 安装 cpp2rust-demo
#   § 2. 定位源文件
#   § 3. 编译目标文件（nm 符号验证）
#   § 4. cpp2rust-demo init（编译拦截 + FFI 生成）
#   § 5. cpp2rust-demo merge（整理输出）
#   § 5a-5c. 验证生成项目（build.rs / cargo check / cargo test）
#   § 6. FFI 验证（nm 符号 / import_class! / import_lib! / link_name）
#   § 7. 结果汇报
#
# 子模块未初始化时，对应脚本会优雅跳过（exit 0），不影响其他项目验证。
# =============================================================================

set -euo pipefail

# ─── 颜色 / 辅助函数 ──────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ OK ]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; }
step()  { echo -e "\n${BOLD}══════════════════════════════════════════${NC}"; \
          echo -e "${BOLD}  $*${NC}"; \
          echo -e "${BOLD}══════════════════════════════════════════${NC}"; }

# ─── 脚本配置 ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 8 个真实项目列表
PROJECTS=(
    "rapidjson"
    "tinyxml2"
    "pugixml"
    "sqlite3"
    "nlohmann-json"
    "fmtlib"
    "magic-enum"
    "tomlplusplus"
)

# 统计变量
PASSED=0
FAILED=0
SKIPPED=0
START_TIME=$(date +%s)

# =============================================================================
# 主循环：依次运行每个项目的验证脚本
# =============================================================================
step "开始执行全部项目验证"

info "脚本目录：${SCRIPT_DIR}"
info "项目总数：${#PROJECTS[@]}"
info "时间：$(date '+%Y-%m-%d %H:%M:%S')"
echo ""

for proj in "${PROJECTS[@]}"; do
    echo ""
    echo "════════════════════════════════════════════════"
    echo "  验证项目：${proj}"
    echo "════════════════════════════════════════════════"
    
    script="${SCRIPT_DIR}/verify-${proj}-ffi.sh"
    
    if [ ! -f "${script}" ]; then
        warn "脚本不存在：${script}"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi
    
    # 执行验证脚本
    if bash "${script}"; then
        ok "${proj} 验证通过 ✓"
        PASSED=$((PASSED + 1))
    else
        EXIT_CODE=$?
        # exit 0 表示子模块未初始化，优雅跳过
        if [ "${EXIT_CODE}" -eq 0 ]; then
            info "${proj} 跳过（子模块未初始化）"
            SKIPPED=$((SKIPPED + 1))
        else
            fail "${proj} 验证失败 ✗"
            FAILED=$((FAILED + 1))
        fi
    fi
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# =============================================================================
# 全局汇总报告
# =============================================================================
echo ""
echo ""
step "全部验证完成"

echo ""
echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│            cpp2rust-demo 全局验证汇总报告               │${NC}"
echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}验证项目总数：${NC}  ${#PROJECTS[@]}"
echo -e "  ${BOLD}通过：${NC}          ${GREEN}${PASSED}${NC}"
echo -e "  ${BOLD}失败：${NC}          ${RED}${FAILED}${NC}"
echo -e "  ${BOLD}跳过：${NC}          ${YELLOW}${SKIPPED}${NC}"
echo -e "  ${BOLD}总耗时：${NC}        ${DURATION} 秒"
echo ""

# 显示详细结果
if [ "${FAILED}" -gt 0 ]; then
    echo -e "${RED}✗ 存在失败的验证项目，请检查上方日志${NC}"
    echo ""
    exit 1
elif [ "${PASSED}" -eq 0 ] && [ "${SKIPPED}" -gt 0 ]; then
    echo -e "${YELLOW}⚠ 所有项目均被跳过（子模块未初始化）${NC}"
    echo -e "  运行以下命令初始化全部子模块："
    echo -e "    git submodule update --init --recursive"
    echo ""
    exit 0
else
    echo -e "${GREEN}✓ 全部验证项目通过！${NC}"
    echo ""
    
    # 成功时显示使用建议
    if [ "${PASSED}" -gt 0 ]; then
        echo "──────────────────────────────────────────────────"
        echo "  验证成功后，你可以："
        echo ""
        echo "  1. 查看生成的 Rust FFI 代码："
        echo "     find .cpp2rust/*/rust/src -name '*.rs' | head"
        echo ""
        echo "  2. 进入生成项目测试冒烟用例："
        echo "     cd .cpp2rust/<project>_demo/rust && cargo test"
        echo ""
        echo "  3. 参考验证脚本为自己的 C++ 项目编写转换流程："
        echo "     cat usage/verify-tinyxml2-ffi.sh  # 单文件项目模板"
        echo "     cat usage/verify-fmtlib-ffi.sh    # CMake 项目模板"
        echo "──────────────────────────────────────────────────"
    fi
    
    exit 0
fi
