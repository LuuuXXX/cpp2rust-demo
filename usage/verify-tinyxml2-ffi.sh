#!/usr/bin/env bash
# =============================================================================
# verify-tinyxml2-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 tinyxml2 C++ 库的 Rust safe FFI 生成能力。
#
# tinyxml2 是单头 + 单 .cpp 的经典 XML 解析库，含 XMLDocument/XMLElement/XMLNode
# 等典型 OOP 类层级。本脚本直接拦截库自身实现单元 tinyxml2.cpp（带真实实现 .cpp 的库）。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+编译源 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/tinyxml2`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 tinyxml2_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="tinyxml2"
PROJECT_LABEL="tinyxml2（单头 + 单 .cpp XML 解析库，直接拦截库实现单元）"
SUBMODULE="references/tinyxml2"
FEATURE="${FEATURE:-tinyxml2_ffi}"
HEADER_ONLY=0
SOURCES="tinyxml2.cpp"
INCLUDES=""                       # 头文件与 .cpp 同目录，子模块根即包含目录
CXX_STD="${CXX_STD:-c++17}"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
