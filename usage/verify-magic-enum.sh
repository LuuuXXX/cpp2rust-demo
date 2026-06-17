#!/usr/bin/env bash
# =============================================================================
# verify-magic-enum.sh —— magic_enum 直出（无 shim）工作流本地验证
#
# magic_enum 是 header-only 库（重度 constexpr / 模板元编程）。本脚本写一个最小驱动
# .cpp（include <magic_enum/magic_enum.hpp> + 仅声明、标量/std 签名的包装类，与
# tests/magic_enum_e2e_test.rs 同构）触发拦截，验证重度 constexpr 头文件的解析。
#
# 可配置环境变量：
#   FEATURE          feature 名称（默认 magic_enum）
#   SKIP_INSTALL     置 1 跳过 cargo install
#   MAGIC_ENUM_DIR   覆盖 magic_enum 源码目录（默认 references/magic_enum）
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/lib/common.sh"

c2r_repo_dir >/dev/null
c2r_init_env
c2r_install_tool

LIB_DIR="${MAGIC_ENUM_DIR:-${REPO_DIR}/references/magic_enum}"
if [ ! -e "${LIB_DIR}/include/magic_enum/magic_enum.hpp" ]; then
    c2r_ensure_submodule "references/magic_enum" || fail "magic_enum 源码缺失：${LIB_DIR}"
fi

FEATURE="${FEATURE:-magic_enum}"
LIB_DISPLAY="magic_enum（直出工作流，header-only）"
CXX_STD="c++17"
C2R_SRC_FILES=()
C2R_INCLUDES=("${LIB_DIR}/include")
C2R_DRIVER_CONTENT='// magic_enum 驱动文件 — 验证重度 constexpr/模板头文件解析
#include <magic_enum/magic_enum.hpp>
#include <string>

namespace enumwrap_ns {

enum class Color { Red, Green, Blue };

class ColorWrapper {
public:
    int count() const;
    std::string name_of(int idx) const;
};

}  // namespace enumwrap_ns'

c2r_run_direct
c2r_finish
