#!/usr/bin/env bash
# =============================================================================
# verify-nlohmann-json-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 nlohmann/json（header-only）的 Rust safe FFI 生成能力。
#
# nlohmann/json 是 header-only 库（单个 json.hpp ~23K 行，重度模板）。本脚本生成最小
# 驱动 .cpp，include 库头触发解析压测；驱动类方法仅声明，签名只用标量/std 类型（不引用
# 库内部类型），与 tests/nlohmann_json_e2e_test.rs 的 JSON_DRIVER_CPP 同构，保证生成绑定可编译。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+生成驱动 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/nlohmann-json`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 nlohmann_json_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="nlohmann-json"
PROJECT_LABEL="nlohmann/json（header-only，单超大头文件 + 重度模板）"
SUBMODULE="references/nlohmann-json"
FEATURE="${FEATURE:-nlohmann_json_ffi}"
HEADER_ONLY=1
INCLUDES="include"
CXX_STD="${CXX_STD:-c++17}"

# 最小驱动 .cpp（复刻 tests/nlohmann_json_e2e_test.rs 的 JSON_DRIVER_CPP）
DRIVER_CPP='
// nlohmann/json 驱动文件 — 用于测试模板类提取能力
#include <nlohmann/json.hpp>

// 使用基本类型触发模板实例化
using json = nlohmann::json;

class JsonWrapper {
public:
    json parse(const std::string& s);
    void set_int(const std::string& key, int value);
};
'

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
