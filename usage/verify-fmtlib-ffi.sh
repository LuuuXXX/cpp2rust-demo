#!/usr/bin/env bash
# =============================================================================
# verify-fmtlib-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 {fmt}（fmtlib）的 Rust safe FFI 生成能力。
#
# {fmt} 既可 header-only 使用，也提供编译单元 src/format.cc / src/os.cc。本脚本沿用
# tests/fmtlib_e2e_test.rs 的处理方式，直接拦截库自身实现单元 src/format.cc 与 src/os.cc
# （带真实实现 .cpp 的库），include 路径为 include/ 与 src/。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+编译源 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 子模块缺失时自动 `git submodule update --init references/fmtlib`，失败则跳过。
#
# 可配置环境变量：
#   FEATURE       默认 fmtlib_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="fmtlib"
PROJECT_LABEL="{fmt}（fmtlib，直接拦截库实现单元 format.cc / os.cc）"
SUBMODULE="references/fmtlib"
FEATURE="${FEATURE:-fmtlib_ffi}"
HEADER_ONLY=0
SOURCES="src/format.cc src/os.cc"
INCLUDES="include src"
CXX_STD="${CXX_STD:-c++17}"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
