#!/usr/bin/env bash
# =============================================================================
# verify-tomlplusplus-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 toml++（tomlplusplus，header-only）的 Rust safe FFI 生成能力。
#
# toml++ 是大型单头实库（header-only）。本脚本生成最小驱动 .cpp，include 库头触发解析压测；
# 驱动类方法仅声明，签名只用标量/std 类型（不引用库类型），与 tests/tomlplusplus_e2e_test.rs
# 的 TOML_DRIVER_CPP 同构，保证生成绑定可编译。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+生成驱动 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/tomlplusplus`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 tomlplusplus_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="tomlplusplus"
PROJECT_LABEL="toml++（tomlplusplus，header-only，大型单头实库）"
SUBMODULE="references/tomlplusplus"
FEATURE="${FEATURE:-tomlplusplus_ffi}"
HEADER_ONLY=1
INCLUDES="include"
CXX_STD="${CXX_STD:-c++17}"

# 最小驱动 .cpp（复刻 tests/tomlplusplus_e2e_test.rs 的 TOML_DRIVER_CPP）
DRIVER_CPP='
// toml++ 驱动文件 — 用于测试大型单头实库的解析能力
#define TOML_HEADER_ONLY 1
#include <toml++/toml.hpp>
#include <string>

namespace tomlwrap_ns {

// 方法仅声明，签名用标量/std 类型（不引用 toml 类型），
// 与 nlohmann/json E2E 同构，保证生成绑定可编译。
class TomlWrapper {
public:
    int int_value(const std::string& key) const;
    std::string string_value(const std::string& key) const;
};

}  // namespace tomlwrap_ns
'

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
