#!/usr/bin/env bash
# =============================================================================
# verify-nlohmann-json.sh —— nlohmann/json 直出（无 shim）工作流本地验证
#
# nlohmann/json 是 header-only 库（单个 json.hpp ~23K 行，重度模板）。本脚本写一个
# 最小驱动 .cpp（include <nlohmann/json.hpp> + 一个仅声明、标量/std 签名的包装类，
# 与 tests/nlohmann_json_e2e_test.rs 同构）触发拦截，验证超大头文件的解析与可编译性。
#
# 可配置环境变量：
#   FEATURE          feature 名称（默认 nlohmann_json）
#   SKIP_INSTALL     置 1 跳过 cargo install
#   NLOHMANN_DIR     覆盖 nlohmann-json 源码目录（默认 references/nlohmann-json）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${NLOHMANN_DIR:-${REPO_DIR}/references/nlohmann-json}"
if [ ! -e "${LIB_DIR}/include/nlohmann/json.hpp" ]; then
    c2r_ensure_submodule "references/nlohmann-json" || fail "nlohmann-json 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-nlohmann_json}"
LIB_DISPLAY="nlohmann/json（直出工作流，header-only）"
CXX_STD="c++17"
C2R_SRC_FILES=()
C2R_INCLUDES=("${LIB_DIR}/include")
# 驱动：仅声明、标量/std 签名，不引用 nlohmann 模板类型，保证生成绑定可编译。
C2R_DRIVER_CONTENT='// nlohmann/json 驱动文件 — 验证超大模板头文件解析
#include <nlohmann/json.hpp>
#include <string>

namespace jsonwrap_ns {

class JsonWrapper {
public:
    int int_value(const std::string& key) const;
    std::string string_value(const std::string& key) const;
};

}  // namespace jsonwrap_ns'

c2r_run_direct
c2r_finish
