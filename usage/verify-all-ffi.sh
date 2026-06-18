#!/usr/bin/env bash
# =============================================================================
# verify-all-ffi.sh
#
# 用途：一键运行 cpp2rust-demo 全部 8 个真实库的本地验证脚本
#       （rapidjson + tinyxml2 + pugixml + nlohmann-json + fmtlib
#         + magic_enum + tomlplusplus + sqlite3）
#
# 行为：
#   - 每个库独立运行，日志写入 /tmp/cpp2rust-verify-<lib>.log
#   - 任一库失败不阻塞其他库继续运行
#   - 末尾打印 PASS / FAIL 矩阵与失败日志路径
#   - 全部通过 exit 0；任一失败 exit 1（便于 CI 接入）
#
# 用法：
#   bash usage/verify-all-ffi.sh                       # 跑全部 8 个
#   bash usage/verify-all-ffi.sh tinyxml2 fmtlib       # 只跑指定库
#   SKIP_INSTALL=1 bash usage/verify-all-ffi.sh        # 已安装 cpp2rust-demo 时跳过安装
#
# 退出码：
#   0 = 全部通过
#   1 = 至少一个失败
#   2 = 传入了未知的库名
# =============================================================================

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ─── 颜色 ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

# ─── 支持的库清单（与 usage/verify-<lib>-ffi.sh 一一对应） ───────────────────
ALL_LIBS=(
    rapidjson
    tinyxml2
    pugixml
    nlohmann-json
    fmtlib
    magic_enum
    tomlplusplus
    sqlite3
)

# 命令行参数：未传则跑全部；传入了未知库名则报错
SELECTED=()
if [ $# -gt 0 ]; then
    for lib in "$@"; do
        # 校验是否在 ALL_LIBS 中
        known_found=0
        for known in "${ALL_LIBS[@]}"; do
            if [ "$lib" = "$known" ]; then
                SELECTED+=("$lib")
                known_found=1
                break
            fi
        done
        if [ "$known_found" = "0" ]; then
            echo -e "${RED}[FAIL]${NC} 未知库名：$lib"
            echo "  支持的库：${ALL_LIBS[*]}"
            exit 2
        fi
    done
else
    SELECTED=("${ALL_LIBS[@]}")
fi

echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo -e "${BOLD}  cpp2rust-demo 真实库 FFI 验证（聚合）${NC}"
echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo ""
echo -e "  ${BOLD}待验证库：${NC} ${SELECTED[*]}"
echo -e "  ${BOLD}日志目录：${NC} /tmp/cpp2rust-verify-<lib>.log"
echo ""

# ─── 逐个执行 ────────────────────────────────────────────────────────────────
declare -a SCRIPT_MAP_LIBS SCRIPT_MAP_FILES
SCRIPT_MAP_LIBS=(rapidjson tinyxml2 pugixml nlohmann-json fmtlib magic_enum tomlplusplus sqlite3)
SCRIPT_MAP_FILES=(
    "verify-rapidjson-ffi.sh"
    "verify-tinyxml2-ffi.sh"
    "verify-pugixml-ffi.sh"
    "verify-nlohmann-json-ffi.sh"
    "verify-fmtlib-ffi.sh"
    "verify-magic_enum-ffi.sh"
    "verify-tomlplusplus-ffi.sh"
    "verify-sqlite3-ffi.sh"
)

PASS=()
FAIL=()

for lib in "${SELECTED[@]}"; do
    # 找到对应脚本
    script=""
    for i in "${!SCRIPT_MAP_LIBS[@]}"; do
        if [ "${SCRIPT_MAP_LIBS[$i]}" = "$lib" ]; then
            script="${SCRIPT_DIR}/${SCRIPT_MAP_FILES[$i]}"
            break
        fi
    done
    if [ -z "$script" ] || [ ! -f "$script" ]; then
        echo -e "${RED}[FAIL]${NC} $lib：脚本不存在 $script"
        FAIL+=("$lib")
        continue
    fi

    log="/tmp/cpp2rust-verify-${lib}.log"
    echo -e "${CYAN}[RUN ]${NC} $lib ..."
    # 单个库失败不终止整个聚合；记录 rc 后继续
    bash "$script" >"$log" 2>&1
    rc=$?
    if [ "$rc" -eq 0 ]; then
        echo -e "${GREEN}[ OK ]${NC} $lib 通过（日志：$log）"
        PASS+=("$lib")
    else
        echo -e "${RED}[FAIL]${NC} $lib 失败（退出码 $rc，日志：$log）"
        # 打印日志末尾 20 行帮助定位
        echo -e "${YELLOW}──── 末尾 20 行日志 ────${NC}"
        tail -n 20 "$log" 2>/dev/null | sed 's/^/    /'
        echo -e "${YELLOW}────────────────────────${NC}"
        FAIL+=("$lib")
    fi
done

# ─── 矩阵汇总 ────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo -e "${BOLD}  汇总报告${NC}"
echo -e "${BOLD}══════════════════════════════════════════${NC}"
echo ""
total=${#SELECTED[@]}
pass_n=${#PASS[@]}
fail_n=${#FAIL[@]}
echo -e "  ${BOLD}总数：${NC} $total    ${GREEN}通过：$pass_n${NC}    ${RED}失败：$fail_n${NC}"
echo ""

if [ "$pass_n" -gt 0 ]; then
    echo -e "  ${GREEN}通过：${NC}${PASS[*]}"
fi
if [ "$fail_n" -gt 0 ]; then
    echo -e "  ${RED}失败：${NC}${FAIL[*]}"
    echo ""
    echo -e "  ${BOLD}查看失败日志：${NC}"
    for lib in "${FAIL[@]}"; do
        echo "    tail -n 100 /tmp/cpp2rust-verify-${lib}.log"
    done
fi
echo ""

if [ "$fail_n" -gt 0 ]; then
    exit 1
fi
exit 0
