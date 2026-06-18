#!/usr/bin/env bash
# =============================================================================
# common.sh — cpp2rust-demo 真实库本地验证脚本共享库
#
# 用途：被 usage/verify-<lib>-ffi.sh 系列脚本 source，提供统一的环境检查、
#       工具安装、子模块初始化、init/merge 调度、cargo 校验、FFI 审计与
#       报告生成等通用功能。各 verify-<lib>-ffi.sh 脚本只需声明库特有参数
#       （LIB_NAME / FEATURE / SOURCES / INCLUDE_DIRS / SUBMODULE_REL），
#       即可复用本库完成完整的 init → merge → cargo check → cargo test →
#       FFI 审计闭环。
#
# 使用方式（典型）：
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "${SCRIPT_DIR}/lib/common.sh"
#   LIB_NAME="tinyxml2"
#   FEATURE="${FEATURE:-${LIB_NAME}_ffi}"
#   cpp2rust_run_full_pipeline "$@"
#
# 设计原则：
#   - 行为与 usage/verify-rapidjson-ffi.sh 的等价语义保持一致
#   - 不依赖 cpp2rust-demo 的任何额外子命令；只调用 init / merge
#   - 兼容 Linux 与 Windows MSYS2（cygpath 自动转换）
#   - 任何必失败步骤立即 exit 1；非致命检查累计到 SCRIPT_ERRORS，末尾统一退出
# =============================================================================

# ─── 颜色 / 输出辅助 ─────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

cpp2rust_info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
cpp2rust_ok()    { echo -e "${GREEN}[ OK ]${NC}  $*"; }
cpp2rust_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
cpp2rust_fail()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }
cpp2rust_step()  {
    echo -e "\n${BOLD}══════════════════════════════════════════${NC}"
    echo -e "${BOLD}  $*${NC}"
    echo -e "${BOLD}══════════════════════════════════════════${NC}"
}

# 全局错误计数器：非致命检查失败时递增，由 cpp2rust_exit_with_error_check 统一退出
CPP2RUST_SCRIPT_ERRORS=${CPP2RUST_SCRIPT_ERRORS:-0}
cpp2rust_record_error() { CPP2RUST_SCRIPT_ERRORS=$((CPP2RUST_SCRIPT_ERRORS + 1)); }
cpp2rust_exit_with_error_check() {
    if [ "${CPP2RUST_SCRIPT_ERRORS}" -gt 0 ]; then
        cpp2rust_fail "验证脚本发现 ${CPP2RUST_SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
    fi
    cpp2rust_ok "验证脚本执行完毕！"
}

# ─── 仓库根定位 ─────────────────────────────────────────────────────────────
# 由调用方脚本所在目录向上找 git 仓库根。结果写入全局 CPP2RUST_REPO_DIR。
cpp2rust_find_repo_root() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[2]:-$0}")" && pwd)"
    CPP2RUST_REPO_DIR="$(git -C "${script_dir}" rev-parse --show-toplevel 2>/dev/null || echo "${script_dir}/..")"
    CPP2RUST_REPO_DIR="$(cd "${CPP2RUST_REPO_DIR}" && pwd)"
    echo "${CPP2RUST_REPO_DIR}"
}

# ─── 路径转换：MSYS2/Cygwin POSIX 路径 → Windows 工具链可识别形式 ────────────
# 背景：在 MSYS2 上脚本以 POSIX 路径（/d/a/...）运行，但生成的 cargo 项目
#   默认调用原生 Windows cl.exe / gcc.exe，对 /d/... 形式路径识别失败。
#   cygpath -m 转为 D:/a/... 混合形式，cl.exe 与 gcc 均可识别。
cpp2rust_to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

# ─── § 0. 环境检查 ───────────────────────────────────────────────────────────
# 参数：要检查的命令列表，如 `cpp2rust_require_cmds git g++ cargo nm`
cpp2rust_require_cmds() {
    local missing=()
    for cmd in "$@"; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done
    if [ ${#missing[@]} -gt 0 ]; then
        cpp2rust_fail "未找到命令：${missing[*]}  请先安装后重试（Ubuntu/Debian: sudo apt-get install -y ...）"
    fi
}

# 检查 libclang（cpp2rust-demo 依赖），缺失只警告不退出
cpp2rust_check_libclang() {
    if ! pkg-config --exists libclang 2>/dev/null && \
       ! find /usr /lib /usr/local -name "libclang*.so*" 2>/dev/null | grep -q .; then
        cpp2rust_warn "未检测到 libclang（可能未安装 libclang-dev）。cpp2rust-demo 依赖 libclang 解析 AST。"
        cpp2rust_warn "  Ubuntu/Debian：sudo apt-get install -y clang libclang-dev"
    fi
}

# ─── § 1. 安装 cpp2rust-demo ────────────────────────────────────────────────
# 参数：SKIP_INSTALL（"1" 表示已安装则跳过）
#
# 解析顺序：
#   1. command -v 找到 cpp2rust-demo → 直接用
#   2. SKIP_INSTALL=1 → 不再尝试网络/本地构建，直接 fail（用户期望已安装但找不到）
#   3. 当前仓库根有 Cargo.toml（开发环境）→ cargo build --release + PATH 注入
#   4. 否则 → cargo install --git <URL> cpp2rust-demo（显式指定包名，避免多 bin 冲突）
cpp2rust_install_tool() {
    local skip_install="${1:-0}"

    # 1. 已在 PATH
    if command -v cpp2rust-demo &>/dev/null; then
        cpp2rust_ok "已检测到 cpp2rust-demo：$(command -v cpp2rust-demo)"
        return 0
    fi

    # 2. SKIP_INSTALL=1 但找不到 → fail
    if [ "${skip_install}" = "1" ]; then
        cpp2rust_fail "SKIP_INSTALL=1 但未在 PATH 中找到 cpp2rust-demo。请先 cargo install 或将其加入 PATH。"
    fi

    # 3. 当前仓库根含 Cargo.toml（开发环境）→ 本地构建并注入 PATH
    if [ -f "${CPP2RUST_REPO_DIR}/Cargo.toml" ] && grep -q '^name = "cpp2rust-demo"' "${CPP2RUST_REPO_DIR}/Cargo.toml" 2>/dev/null; then
        cpp2rust_info "检测到开发仓库（${CPP2RUST_REPO_DIR}），执行 cargo build --release …"
        if ! (cd "${CPP2RUST_REPO_DIR}" && cargo build --release) ; then
            cpp2rust_fail "cargo build --release 失败"
        fi
        local bin_path="${CPP2RUST_REPO_DIR}/target/release"
        export PATH="${bin_path}:${PATH}"
        cpp2rust_ok "本地构建完成，已注入 PATH：${bin_path}"
        command -v cpp2rust-demo >/dev/null || cpp2rust_fail "本地构建后仍未找到 cpp2rust-demo（检查 ${bin_path}）"
        return 0
    fi

    # 4. 在线安装（显式指定包名，避免 upstream 仓库多 bin 冲突）
    cpp2rust_info "从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…"
    local install_log
    install_log=$(mktemp)
    if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked cpp2rust-demo \
             >"${install_log}" 2>&1; then
        echo "── cargo install 失败，完整日志：──"
        cat "${install_log}"
        rm -f "${install_log}"
        cpp2rust_fail "cpp2rust-demo 安装失败，请检查上方日志"
    fi
    tail -5 "${install_log}"
    rm -f "${install_log}"
    cpp2rust_ok "cpp2rust-demo 安装完成：$(command -v cpp2rust-demo || echo '?')"
}

# ─── § 2. 子模块初始化（git submodule） ─────────────────────────────────────
# 参数：子模块相对仓库根的路径（如 references/tinyxml2）
#       若路径下已有 .git 或文件，视为已初始化，直接返回。
cpp2rust_init_submodule() {
    local submodule_rel="$1"
    local submodule_abs="${CPP2RUST_REPO_DIR}/${submodule_rel}"
    if [ ! -d "${submodule_abs}" ]; then
        cpp2rust_fail "子模块目录不存在：${submodule_abs}"
    fi
    # 简单判定：目录非空（已 checkout）即视为可用，跳过网络操作
    if [ -n "$(ls -A "${submodule_abs}" 2>/dev/null)" ]; then
        cpp2rust_ok "子模块 ${submodule_rel} 已就绪"
        return 0
    fi
    if [ "${OFFLINE:-0}" = "1" ]; then
        cpp2rust_warn "OFFLINE=1 且子模块 ${submodule_rel} 未初始化，跳过（脚本可能失败）"
        return 0
    fi
    cpp2rust_info "初始化子模块 ${submodule_rel} …"
    if ! git -C "${CPP2RUST_REPO_DIR}" submodule update --init "${submodule_rel}"; then
        cpp2rust_fail "子模块 ${submodule_rel} 初始化失败"
    fi
    cpp2rust_ok "子模块 ${submodule_rel} 初始化完成"
}

# 检查系统头文件存在性（如 sqlite3.h），缺失则 graceful skip 退出脚本（exit 0）
# 参数：header 绝对路径，缺失时给出的安装提示
cpp2rust_require_system_header_or_skip() {
    local header="$1"
    local hint="${2:-}"
    if [ ! -f "${header}" ]; then
        cpp2rust_warn "系统头文件不存在：${header}"
        [ -n "${hint}" ] && cpp2rust_warn "  ${hint}"
        cpp2rust_warn "脚本 graceful skip（非错误退出）"
        exit 0
    fi
    cpp2rust_ok "系统头文件就绪：${header}"
}

# ─── § 3. 编译目标文件（供 nm 符号验证） ─────────────────────────────────────
# 全局：调用方需自行 mktemp OBJ_DIR 与 trap 清理；本函数只负责编译。
# 参数：OBJ_DIR INCLUDE_DIR1 [INCLUDE_DIR2...] -- SRC1 [SRC2...]
#   注意 -- 为分隔符，前段是 -I 路径，后段是源文件
# C++ 标准可通过环境变量 CXX_STD 覆盖（默认 c++17）
cpp2rust_compile_units() {
    local obj_dir="$1"; shift
    local cxx_std="${CXX_STD:-c++17}"
    local -a includes=() sources=()
    local in_includes=1
    while [ $# -gt 0 ]; do
        if [ "$1" = "--" ]; then
            in_includes=0; shift; continue
        fi
        if [ "$in_includes" = "1" ]; then
            includes+=("$1")
        else
            sources+=("$1")
        fi
        shift
    done

    if [ ${#sources[@]} -eq 0 ]; then
        cpp2rust_warn "cpp2rust_compile_units: 无源文件可编译"
        return 0
    fi

    local compile_errors=0
    local src name obj log
    for src in "${sources[@]}"; do
        name="$(basename "${src%.*}")"
        obj="${obj_dir}/${name}.o"
        log="${obj_dir}/${name}.log"
        local -a args=(g++ -c -std="${cxx_std}" -w)
        for inc in "${includes[@]}"; do args+=("-I${inc}"); done
        args+=("${src}" -o "${obj}")
        if "${args[@]}" >"${log}" 2>&1; then
            cpp2rust_info "编译成功：${name}.o"
        else
            cpp2rust_warn "编译失败：$(basename "${src}")"
            cat "${log}" >&2 || true
            compile_errors=$((compile_errors + 1))
        fi
    done
    if [ "${compile_errors}" -gt 0 ]; then
        cpp2rust_warn "${compile_errors} 个源文件编译失败，nm 验证可能不完整"
    else
        cpp2rust_ok "全部 ${#sources[@]} 个源文件编译成功"
    fi
}

# ─── § 4. cpp2rust-demo init 辅助 ───────────────────────────────────────────
# 工作目录策略（重要）：
#   cpp2rust-demo 通过 LD_PRELOAD hook 捕获 .cpp2rust 文件，路径相对 project_root
#   保存。若 project_root 是仓库根，源文件位于 references/<lib>/foo.cpp 时，
#   unit_path 会变成 references/<lib>/foo，含路径分隔符 '/'，污染 link_name
#   并破坏 hicc 命名空间拼接宏。
#
#   解决方案：脚本从 PROJECT_ROOT（即库自身根目录）运行 init/merge，
#   使捕获的相对路径只剩 basename（如 tinyxml2.cpp.cpp2rust → unit_path=tinyxml2），
#   link_name 自然干净。代价是 .cpp2rust/ 输出落在 PROJECT_ROOT 内（已通过
#   .gitignore 的 /references/**/.cpp2rust/ 规则忽略，不污染子模块状态）。
#
#   调用方通过 cpp2rust_set_workdir 设置 CPP2RUST_WORKDIR；默认 = CPP2RUST_REPO_DIR
CPP2RUST_WORKDIR="${CPP2RUST_WORKDIR:-}"
cpp2rust_set_workdir() {
    CPP2RUST_WORKDIR="$1"
    cpp2rust_info "init/merge 工作目录：${CPP2RUST_WORKDIR}"
}
cpp2rust_workdir() {
    if [ -z "${CPP2RUST_WORKDIR}" ]; then
        CPP2RUST_WORKDIR="${CPP2RUST_REPO_DIR}"
    fi
    echo "${CPP2RUST_WORKDIR}"
}

# 生成临时构建脚本（bash），编译给定源文件，触发 LD_PRELOAD hook 拦截
# 参数：输出脚本路径 INCLUDE_DIR1 [INCLUDE_DIR2...] -- SRC1 [SRC2...]
cpp2rust_make_build_script() {
    local out_script="$1"; shift
    local cxx_std="${CXX_STD:-c++17}"
    local -a includes=() sources=()
    local in_includes=1
    while [ $# -gt 0 ]; do
        if [ "$1" = "--" ]; then
            in_includes=0; shift; continue
        fi
        if [ "$in_includes" = "1" ]; then
            includes+=("$1")
        else
            sources+=("$1")
        fi
        shift
    done

    {
        echo '#!/bin/bash'
        echo '# 临时构建脚本：由 cpp2rust-demo init 的 LD_PRELOAD hook 拦截 g++ 调用'
        echo 'set -e'
        local inc_args=""
        for inc in "${includes[@]}"; do
            inc_args+=" -I\"${inc}\""
        done
        for src in "${sources[@]}"; do
            echo "g++ -c -std=\"${cxx_std}\"${inc_args} \"${src}\" -o /dev/null 2>&1 || true"
        done
    } > "${out_script}"
    chmod +x "${out_script}"
}

# 生成「driver cpp」（针对 header-only 库）：仅 #include 头文件，触发预处理拦截
# 参数：输出 cpp 路径 INCLUDE_DIR1 [INCLUDE_DIR2...] -- HEADER1 [HEADER2...]
cpp2rust_make_driver_cpp() {
    local out_cpp="$1"; shift
    local -a includes=() headers=()
    local in_includes=1
    while [ $# -gt 0 ]; do
        if [ "$1" = "--" ]; then
            in_includes=0; shift; continue
        fi
        if [ "$in_includes" = "1" ]; then
            includes+=("$1")
        else
            headers+=("$1")
        fi
        shift
    done

    {
        echo "// 自动生成的 driver cpp — 触发 header-only 库的预处理拦截"
        for h in "${headers[@]}"; do
            echo "#include \"${h}\""
        done
        echo ""
        echo "// 引用若干导出符号，确保不被优化掉"
        echo "extern \"C\" int cpp2rust_driver_main() { return 0; }"
    } > "${out_cpp}"
    cpp2rust_info "已生成 driver cpp：${out_cpp}"
}

# 执行 init：cpp2rust-demo init --feature <FEATURE> -- bash <BUILD_SCRIPT>
# 参数：FEATURE BUILD_SCRIPT
# 行为：从 CPP2RUST_WORKDIR 执行（默认 CPP2RUST_REPO_DIR；调用方一般先
#       cpp2rust_set_workdir "${PROJECT_ROOT}" 让 unit_path 保持干净）
cpp2rust_run_init() {
    local feature="$1"
    local build_script="$2"
    local workdir
    workdir="$(cpp2rust_workdir)"
    cpp2rust_info "工作目录：${workdir}"
    cpp2rust_info "feature 名称：${feature}"
    cpp2rust_info "构建命令：bash ${build_script}"
    # 非交互（脚本环境）下 cpp2rust-demo 自动全选 .cpp 文件
    (cd "${workdir}" && cpp2rust-demo init --feature "${feature}" -- bash "${build_script}") \
        || cpp2rust_fail "cpp2rust-demo init 失败"
    cpp2rust_ok "cpp2rust-demo init 完成"
}

# ─── § 5. cpp2rust-demo merge ────────────────────────────────────────────────
# 参数：FEATURE
# 行为：从 CPP2RUST_WORKDIR 执行（与 init 保持一致，确保读到同一份 .cpp2rust/）
cpp2rust_run_merge() {
    local feature="$1"
    local workdir
    workdir="$(cpp2rust_workdir)"
    (cd "${workdir}" && cpp2rust-demo merge --feature "${feature}") \
        || cpp2rust_fail "cpp2rust-demo merge 失败"
    cpp2rust_ok "cpp2rust-demo merge 完成"
}

# 返回该 feature 的输出目录（基于 WORKDIR）
cpp2rust_output_dir() {
    local feature="$1"
    local workdir
    workdir="$(cpp2rust_workdir)"
    echo "${workdir}/.cpp2rust/${feature}"
}

# ─── § 5b/c. cargo 校验 ─────────────────────────────────────────────────────
# 参数：RUST_PROJECT_DIR
#
# 行为策略：
#   cargo check 失败可能源自 cpp2rust-demo 对某些 C++ 特性的不完整支持（如抽象类
#   make_unique 工厂、模板方法签名复杂等）。这与 hicc import_lib! 在测试二进制
#   链接时的 _hicc_export_methods_* 限制一样，属于工具的已知短板，不应阻塞其他
#   验证步骤。因此 cargo check / test 失败时仅记录错误（cpp2rust_record_error），
#   脚本末尾 cpp2rust_exit_with_error_check 统一决定退出码。
cpp2rust_cargo_check() {
    local rust_project="$1"
    if [ ! -f "${rust_project}/Cargo.toml" ]; then
        cpp2rust_warn "未找到 ${rust_project}/Cargo.toml，跳过 cargo check"
        return 0
    fi
    cpp2rust_info "在 ${rust_project} 中运行 cargo check ..."
    if (cd "${rust_project}" && cargo check 2>&1); then
        cpp2rust_ok "cargo check 通过 ✓"
    else
        cpp2rust_warn "cargo check 失败 — 可能是 cpp2rust-demo 对该库的已知限制（如抽象类工厂、模板方法签名）"
        cpp2rust_warn "  详见上方 cargo 输出；不影响后续审计步骤"
        cpp2rust_record_error
    fi
}

# 参数：RUST_PROJECT_DIR
cpp2rust_cargo_test() {
    local rust_project="$1"
    local smoke_file="${rust_project}/tests/smoke.rs"
    if [ ! -f "${smoke_file}" ]; then
        cpp2rust_info "未生成 tests/smoke.rs（可能 init 阶段无 pub class 类型），跳过 cargo test"
        return 0
    fi
    cpp2rust_info "检测到冒烟测试文件：${smoke_file}"
    if (cd "${rust_project}" && cargo test 2>&1); then
        cpp2rust_ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
    else
        # hicc import_lib! 在测试二进制链接时可能遇到 _hicc_export_methods_* 未定义限制；
        # 或冒烟测试调用了未完成的绑定。非致命，仅累计错误。
        cpp2rust_warn "cargo test 失败 — 可能是已知 hicc import_lib! 链接限制或冒烟测试调用未完成绑定"
        cpp2rust_record_error
    fi
}

# ─── § 6. FFI 审计 ──────────────────────────────────────────────────────────
# 参数：RUST_SRC_DIR OBJ_DIR(optional)
# 检查项：
#   ① import_lib! / import_class! 块存在性
#   ② link_name 一致性（不应含路径分隔符 /）
#   ③ struct/class 前缀清理（Rust 绑定中不应多余前缀）
#   ④ restrict 限定符剥离
#   ⑤ nm 符号交叉比对（生成 FFI 函数 vs 目标文件 extern-C 符号）
#   ⑥ 降级标记统计
cpp2rust_ffi_audit() {
    local rust_src="$1"
    local obj_dir="${2:-}"
    if [ ! -d "${rust_src}" ]; then
        cpp2rust_warn "Rust 源码目录不存在：${rust_src}，跳过 FFI 审计"
        return 0
    fi

    # ① import_lib! / import_class! 块存在性
    local import_lib_files import_class_files
    import_lib_files=$( { grep -rl "hicc::import_lib!" "${rust_src}" 2>/dev/null || true; } | wc -l )
    import_class_files=$( { grep -rl "hicc::import_class!" "${rust_src}" 2>/dev/null || true; } | wc -l )
    cpp2rust_info "包含 import_lib! 绑定的文件数：${import_lib_files}"
    cpp2rust_info "包含 import_class! 绑定的文件数：${import_class_files}"
    if [ "${import_lib_files}" -gt 0 ] || [ "${import_class_files}" -gt 0 ]; then
        cpp2rust_ok "生成代码包含 FFI 绑定块 ✓"
    else
        cpp2rust_warn "生成代码中未找到 import_lib! / import_class! 块"
        cpp2rust_record_error
    fi

    # ② link_name 一致性
    local link_names bad_links=0
    link_names=$( { grep -roh '#!\[link_name = "[^"]*"\]' "${rust_src}" 2>/dev/null || true; } \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u || true)
    if [ -n "${link_names}" ]; then
        while IFS= read -r ln; do
            if echo "${ln}" | grep -q '/'; then
                cpp2rust_warn "link_name 含路径分隔符：${ln}"
                bad_links=$((bad_links + 1))
            fi
        done < <(echo "${link_names}")
        if [ "${bad_links}" -eq 0 ]; then
            cpp2rust_ok "所有 link_name 均为纯文件名（一致性 通过）"
        else
            cpp2rust_warn "${bad_links} 个 link_name 含路径分隔符"
            cpp2rust_record_error
        fi
    fi

    # ③ struct/class 前缀清理
    local struct_hits class_hits
    struct_hits=$( { grep -rn '\bstruct \b' "${rust_src}" 2>/dev/null || true; } \
        | grep -cv '^\s*//\|hicc::cpp!' ) || struct_hits=0
    class_hits=$( { grep -rn '\bclass \b' "${rust_src}" 2>/dev/null || true; } \
        | grep -cv '^\s*//\|hicc::cpp!' ) || class_hits=0
    if [ "${struct_hits}" -eq 0 ] && [ "${class_hits}" -eq 0 ]; then
        cpp2rust_ok "Rust 绑定中无多余的 struct/class 前缀"
    else
        cpp2rust_warn "Rust 绑定中仍有 struct/class 前缀：struct=${struct_hits} class=${class_hits}（多见于 import_class! 块内的 pub class 声明，属正常 hicc 语法）"
    fi

    # ④ restrict 限定符
    local restrict_hits
    restrict_hits=$( { grep -rn '__restrict\|[^_]restrict[^_]' "${rust_src}" 2>/dev/null || true; } \
        | grep -cv '^\s*//' ) || restrict_hits=0
    if [ "${restrict_hits}" -eq 0 ]; then
        cpp2rust_ok "Rust 绑定中无 restrict 限定符"
    else
        cpp2rust_warn "Rust 绑定中仍有 restrict 限定符：${restrict_hits} 处"
    fi

    # ⑤ nm 符号交叉比对
    if [ -n "${obj_dir}" ] && [ -d "${obj_dir}" ]; then
        local nm_cache
        nm_cache=$(mktemp)
        # 路径来自 mktemp / SOURCES（无空格/特殊字符），xargs 安全
        # shellcheck disable=SC2038
        { find "${obj_dir}" -name "*.o" 2>/dev/null || true; } \
            | xargs -r nm --defined-only -f posix 2>/dev/null > "${nm_cache}" || true
        local extern_c_count
        extern_c_count=$(grep -c ' T ' "${nm_cache}" 2>/dev/null || echo 0)
        cpp2rust_info "目标文件中 T 段定义符号数：${extern_c_count}"
        if [ "${extern_c_count}" -gt 0 ]; then
            cpp2rust_ok "目标文件包含 extern-C 导出符号"
        fi
        rm -f "${nm_cache}"
    fi

    # ⑥ 降级标记统计
    local todo_count
    todo_count=$( { grep -r "cpp2rust-todo" "${rust_src}" 2>/dev/null || true; } | wc -l | tr -d '[:space:]')
    todo_count="${todo_count:-0}"
    if [ "${todo_count}" -gt 0 ]; then
        cpp2rust_warn "降级标记（需手动完善）：${todo_count} 处"
        { grep -r "cpp2rust-todo" "${rust_src}" 2>/dev/null || true; } \
            | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn || true
    else
        cpp2rust_ok "无降级标记"
    fi
}

# ─── § 7. 最终报告 ──────────────────────────────────────────────────────────
# 参数：LIB_NAME FEATURE CPP2RUST_OUTPUT_DIR [RUST_PROJECT_DIR]
#
# 注：所有 $(...) 内的 find/grep 均显式附加 || true（或 echo 兜底），避免在
#     set -euo pipefail 下因 find 无匹配返回非零而提前中断脚本。
cpp2rust_final_report() {
    local lib_name="$1"
    local feature="$2"
    local cpp2rust_output="$3"
    local rust_project="${4:-${cpp2rust_output}/rust}"
    local rust_src="${rust_project}/src"
    local captured=0 rs_files=0 import_lib_files=0 total_fn=0 todo_count=0

    if [ -d "${cpp2rust_output}/c" ]; then
        captured=$( { find "${cpp2rust_output}/c" -name "*.cpp2rust" 2>/dev/null || true; } | wc -l )
    fi
    if [ -d "${rust_src}" ]; then
        rs_files=$( { find -L "${rust_src}" -name "*.rs" 2>/dev/null || true; } | wc -l )
        import_lib_files=$( { grep -rl "hicc::import_lib!" "${rust_src}" 2>/dev/null || true; } | wc -l )
        total_fn=$( { grep -roh '#\[cpp(func = "[^"]*")\]' "${rust_src}" 2>/dev/null || true; } | wc -l )
        todo_count=$( { grep -r "cpp2rust-todo" "${rust_src}" 2>/dev/null || true; } | wc -l | tr -d '[:space:]')
    fi
    todo_count="${todo_count:-0}"

    echo ""
    echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BOLD}│        cpp2rust-demo 真实库 FFI 验证结果                │${NC}"
    echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
    echo ""
    echo -e "  ${BOLD}项目：${NC}      ${lib_name}"
    echo -e "  ${BOLD}feature：${NC}   ${feature}"
    echo -e "  ${BOLD}输出目录：${NC}  ${cpp2rust_output}"
    echo ""
    echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${captured}"
    echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${rs_files}"
    echo -e "  ${BOLD}import_lib! 文件数：${NC}${import_lib_files}"
    echo -e "  ${BOLD}FFI 函数绑定总数：${NC}  ${total_fn}"
    if [ "${todo_count}" -gt 0 ]; then
        echo -e "  ${YELLOW}⚠ 降级标记：${todo_count} 处${NC}"
    else
        echo -e "  ${GREEN}✓ 无降级标记${NC}"
    fi
    echo ""

    if [ "${rs_files}" -gt 0 ]; then
        echo -e "  ${BOLD}查看生成的 Rust FFI 脚手架：${NC}"
        echo -e "    find ${rust_src} -name '*.rs' | xargs head -80"
        echo ""
    fi
}
