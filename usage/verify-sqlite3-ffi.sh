#!/usr/bin/env bash
# =============================================================================
# verify-sqlite3-ffi.sh
#
# 用途：本地验证 cpp2rust-demo 对 SQLite3 C 接口的 Rust safe FFI 生成能力。
#
# SQLite3 是纯 `extern "C"` 接口的 C 库。本脚本沿用 tests/sqlite3_e2e_test.rs 的处理
# 方式：以一层 C++ 驱动包装 `extern "C" { #include <sqlite3.h> }`，直接使用系统安装的
# sqlite3.h 头文件并链接系统 libsqlite3，验证工具对 extern "C" 接口的 import_lib! 映射。
#
# 流程总览（详见 usage/lib/verify-common.sh 的七阶段骨架）：
#   § 0 环境检查 → § 1 安装工具 → § 2/3 定位+编译源 → § 4 init → § 5 merge
#   → § 5a build.rs 校验 → § 5b/c cargo check/test → § 6 符号验证 → § 7 汇报
#
# 系统 sqlite3.h 缺失时自动跳过（不中断）：
#   Ubuntu/Debian 安装：sudo apt-get install -y libsqlite3-dev
#
# 可配置环境变量：
#   FEATURE       默认 sqlite3_ffi
#   SKIP_INSTALL  置 1 跳过 cargo install（已安装时加速）
#   CXX_STD       默认 c++17
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev \
#                           libsqlite3-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

LIB_NAME="sqlite3"
PROJECT_LABEL="SQLite3（纯 extern \"C\" 接口，经 C++ 驱动包装系统 sqlite3.h）"
FEATURE="${FEATURE:-sqlite3_ffi}"
CXX_STD="${CXX_STD:-c++17}"

# 系统头文件模式：不依赖子模块，校验 /usr/include/sqlite3.h 存在
SKIP_SUBMODULE=1
REQUIRE_SYSTEM_HEADER="${REQUIRE_SYSTEM_HEADER:-/usr/include/sqlite3.h}"

# C++ 驱动：经 extern "C" 包装系统 sqlite3.h（与 tests/sqlite3_e2e_test.rs 一致）
DRIVER_CPP='
// sqlite3 C++ 驱动 —— 用于测试工具对 extern "C" 接口的处理能力
extern "C" {
#include <sqlite3.h>
}
'

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/verify-common.sh"

vc_run
