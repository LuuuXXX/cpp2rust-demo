#!/usr/bin/env bash
# =============================================================================
# verify-pugixml-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 pugixml C++ 库的 Rust safe FFI 生成能力。
#
# pugixml 是单头 + 单源的 XML 解析库，具有清晰的 xml_document/xml_node/
# xml_attribute OOP API 与迭代器类。本脚本直接拦截库自身实现单元 src/pugixml.cpp。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+编译源 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/pugixml`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 pugixml_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="pugixml"
PROJECT_LABEL="pugixml（单头 + 单源 XML 解析库，直接拦截库实现单元）"
SUBMODULE="references/pugixml"
FEATURE="${FEATURE:-pugixml_ffi}"
HEADER_ONLY=0
SOURCES="src/pugixml.cpp"
INCLUDES="src"                    # pugixml.hpp 位于 src/，与实现单元同目录
CXX_STD="${CXX_STD:-c++17}"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
