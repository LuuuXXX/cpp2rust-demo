#!/usr/bin/env bash
# =============================================================================
# verify-magic-enum-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 magic_enum（header-only）的 Rust safe FFI 生成能力。
#
# magic_enum 是 header-only 库，重度使用 constexpr / 模板元编程。本脚本生成最小驱动
# .cpp，include 库头触发解析压测；驱动类方法仅声明，签名只用标量/std 类型（不引用库类型），
# 与 tests/magic_enum_e2e_test.rs 的 ENUM_DRIVER_CPP 同构，保证生成绑定可编译。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+生成驱动 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/magic_enum`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 magic_enum_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="magic_enum"
PROJECT_LABEL="magic_enum（header-only，重度 constexpr / 模板元编程）"
SUBMODULE="references/magic_enum"
FEATURE="${FEATURE:-magic_enum_ffi}"
HEADER_ONLY=1
INCLUDES="include"
CXX_STD="${CXX_STD:-c++17}"

# 最小驱动 .cpp（复刻 tests/magic_enum_e2e_test.rs 的 ENUM_DRIVER_CPP）
DRIVER_CPP='
// magic_enum 驱动文件 — 用于测试重度 constexpr/模板元编程头文件的解析能力
#include <magic_enum/magic_enum.hpp>
#include <string>

namespace enumwrap_ns {

enum class Color { Red, Green, Blue };

// 方法仅声明，签名用标量/std 类型（不引用 magic_enum 类型），
// 与 nlohmann/json E2E 同构，保证生成绑定可编译。
class ColorWrapper {
public:
    int count() const;
    std::string name_of(int idx) const;
};

}  // namespace enumwrap_ns
'

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
