#!/usr/bin/env bash
# =============================================================================
# verify-all.sh —— 统一入口：顺序执行全部 verify-<lib>.sh 并汇总总表
#
# 顺序运行每个直出/ C 接口工作流脚本，把每个库的「捕获文件数 / 生成 .rs 数 /
# 绑定数 / 降级标记 / cargo check 结果」汇总为一张总表；任一库失败累加错误并最终
# 非零退出。
#
# 可配置环境变量：
#   ONLY           逗号分隔的库子集（如 ONLY=tinyxml2,fmtlib），仅运行其中的脚本
#   SKIP_INSTALL   置 1 跳过 cargo install（复用已安装工具，强烈建议批量运行时设置）
#   FEATURE        若设置则透传给各子脚本（一般留空，让每个库用各自默认 feature）
#
# 示例：
#   SKIP_INSTALL=1 bash usage/verify-all.sh
#   SKIP_INSTALL=1 ONLY=tinyxml2,fmtlib bash usage/verify-all.sh
# =============================================================================
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

# 直出工作流库清单（rapidjson 为 shim 工作流，由 verify-rapidjson-ffi.sh 单独运行）。
ALL_LIBS=(tinyxml2 pugixml nlohmann-json fmtlib magic-enum tomlplusplus sqlite3)

# 解析 ONLY 子集。
declare -a LIBS=()
if [ -n "${ONLY:-}" ]; then
    IFS=',' read -r -a sel <<< "${ONLY}"
    for want in "${sel[@]}"; do
        want="$(echo "${want}" | tr -d '[:space:]')"
        for lib in "${ALL_LIBS[@]}"; do
            if [ "${lib}" = "${want}" ]; then LIBS+=("${lib}"); fi
        done
    done
    if [ "${#LIBS[@]}" -eq 0 ]; then
        fail "ONLY=${ONLY} 未匹配任何已知库（可选：${ALL_LIBS[*]}）"
    fi
else
    LIBS=("${ALL_LIBS[@]}")
fi

# 总表临时文件：每行 "lib|result|logfile"。
SUMMARY=$(mktemp)
TOTAL_ERRORS=0

# 强制把 SKIP_INSTALL 透传给子脚本（避免每个库重复 cargo install）。
export SKIP_INSTALL="${SKIP_INSTALL:-0}"

for lib in "${LIBS[@]}"; do
    script="${SCRIPT_DIR}/verify-${lib}.sh"
    step "运行 verify-${lib}.sh"
    if [ ! -x "${script}" ] && [ ! -f "${script}" ]; then
        warn "脚本不存在：${script}，跳过"
        echo "${lib}|missing|" >> "${SUMMARY}"
        TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        continue
    fi
    log=$(mktemp)
    if bash "${script}" 2>&1 | tee "${log}"; then
        result="pass"
    else
        result="fail"
        TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
    fi
    # 从子脚本输出里抽取统计数字（容错：缺失则填 ?）。
    captured=$(grep -oE '捕获预处理文件数：[[:space:]]*[0-9]+' "${log}" | tail -1 | grep -oE '[0-9]+$' || echo "?")
    rs=$(grep -oE '生成 Rust 文件数：[[:space:]]*[0-9]+' "${log}" | tail -1 | grep -oE '[0-9]+$' || echo "?")
    fns=$(grep -oE '函数绑定数：[[:space:]]*[0-9]+' "${log}" | tail -1 | grep -oE '[0-9]+$' || echo "?")
    todo=$(grep -oE '降级标记数：[[:space:]]*[0-9]+' "${log}" | tail -1 | grep -oE '[0-9]+$' || echo "?")
    cc=$(grep -oE 'cargo check：[[:space:]]*[a-z]+' "${log}" | tail -1 | grep -oE '[a-z]+$' || echo "?")
    echo "${lib}|${result}|${captured}|${rs}|${fns}|${todo}|${cc}" >> "${SUMMARY}"
    rm -f "${log}"
done

# ─── 汇总总表 ─────────────────────────────────────────────────────────────────
step "§ 全部库验证汇总"
printf '%-16s %-6s %-8s %-8s %-8s %-8s %-8s\n' "库" "结果" "捕获" "生成rs" "绑定" "降级" "check"
printf '%-16s %-6s %-8s %-8s %-8s %-8s %-8s\n' "----" "----" "----" "----" "----" "----" "----"
while IFS='|' read -r lib result captured rs fns todo cc; do
    printf '%-16s %-6s %-8s %-8s %-8s %-8s %-8s\n' \
        "${lib}" "${result}" "${captured:-?}" "${rs:-?}" "${fns:-?}" "${todo:-?}" "${cc:-?}"
done < "${SUMMARY}"
rm -f "${SUMMARY}"

echo ""
if [ "${TOTAL_ERRORS}" -gt 0 ]; then
    fail "共有 ${TOTAL_ERRORS} 个库验证失败，请检查上方各库 [FAIL] / [WARN] 输出"
fi
ok "全部库验证通过 ✓"
