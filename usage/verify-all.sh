#!/usr/bin/env bash
# =============================================================================
# verify-all.sh —— 统一入口：顺序调用全部 verify-*-ffi.sh 并汇总通过/跳过/失败矩阵
#
# 用途：
#   依次执行 usage/ 下的全部 per-library 验证脚本（rapidjson + 7 个真实库），每库独立
#   计错，单库失败不阻断其余库；末尾打印「通过 / 跳过 / 失败」矩阵。只要有任一库失败，
#   最终以非零退出，供 CI 捕获。
#
# 用法：
#   bash usage/verify-all.sh                       # 跑全部库
#   LIBS="tinyxml2 pugixml" bash usage/verify-all.sh   # 只跑指定库（空格分隔）
#   SKIP_INSTALL=1 bash usage/verify-all.sh         # 已安装 cpp2rust-demo 时加速
#
# 退出码：
#   0 —— 所有被执行的库均通过（跳过的库不计为失败）
#   1 —— 至少一个库执行失败
# =============================================================================

set -uo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 全部库 → 对应脚本（顺序：rapidjson → 实现 .cpp 类 → header-only 类）
ALL_LIBS=(rapidjson tinyxml2 pugixml sqlite3 nlohmann-json fmtlib magic-enum tomlplusplus)

declare -A SCRIPT_OF=(
    [rapidjson]="verify-rapidjson-ffi.sh"
    [tinyxml2]="verify-tinyxml2-ffi.sh"
    [pugixml]="verify-pugixml-ffi.sh"
    [sqlite3]="verify-sqlite3-ffi.sh"
    [nlohmann-json]="verify-nlohmann-json-ffi.sh"
    [fmtlib]="verify-fmtlib-ffi.sh"
    [magic-enum]="verify-magic-enum-ffi.sh"
    [tomlplusplus]="verify-tomlplusplus-ffi.sh"
)

# 按 LIBS= 过滤（未设置则跑全部）
if [ -n "${LIBS:-}" ]; then
    read -r -a SELECTED <<< "${LIBS}"
else
    SELECTED=("${ALL_LIBS[@]}")
fi

declare -A RESULT   # lib → PASS / FAIL / SKIP

echo -e "${BOLD}cpp2rust-demo —— 全部库本地验证（verify-all.sh）${NC}"
echo -e "待执行库：${SELECTED[*]}\n"

for lib in "${SELECTED[@]}"; do
    script="${SCRIPT_OF[$lib]:-}"
    if [ -z "${script}" ]; then
        echo -e "${YELLOW}[WARN]${NC} 未知库：${lib}（跳过）"
        RESULT[$lib]="SKIP"
        continue
    fi
    script_path="${SCRIPT_DIR}/${script}"
    if [ ! -f "${script_path}" ]; then
        echo -e "${YELLOW}[WARN]${NC} 脚本不存在：${script_path}（跳过）"
        RESULT[$lib]="SKIP"
        continue
    fi

    echo -e "\n${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║  开始验证：${lib}  （${script}）${NC}"
    echo -e "${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"

    # 子 shell 运行，单库失败/跳过不影响其余库。
    # 约定：脚本内 `exit 0` 既用于「成功」也用于「子模块缺失而优雅跳过」，
    # 二者退出码相同；这里以退出码 0 记为 PASS，非 0 记为 FAIL。
    if bash "${script_path}"; then
        RESULT[$lib]="PASS"
    else
        RESULT[$lib]="FAIL"
    fi
done

# ─── 汇总矩阵 ────────────────────────────────────────────────────────────────
echo -e "\n${BOLD}┌──────────────────────────────────────────┐${NC}"
echo -e "${BOLD}│          verify-all 结果汇总矩阵          │${NC}"
echo -e "${BOLD}└──────────────────────────────────────────┘${NC}"

PASS_N=0; FAIL_N=0; SKIP_N=0
for lib in "${SELECTED[@]}"; do
    r="${RESULT[$lib]:-SKIP}"
    case "${r}" in
        PASS) echo -e "  ${GREEN}✓ PASS${NC}  ${lib}"; PASS_N=$((PASS_N + 1)) ;;
        FAIL) echo -e "  ${RED}✗ FAIL${NC}  ${lib}"; FAIL_N=$((FAIL_N + 1)) ;;
        *)    echo -e "  ${YELLOW}- SKIP${NC}  ${lib}"; SKIP_N=$((SKIP_N + 1)) ;;
    esac
done

echo ""
echo -e "  通过 ${GREEN}${PASS_N}${NC} / 失败 ${RED}${FAIL_N}${NC} / 跳过 ${YELLOW}${SKIP_N}${NC}"

if [ "${FAIL_N}" -gt 0 ]; then
    echo -e "\n${RED}存在失败的库，verify-all 以非零退出（供 CI 捕获）。${NC}"
    exit 1
fi
echo -e "\n${GREEN}全部被执行的库均通过（跳过的库不计为失败）。${NC}"
