#!/usr/bin/env bash
# =============================================================================
# usage/lib/common.sh
#
# cpp2rust-demo 真实库本地验证脚本的公共函数库。
#
# 设计目标：把 verify-rapidjson-ffi.sh 中可复用的部分下沉为函数，使新增的
#   verify-<lib>.sh 只需声明「库路径 / include / 构建命令 / feature 名」等少量
#   参数，即可复用同一套「环境检查 → 安装工具 → init → merge → build.rs 校验 →
#   cargo check → cargo test → 统计降级标记/绑定 → 结果汇报」的流程。
#
# 使用方式（在 verify-<lib>.sh 中）：
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "${SCRIPT_DIR}/lib/common.sh"
#   c2r_init_env
#   c2r_install_tool
#   c2r_ensure_submodule references/tinyxml2
#   # 直出工作流：声明库参数后调用 c2r_run_direct
#   LIB_DISPLAY="tinyxml2" FEATURE="tinyxml2" \
#   CXX_STD="c++17" \
#   c2r_run_direct
#
# 所有函数遵循「失败累加 SCRIPT_ERRORS、最终由 c2r_finish 统一非零退出」的约定，
# 保证可在 CI / 本地一致地失败退出（配合各脚本 `set -euo pipefail`）。
# =============================================================================

# ─── 颜色 / 日志 ──────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    C2R_RED='\033[0;31m'; C2R_GREEN='\033[0;32m'; C2R_YELLOW='\033[1;33m'
    C2R_CYAN='\033[0;36m'; C2R_BOLD='\033[1m'; C2R_NC='\033[0m'
else
    C2R_RED=''; C2R_GREEN=''; C2R_YELLOW=''; C2R_CYAN=''; C2R_BOLD=''; C2R_NC=''
fi

# 全局错误计数器：非致命但重要的检查失败时递增，脚本末尾由 c2r_finish 统一退出。
SCRIPT_ERRORS="${SCRIPT_ERRORS:-0}"

info()  { echo -e "${C2R_CYAN}[INFO]${C2R_NC}  $*"; }
ok()    { echo -e "${C2R_GREEN}[ OK ]${C2R_NC}  $*"; }
warn()  { echo -e "${C2R_YELLOW}[WARN]${C2R_NC}  $*"; }
fail()  { echo -e "${C2R_RED}[FAIL]${C2R_NC}  $*" >&2; exit 1; }
# 记录一个非致命错误（累加计数，稍后由 c2r_finish 统一退出）。
err()   { echo -e "${C2R_RED}[FAIL]${C2R_NC}  $*" >&2; SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1)); }
step()  { echo -e "\n${C2R_BOLD}══════════════════════════════════════════${C2R_NC}"; \
          echo -e "${C2R_BOLD}  $*${C2R_NC}"; \
          echo -e "${C2R_BOLD}══════════════════════════════════════════${C2R_NC}"; }

need_cmd() { command -v "$1" &>/dev/null || fail "未找到命令：$1  请先安装后重试"; }

# 将路径转换为可被「消费 build.rs 的本地 C++ 工具链」识别的形式。
# 在 MSYS2 / Cygwin 上用 `cygpath -m` 转为带盘符的正斜杠混合路径（D:/a/...），
# 非 Windows 环境无 cygpath，原样返回，保持 Linux 行为不变。
to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

# ─── 仓库根目录探测 ───────────────────────────────────────────────────────────
# 设置 REPO_DIR 为 cpp2rust-demo 仓库根目录（基于调用脚本所在目录向上探测）。
c2r_repo_dir() {
    local caller_dir
    caller_dir="$(cd "$(dirname "${BASH_SOURCE[1]:-$0}")" && pwd)"
    local root
    root="$(git -C "${caller_dir}" rev-parse --show-toplevel 2>/dev/null || echo "${caller_dir}/..")"
    REPO_DIR="$(cd "${root}" && pwd)"
    printf '%s' "${REPO_DIR}"
}

# ─── § 0. 环境检查 ────────────────────────────────────────────────────────────
c2r_init_env() {
    step "§ 0. 环境检查"
    need_cmd git
    need_cmd g++
    need_cmd cargo
    NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    # libclang 检查（cpp2rust-demo 依赖 libclang 解析 AST）。
    if ! pkg-config --exists libclang 2>/dev/null && \
       ! find /usr /lib /usr/local -name "libclang*.so*" 2>/dev/null | grep -q .; then
        warn "未检测到 libclang（可能未安装 libclang-dev）。"
        warn "  Ubuntu/Debian：sudo apt-get install -y clang libclang-dev"
    fi
    ok "环境检查完成（CPU=${NPROC}）"
}

# ─── § 1. 安装 cpp2rust-demo ──────────────────────────────────────────────────
# 支持 SKIP_INSTALL=1：已安装时跳过 cargo install。
c2r_install_tool() {
    step "§ 1. 安装 cpp2rust-demo"
    if [ "${SKIP_INSTALL:-0}" = "1" ] && command -v cpp2rust-demo &>/dev/null; then
        ok "已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）：$(command -v cpp2rust-demo)"
    else
        info "从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…"
        local log
        log=$(mktemp)
        if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked \
                 >"${log}" 2>&1; then
            echo "── cargo install 失败，完整日志：──"
            cat "${log}"; rm -f "${log}"
            fail "cpp2rust-demo 安装失败，请检查上方日志"
        fi
        tail -5 "${log}"; rm -f "${log}"
        ok "cpp2rust-demo 安装完成：$(command -v cpp2rust-demo)"
    fi
    cpp2rust-demo --version 2>/dev/null || true
}

# ─── 子模块检测 / 初始化 ──────────────────────────────────────────────────────
# c2r_ensure_submodule <相对路径>   缺失时尝试 git submodule update --init。
# 返回 0 表示就绪，返回 1 表示缺失（调用方可据此跳过）。
c2r_ensure_submodule() {
    local sub="$1"
    local repo="${REPO_DIR:?REPO_DIR 未设置，请先调用 c2r_repo_dir}"
    if [ -e "${repo}/${sub}" ] && [ -n "$(ls -A "${repo}/${sub}" 2>/dev/null)" ]; then
        return 0
    fi
    warn "子模块未初始化：${sub}，尝试 git submodule update --init …"
    if git -C "${repo}" submodule update --init "${sub}" >/dev/null 2>&1 && \
       [ -n "$(ls -A "${repo}/${sub}" 2>/dev/null)" ]; then
        ok "子模块已初始化：${sub}"
        return 0
    fi
    warn "子模块初始化失败或为空：${sub}"
    return 1
}

# ─── build.rs 校验（方案 A：信任工具自动注入；缺失时仅告警，不就地改写） ──────
c2r_check_build_rs() {
    local build_rs="$1"
    step "§ build.rs 校验"
    if [ ! -f "${build_rs}" ]; then
        warn "未找到 ${build_rs}，跳过 build.rs 校验"
        return 0
    fi
    info "工具生成的 build.rs："
    sed 's/^/    /' "${build_rs}"
    if grep -q 'cc_build.include' "${build_rs}" && grep -q 'cc_build.file' "${build_rs}"; then
        ok "build.rs 已由工具自动注入头路径 + 实现文件（方案 A 生效）"
    else
        info "build.rs 未注入 cc_build.include/file（直出绑定的纯声明驱动常见，属正常）"
    fi
}

# ─── 统计：降级标记 ───────────────────────────────────────────────────────────
# 设置全局 TODO_COUNT，并按 [TAG] 分类打印。
c2r_count_downgrades() {
    local rust_src="$1"
    step "§ 降级标记统计（cpp2rust-todo）"
    TODO_COUNT=0
    if [ -d "${rust_src}" ]; then
        TODO_COUNT=$(grep -r "cpp2rust-todo" "${rust_src}" 2>/dev/null | wc -l | tr -d '[:space:]')
        TODO_COUNT="${TODO_COUNT:-0}"
        if [ "${TODO_COUNT}" -gt 0 ]; then
            grep -r "cpp2rust-todo" "${rust_src}" 2>/dev/null \
                | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn || true
        else
            echo "  （无降级标记）"
        fi
    fi
}

# ─── 统计：import_class! / import_lib! 绑定存在性 ─────────────────────────────
# 设置全局 IMPORT_LIB_FILES / IMPORT_CLASS_FILES / FN_BINDINGS。
c2r_check_import_macros() {
    local rust_src="$1"
    step "§ import_class! / import_lib! 绑定检查"
    IMPORT_LIB_FILES=0; IMPORT_CLASS_FILES=0; FN_BINDINGS=0
    if [ ! -d "${rust_src}" ]; then
        warn "Rust 源码目录不存在：${rust_src}"
        return 0
    fi
    IMPORT_LIB_FILES=$(grep -rl "hicc::import_lib!" "${rust_src}" 2>/dev/null | wc -l | tr -d '[:space:]')
    IMPORT_CLASS_FILES=$(grep -rl "hicc::import_class!" "${rust_src}" 2>/dev/null | wc -l | tr -d '[:space:]')
    FN_BINDINGS=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${rust_src}" 2>/dev/null | wc -l | tr -d '[:space:]')
    info "包含 import_lib! 的文件数：${IMPORT_LIB_FILES}"
    info "包含 import_class! 的文件数：${IMPORT_CLASS_FILES}"
    info "函数绑定数（#[cpp(func=...)]）：${FN_BINDINGS}"
    if [ "${IMPORT_LIB_FILES}" -gt 0 ] || [ "${IMPORT_CLASS_FILES}" -gt 0 ]; then
        ok "生成代码包含 FFI 绑定块（import_class! / import_lib!）✓"
    else
        warn "生成代码中未找到 import_class! / import_lib! 块（纯声明驱动或纯 C 接口时可能为空）"
    fi
}

# ─── nm 符号交叉比对 ──────────────────────────────────────────────────────────
# c2r_nm_cross_check <对象文件目录> <rust_src>
# 把生成的 #[cpp(func=...)] 名与对象文件 nm 符号交叉比对（仅信息性，不计错误）。
c2r_nm_cross_check() {
    local obj_dir="$1" rust_src="$2"
    step "§ nm 符号交叉比对（生成绑定 vs. 对象文件符号）"
    command -v nm &>/dev/null || { info "未找到 nm，跳过符号比对"; return 0; }
    local nm_cache
    nm_cache=$(mktemp)
    find "${obj_dir}" -name "*.o" -exec nm --defined-only -f posix {} + \
        > "${nm_cache}" 2>/dev/null || true
    local funs
    funs=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${rust_src}" 2>/dev/null \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u)
    if [ -n "${funs}" ]; then
        echo "${funs}" | head -15 | while IFS= read -r fname; do
            printf "  %-45s" "${fname}"
            if grep -q "${fname}" "${nm_cache}" 2>/dev/null; then
                echo -e "${C2R_GREEN}✓ 在对象文件中找到${C2R_NC}"
            else
                echo -e "${C2R_YELLOW}? 未直接找到（hicc cpp! 宏展开后才出现，属正常）${C2R_NC}"
            fi
        done || true
    else
        info "未找到 #[cpp(func=...)] 标注（可能全部通过 import_class! 绑定）"
    fi
    rm -f "${nm_cache}"
}

# ─── cargo check ──────────────────────────────────────────────────────────────
# 失败计入 SCRIPT_ERRORS（不立即退出，便于汇报全貌）。设置 CARGO_CHECK_RESULT。
c2r_cargo_check() {
    local rust_project="$1"
    step "§ cargo check（验证生成的 Rust 项目可编译）"
    CARGO_CHECK_RESULT="skip"
    if [ ! -f "${rust_project}/Cargo.toml" ]; then
        warn "未找到 ${rust_project}/Cargo.toml，跳过 cargo check"
        return 0
    fi
    if grep -q 'hicc-std' "${rust_project}/Cargo.toml"; then
        ok "Cargo.toml 含 hicc-std 依赖"
    else
        info "Cargo.toml 未含 hicc-std 依赖（无 STL 绑定时正常）"
    fi
    if (cd "${rust_project}" && cargo check 2>&1); then
        ok "cargo check 通过 ✓"
        CARGO_CHECK_RESULT="pass"
    else
        err "cargo check 失败 — 生成的 FFI 代码存在编译错误"
        CARGO_CHECK_RESULT="fail"
    fi
}

# ─── cargo test（仅当生成了 smoke 测试时） ───────────────────────────────────
c2r_cargo_test() {
    local rust_project="$1"
    step "§ cargo test（验证生成的冒烟测试）"
    local smoke
    if [ -f "${rust_project}/tests/smoke.rs" ]; then
        smoke="${rust_project}/tests/smoke.rs"
    elif [ -f "${rust_project}/tests/smoke_test.rs" ]; then
        smoke="${rust_project}/tests/smoke_test.rs"
    else
        info "未生成 tests/smoke.rs（init 阶段无可往返断言的 pub class），跳过 cargo test"
        return 0
    fi
    info "检测到冒烟测试：${smoke}"
    if (cd "${rust_project}" && cargo test 2>&1); then
        ok "cargo test 通过 ✓"
    else
        warn "cargo test 失败 — 请检查上方链接日志（可能为环境/符号解析问题）"
    fi
}

# ─── § init + merge（直出工作流通用驱动） ────────────────────────────────────
# 调用前需设置：
#   FEATURE        feature 名称
#   LIB_DISPLAY    展示用库名
#   CXX_STD        C++ 标准（默认 c++17）
#   C2R_SRC_FILES  （数组）要编译触发拦截的实现 .cpp（绝对路径）
#   C2R_INCLUDES   （数组）-I 包含路径（绝对路径）
#   C2R_DRIVER_CONTENT （可选）若非空，写入临时驱动 .cpp 并加入编译集（用于 header-only 库）
# 设置全局：CPP2RUST_OUTPUT / RUST_PROJECT / RUST_SRC / CAPTURED / RS_FILES / OBJ_DIR
c2r_init_and_merge() {
    local feature="${FEATURE:?FEATURE 未设置}"
    local std="${CXX_STD:-c++17}"
    local repo="${REPO_DIR:?REPO_DIR 未设置}"

    # 组装编译文件集（实现单元 + 可选驱动）。
    local -a srcs=()
    if [ "${#C2R_SRC_FILES[@]}" -gt 0 ]; then
        srcs+=("${C2R_SRC_FILES[@]}")
    fi
    OBJ_DIR=$(mktemp -d)
    local driver=""
    if [ -n "${C2R_DRIVER_CONTENT:-}" ]; then
        driver="${OBJ_DIR}/${feature}_driver.cpp"
        printf '%s\n' "${C2R_DRIVER_CONTENT}" > "${driver}"
        srcs+=("${driver}")
    fi

    if [ "${#srcs[@]}" -eq 0 ]; then
        fail "c2r_init_and_merge：未提供任何待编译源文件（C2R_SRC_FILES / C2R_DRIVER_CONTENT 均为空）"
    fi

    # 组装 -I 参数。
    local -a inc_args=()
    local inc
    for inc in "${C2R_INCLUDES[@]:-}"; do
        [ -n "${inc}" ] && inc_args+=("-I${inc}")
    done

    # ── § 3. 预编译对象文件（供 nm 符号比对，可选） ──────────────────────────
    step "§ 预编译对象文件（供 nm 符号比对）"
    local src obj
    for src in "${srcs[@]}"; do
        obj="${OBJ_DIR}/$(basename "${src}" | sed 's/\.[^.]*$//').o"
        if g++ -c -std="${std}" "${inc_args[@]}" "${src}" -o "${obj}" 2>"${OBJ_DIR}/compile.log"; then
            info "编译成功：$(basename "${obj}")"
        else
            warn "编译失败：$(basename "${src}")（不影响 init 拦截，仅 nm 比对可能不完整）"
            sed 's/^/    /' "${OBJ_DIR}/compile.log" >&2 || true
        fi
    done

    # ── § 4. cpp2rust-demo init（编译拦截 + FFI 脚手架生成） ─────────────────
    step "§ cpp2rust-demo init（捕获 FFI 脚手架）"
    local build_script
    build_script=$(mktemp)
    {
        echo '#!/usr/bin/env bash'
        echo 'set -e'
        for src in "${srcs[@]}"; do
            printf 'g++ -c -std="%s"' "${std}"
            for inc in "${inc_args[@]}"; do printf ' "%s"' "${inc}"; done
            printf ' "%s" -o /dev/null\n' "${src}"
        done
    } > "${build_script}"
    chmod +x "${build_script}"

    info "feature：${feature}    C++ 标准：${std}"
    info "构建命令：bash ${build_script}"

    cd "${repo}" || fail "无法进入仓库目录：${repo}"
    cpp2rust-demo init --feature "${feature}" -- bash "${build_script}"
    rm -f "${build_script}"
    ok "cpp2rust-demo init 完成"

    CPP2RUST_OUTPUT="${repo}/.cpp2rust/${feature}"
    CAPTURED=$(find "${CPP2RUST_OUTPUT}/c" -name "*.cpp2rust" 2>/dev/null | wc -l | tr -d '[:space:]')
    info "捕获预处理文件数：${CAPTURED}"

    # ── § 5. cpp2rust-demo merge ────────────────────────────────────────────
    step "§ cpp2rust-demo merge（整理输出结构）"
    cd "${repo}" || fail "无法进入仓库目录：${repo}"
    cpp2rust-demo merge --feature "${feature}"
    ok "merge 完成"

    RUST_PROJECT="${CPP2RUST_OUTPUT}/rust"
    RUST_SRC="${RUST_PROJECT}/src"
    RS_FILES=$(find -L "${RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l | tr -d '[:space:]')
    info "生成 Rust 文件数：${RS_FILES}"
}

# ─── 一站式直出验证（init+merge → 校验 → check/test → 统计） ──────────────────
c2r_run_direct() {
    c2r_init_and_merge
    c2r_check_build_rs "${RUST_PROJECT}/build.rs"
    c2r_count_downgrades "${RUST_SRC}"
    c2r_check_import_macros "${RUST_SRC}"
    c2r_nm_cross_check "${OBJ_DIR}" "${RUST_SRC}"
    c2r_cargo_check "${RUST_PROJECT}"
    c2r_cargo_test "${RUST_PROJECT}"
    rm -rf "${OBJ_DIR}" 2>/dev/null || true
    c2r_report
}

# ─── § 结果汇报 ───────────────────────────────────────────────────────────────
c2r_report() {
    step "§ 生成结果汇报"
    echo ""
    echo -e "  ${C2R_BOLD}项目：${C2R_NC}             ${LIB_DISPLAY:-${FEATURE}}"
    echo -e "  ${C2R_BOLD}feature：${C2R_NC}          ${FEATURE}"
    echo -e "  ${C2R_BOLD}输出目录：${C2R_NC}         ${CPP2RUST_OUTPUT:-?}"
    echo -e "  ${C2R_BOLD}捕获预处理文件数：${C2R_NC} ${CAPTURED:-0}"
    echo -e "  ${C2R_BOLD}生成 Rust 文件数：${C2R_NC} ${RS_FILES:-0}"
    echo -e "  ${C2R_BOLD}import_class! 文件：${C2R_NC} ${IMPORT_CLASS_FILES:-0}"
    echo -e "  ${C2R_BOLD}import_lib! 文件：${C2R_NC}   ${IMPORT_LIB_FILES:-0}"
    echo -e "  ${C2R_BOLD}函数绑定数：${C2R_NC}       ${FN_BINDINGS:-0}"
    echo -e "  ${C2R_BOLD}降级标记数：${C2R_NC}       ${TODO_COUNT:-0}"
    echo -e "  ${C2R_BOLD}cargo check：${C2R_NC}      ${CARGO_CHECK_RESULT:-skip}"
    echo ""
}

# ─── 统一退出 ─────────────────────────────────────────────────────────────────
# 若 SCRIPT_ERRORS>0 则以非零退出，确保 CI/本地一致失败。
c2r_finish() {
    if [ "${SCRIPT_ERRORS:-0}" -gt 0 ]; then
        fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
    fi
    ok "验证脚本执行完毕，未发现错误 ✓"
}
