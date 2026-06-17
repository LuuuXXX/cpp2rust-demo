#!/usr/bin/env bash
# =============================================================================
# verify-tomlplusplus.sh —— toml++ 直出（无 shim）工作流本地验证
#
# toml++ 是 header-only 的大型单头 TOML 库。本脚本写一个最小驱动 .cpp
# （define TOML_HEADER_ONLY + include <toml++/toml.hpp> + 仅声明、标量/std 签名的
# 包装类，与 tests/tomlplusplus_e2e_test.rs 同构）触发拦截。
#
# 可配置环境变量：
#   FEATURE          feature 名称（默认 tomlplusplus）
#   SKIP_INSTALL     置 1 跳过 cargo install
#   TOMLPP_DIR       覆盖 tomlplusplus 源码目录（默认 references/tomlplusplus）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${TOMLPP_DIR:-${REPO_DIR}/references/tomlplusplus}"
if [ ! -e "${LIB_DIR}/include/toml++/toml.hpp" ]; then
    c2r_ensure_submodule "references/tomlplusplus" || fail "tomlplusplus 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-tomlplusplus}"
LIB_DISPLAY="toml++（直出工作流，header-only）"
CXX_STD="c++17"
C2R_SRC_FILES=()
C2R_INCLUDES=("${LIB_DIR}/include")
C2R_DRIVER_CONTENT='// toml++ 驱动文件 — 验证大型单头实库解析
#define TOML_HEADER_ONLY 1
#include <toml++/toml.hpp>
#include <string>

namespace tomlwrap_ns {

class TomlWrapper {
public:
    int int_value(const std::string& key) const;
    std::string string_value(const std::string& key) const;
};

}  // namespace tomlwrap_ns'

c2r_run_direct
c2r_finish
