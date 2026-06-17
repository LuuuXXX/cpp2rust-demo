# =============================================================================
# verify-common.sh —— per-library 验证脚本的共享骨架（被 source，不直接执行）
#
# 用途：
#   将 usage/verify-rapidjson-ffi.sh 的七阶段骨架中「与具体库无关」的通用逻辑
#   下沉为可复用函数，供各 usage/verify-<lib>-ffi.sh 瘦封装脚本 source 复用，
#   避免 7 份脚本复制粘贴。Linux 上的行为与现有 rapidjson 脚本逐字节兼容。
#
# 七阶段骨架（与 rapidjson 脚本一致）：
#   § 0. 环境检查           → vc_check_env
#   § 1. 安装 cpp2rust-demo  → vc_install_tool（含 SKIP_INSTALL）
#   § 2. 定位 / 准备源       → vc_locate_sources（子模块自动 init、header-only 生成驱动）
#   § 3. 编译源目标文件      → vc_compile_sources（供后续 nm 符号验证）
#   § 4. cpp2rust-demo init  → vc_init（拦截构建 → 捕获 .cpp2rust）
#   § 5. cpp2rust-demo merge → vc_merge
#   § 5a. 校验 build.rs       → vc_check_build_rs（方案 A 校验 + 方案 B 兜底）
#   § 5b. cargo check        → vc_cargo_check
#   § 5c. cargo test         → vc_cargo_test
#   § 6. FFI 验证            → vc_verify_ffi（import_lib!/import_class!/link_name/#include）
#   § 7. 生成结果汇报        → vc_report
#
# per-library 脚本只需声明以下环境变量，然后 `source usage/lib/verify-common.sh`
# 并调用 `vc_run`：
#   LIB_NAME      库名（如 tinyxml2），用于显示与 Cargo crate 名
#   FEATURE       cpp2rust-demo feature 名（默认 <LIB_NAME>_ffi）
#   SUBMODULE     references/<lib> 子模块路径（缺失时自动 git submodule update --init）
#   SOURCES       带真实实现的库：相对子模块根的 .cpp/.cc 源（空格分隔）
#   INCLUDES      头文件包含目录（相对子模块根，空格分隔；为空则用子模块根）
#   HEADER_ONLY   置 1 表示 header-only 库（用 DRIVER_CPP 生成最小驱动 .cpp）
#   DRIVER_CPP    header-only 库的驱动 .cpp 内容（include 库头触发解析）
#   CXX_STD       C++ 标准（默认 c++17）
#   PROJECT_LABEL 汇报中显示的项目描述（默认 <LIB_NAME>）
#
# 系统要求（Ubuntu/Debian）：
#   sudo apt-get install -y clang libclang-dev g++ libstdc++-14-dev binutils git curl
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# =============================================================================

set -euo pipefail

# ─── 颜色 / 日志辅助 ──────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ OK ]${NC}  $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*" >&2; exit 1; }
step()  { echo -e "\n${BOLD}══════════════════════════════════════════${NC}"; \
          echo -e "${BOLD}  $*${NC}"; \
          echo -e "${BOLD}══════════════════════════════════════════${NC}"; }

need_cmd() { command -v "$1" &>/dev/null || fail "未找到命令：$1  请先安装后重试"; }

# 将路径转换为可被「消费 build.rs 的本地 C++ 工具链」识别的形式。
# MSYS2 / Cygwin 上用 `cygpath -m` 转为带盘符的正斜杠混合路径（D:/a/...），
# 非 Windows 环境无 cygpath，原样返回，保持 Linux 行为不变。
to_build_path() {
    if command -v cygpath &>/dev/null; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

# 全局错误计数器：结构性失败（init/merge 失败、未生成任何 FFI 绑定）时递增，
# 脚本末尾统一 exit 1（供 verify-all / CI 捕获）。
SCRIPT_ERRORS=0

# STRICT_CARGO=1 时，cargo check / cargo test 失败也计入 SCRIPT_ERRORS（CI 严格模式）。
# 默认 0：真实库可能触发工具尚未实现的 codegen 路径（抽象类工厂、opaque 模板等），
# 此类 cargo check/test 失败仅 warn 并记录到汇报，不中断、不致整脚本失败。
STRICT_CARGO="${STRICT_CARGO:-0}"

# cargo 各阶段结果（OK / FAIL / SKIP），由对应阶段填充，供 § 7 汇报。
VC_CARGO_CHECK="SKIP"
VC_CARGO_TEST="SKIP"

# ─── 内部状态（由各阶段函数填充） ────────────────────────────────────────────
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
SKIP_INSTALL="${SKIP_INSTALL:-0}"
TMPDIR="${TMPDIR:-/tmp}"

VC_REPO_DIR=""          # 仓库根目录
VC_SUBMODULE_DIR=""     # 子模块绝对路径
VC_WORK_DIR=""          # 扁平临时工程根（init 在此运行，保证 unit 名为纯文件名）
VC_SOURCE_PATHS=()      # 待编译/拦截的源文件绝对路径（位于 VC_WORK_DIR，扁平）
VC_INCLUDE_PATHS=()     # 头文件包含目录绝对路径
VC_OBJ_DIR=""           # 编译目标文件输出目录
VC_OUTPUT_DIR=""        # .cpp2rust/<FEATURE>
VC_RUST_PROJECT=""      # 生成的 Rust 项目目录
VC_RUST_SRC=""          # 生成的 Rust 源码目录
VC_NM_CACHE=""          # nm 符号缓存文件
VC_CAPTURED=0           # 捕获 .cpp2rust 文件数
VC_RS_FILES=0           # 生成 .rs 文件数
VC_COMPILE_ERRORS=0     # 源文件编译失败数

# =============================================================================
# § 0. 环境检查
# =============================================================================
vc_check_env() {
    step "§ 0. 环境检查"

    need_cmd git
    need_cmd g++
    need_cmd cargo
    need_cmd nm

    if ! pkg-config --exists libclang 2>/dev/null && \
       ! find /usr /lib /usr/local -name "libclang*.so*" 2>/dev/null | grep -q .; then
        warn "未检测到 libclang（可能未安装 libclang-dev）。cpp2rust-demo 依赖 libclang 解析 AST。"
        warn "  Ubuntu/Debian：sudo apt-get install -y clang libclang-dev"
    fi

    ok "环境检查完成"
}

# =============================================================================
# § 1. 安装 cpp2rust-demo
# =============================================================================
vc_install_tool() {
    step "§ 1. 安装 cpp2rust-demo"

    if [ "${SKIP_INSTALL}" = "1" ] && command -v cpp2rust-demo &>/dev/null; then
        ok "已检测到 cpp2rust-demo，跳过安装（SKIP_INSTALL=1）"
    else
        info "从 GitHub 源码安装 cpp2rust-demo（首次编译需要几分钟）…"
        INSTALL_LOG=$(mktemp)
        if ! cargo install --git https://github.com/LuuuXXX/cpp2rust-demo --locked \
                 >"${INSTALL_LOG}" 2>&1; then
            echo "── cargo install 失败，完整日志：──"
            cat "${INSTALL_LOG}"
            rm -f "${INSTALL_LOG}"
            fail "cpp2rust-demo 安装失败，请检查上方日志"
        fi
        tail -5 "${INSTALL_LOG}"
        rm -f "${INSTALL_LOG}"
        ok "cpp2rust-demo 安装完成：$(which cpp2rust-demo)"
    fi

    cpp2rust-demo --version 2>/dev/null || true
}

# =============================================================================
# § 2. 定位 / 准备源文件（子模块自动 init；header-only 生成驱动）
# =============================================================================
vc_locate_sources() {
    step "§ 2. 定位 / 准备 ${LIB_NAME} 源文件"

    # 从脚本所在目录向上找仓库根目录
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[1]:-${BASH_SOURCE[0]}}")" && pwd)"
    VC_REPO_DIR="$(git -C "${script_dir}" rev-parse --show-toplevel 2>/dev/null || echo "${script_dir}/..")"
    VC_REPO_DIR="$(cd "${VC_REPO_DIR}" && pwd)"

    VC_SUBMODULE_DIR="${VC_REPO_DIR}/${SUBMODULE:-}"

    # 系统头文件库（如 sqlite3）：不依赖子模块，改为校验系统头文件存在。
    # 沿用 tests/sqlite3_e2e_test.rs 的处理方式（驱动 #include <sqlite3.h>，链接系统库）。
    if [ "${SKIP_SUBMODULE:-0}" = "1" ]; then
        if [ -n "${REQUIRE_SYSTEM_HEADER:-}" ] && [ ! -f "${REQUIRE_SYSTEM_HEADER}" ]; then
            warn "系统头文件不存在：${REQUIRE_SYSTEM_HEADER}，跳过 ${LIB_NAME} 验证（不中断）"
            warn "  Ubuntu/Debian 安装示例：sudo apt-get install -y libsqlite3-dev"
            exit 0
        fi
        VC_SUBMODULE_DIR="${VC_REPO_DIR}"
        VC_INCLUDE_PATHS=()
        VC_SOURCE_PATHS=()
        VC_WORK_DIR=$(mktemp -d)
        local driver_cpp="${VC_WORK_DIR}/${LIB_NAME}_driver.cpp"
        printf '%s\n' "${DRIVER_CPP}" > "${driver_cpp}"
        VC_SOURCE_PATHS+=("${driver_cpp}")
        info "系统头文件库：已生成最小驱动 ${driver_cpp}"
        info "扁平工程根：${VC_WORK_DIR}"
        info "源文件数量：${#VC_SOURCE_PATHS[@]}"
        for s in "${VC_SOURCE_PATHS[@]}"; do echo "  - ${s}"; done
        ok "${LIB_NAME} 源文件就绪（系统头文件模式）"
        return
    fi

    # 子模块未初始化则自动 init（与 Makefile `submodules` 目标一致），失败则跳过整库
    if [ ! -d "${VC_SUBMODULE_DIR}" ] || [ -z "$(ls -A "${VC_SUBMODULE_DIR}" 2>/dev/null)" ]; then
        warn "子模块未初始化：${SUBMODULE}，尝试 git submodule update --init ${SUBMODULE}"
        if ! git -C "${VC_REPO_DIR}" submodule update --init "${SUBMODULE}" 2>&1; then
            warn "子模块初始化失败：${SUBMODULE}，跳过 ${LIB_NAME} 验证（不中断）"
            exit 0
        fi
    fi

    if [ ! -d "${VC_SUBMODULE_DIR}" ] || [ -z "$(ls -A "${VC_SUBMODULE_DIR}" 2>/dev/null)" ]; then
        warn "子模块目录仍为空：${VC_SUBMODULE_DIR}，跳过 ${LIB_NAME} 验证（不中断）"
        exit 0
    fi

    # 计算 include 路径（相对子模块根 → 绝对路径；为空则用子模块根）
    VC_INCLUDE_PATHS=()
    if [ -n "${INCLUDES:-}" ]; then
        for inc in ${INCLUDES}; do
            VC_INCLUDE_PATHS+=("${VC_SUBMODULE_DIR}/${inc}")
        done
    else
        VC_INCLUDE_PATHS+=("${VC_SUBMODULE_DIR}")
    fi

    # 扁平临时工程根：init 在此运行 → 捕获的 .cpp2rust 为纯文件名（无子目录），
    # 避免 unit 名/link_name 含路径分隔符（"/"）污染 hicc 生成的 C++。
    # 同时把 .cpp2rust 产物隔离在 /tmp，保持仓库与子模块工作树干净。
    VC_WORK_DIR=$(mktemp -d)

    VC_SOURCE_PATHS=()
    if [ "${HEADER_ONLY:-0}" = "1" ]; then
        # header-only 库：生成最小驱动 .cpp（include 库头触发解析压测）
        local driver_cpp="${VC_WORK_DIR}/${LIB_NAME}_driver.cpp"
        printf '%s\n' "${DRIVER_CPP}" > "${driver_cpp}"
        VC_SOURCE_PATHS+=("${driver_cpp}")
        info "header-only 库：已生成最小驱动 ${driver_cpp}"
    else
        # 带真实实现 .cpp 的库：将库自身编译单元复制到扁平工程根后拦截。
        # 头文件仍经 -I 指向子模块真实目录解析，复制单文件不影响 #include。
        for src_rel in ${SOURCES}; do
            local src_abs="${VC_SUBMODULE_DIR}/${src_rel}"
            if [ ! -f "${src_abs}" ]; then
                warn "源文件不存在：${src_abs}（跳过该源）"
                continue
            fi
            local dst="${VC_WORK_DIR}/$(basename "${src_abs}")"
            cp "${src_abs}" "${dst}"
            VC_SOURCE_PATHS+=("${dst}")
        done
        if [ "${#VC_SOURCE_PATHS[@]}" -eq 0 ]; then
            warn "未找到任何 ${LIB_NAME} 源文件，跳过验证（不中断）"
            exit 0
        fi
    fi

    info "子模块目录：${VC_SUBMODULE_DIR}"
    info "扁平工程根：${VC_WORK_DIR}"
    info "源文件数量：${#VC_SOURCE_PATHS[@]}"
    for s in "${VC_SOURCE_PATHS[@]}"; do echo "  - ${s}"; done
    info "include 路径：${VC_INCLUDE_PATHS[*]}"

    ok "${LIB_NAME} 源文件就绪"
}

# 拼接 -I 选项数组
_vc_include_flags() {
    local flags=()
    for inc in "${VC_INCLUDE_PATHS[@]}"; do
        flags+=("-I${inc}")
    done
    if [ "${#flags[@]}" -gt 0 ]; then
        printf '%s\n' "${flags[@]}"
    fi
}

# =============================================================================
# § 3. 编译源目标文件（供 nm 符号验证）
# =============================================================================
vc_compile_sources() {
    step "§ 3. 编译 ${LIB_NAME} 目标文件"

    VC_OBJ_DIR=$(mktemp -d)
    info "目标文件输出目录：${VC_OBJ_DIR}"

    # 注册 trap 清理临时目录
    trap 'rm -rf "${VC_OBJ_DIR:-}" "${VC_NM_CACHE:-}" "${VC_WORK_DIR:-}" 2>/dev/null || true' EXIT

    local inc_flags
    mapfile -t inc_flags < <(_vc_include_flags)

    VC_COMPILE_ERRORS=0
    local idx=0
    for src in "${VC_SOURCE_PATHS[@]}"; do
        idx=$((idx + 1))
        local name
        name="$(basename "${src}")"
        name="${name%.*}_${idx}"
        local obj="${VC_OBJ_DIR}/${name}.o"
        local compile_log="${VC_OBJ_DIR}/${name}.log"
        if g++ -c -std="${CXX_STD}" "${inc_flags[@]}" \
               "${src}" -o "${obj}" >"${compile_log}" 2>&1; then
            info "编译成功：${name}.o"
        else
            warn "编译失败：$(basename "${src}")"
            cat "${compile_log}" >&2
            VC_COMPILE_ERRORS=$((VC_COMPILE_ERRORS + 1))
        fi
    done

    if [ "${VC_COMPILE_ERRORS}" -gt 0 ]; then
        warn "${VC_COMPILE_ERRORS} 个源文件编译失败，nm 验证可能不完整"
    else
        ok "全部 ${#VC_SOURCE_PATHS[@]} 个源文件编译成功"
    fi
}

# =============================================================================
# § 4. cpp2rust-demo init — 编译拦截 & Rust FFI 脚手架生成
# =============================================================================
vc_init() {
    step "§ 4. cpp2rust-demo init（捕获 FFI 脚手架）"

    # 创建临时构建脚本：g++ -c 编译每个源，触发 LD_PRELOAD hook 拦截。
    # 在扁平工程根（VC_WORK_DIR）内以纯文件名编译，使捕获的 .cpp2rust 不含子目录。
    local build_script
    build_script=$(mktemp)
    {
        echo "#!/bin/bash"
        echo "set -e"
        printf 'cd "%s"\n' "${VC_WORK_DIR}"
        for src in "${VC_SOURCE_PATHS[@]}"; do
            printf 'g++ -c -std="%s"' "${CXX_STD}"
            for inc in "${VC_INCLUDE_PATHS[@]}"; do
                printf ' -I"%s"' "${inc}"
            done
            printf ' "%s" -o /dev/null 2>&1\n' "$(basename "${src}")"
        done
    } > "${build_script}"
    chmod +x "${build_script}"

    info "扁平工程根（项目根）：${VC_WORK_DIR}"
    info "feature 名称：${FEATURE}"
    info "构建命令：bash ${build_script}"

    # 非交互模式：cpp2rust-demo 检测 stdin 不是 TTY 时自动全选所有被拦截的 .cpp 文件。
    # 从 VC_WORK_DIR 运行 → find_project_root 取 VC_WORK_DIR → 捕获路径为纯文件名。
    cd "${VC_WORK_DIR}"
    cpp2rust-demo init \
        --feature "${FEATURE}" \
        -- bash "${build_script}"

    rm -f "${build_script}"
    ok "cpp2rust-demo init 完成"

    VC_OUTPUT_DIR="${VC_WORK_DIR}/.cpp2rust/${FEATURE}"
    info "输出目录：${VC_OUTPUT_DIR}"

    VC_CAPTURED=$(find "${VC_OUTPUT_DIR}/c" -name "*.cpp2rust" 2>/dev/null | wc -l)
    info "捕获预处理文件数：${VC_CAPTURED}"
}

# =============================================================================
# § 5. cpp2rust-demo merge — 整理输出目录
# =============================================================================
vc_merge() {
    step "§ 5. cpp2rust-demo merge（整理输出结构）"

    cd "${VC_WORK_DIR}"
    cpp2rust-demo merge --feature "${FEATURE}"
    ok "merge 完成"

    VC_RUST_PROJECT="${VC_OUTPUT_DIR}/rust"
    VC_RUST_SRC="${VC_RUST_PROJECT}/src"
    VC_RS_FILES=$(find -L "${VC_RUST_SRC}" -name "*.rs" 2>/dev/null | wc -l)
    info "生成 Rust 文件数：${VC_RS_FILES}"
    if [ "${VC_RS_FILES}" -gt 0 ]; then
        echo "──── 生成的 .rs 文件（前 20 条）────"
        find -L "${VC_RUST_SRC}" -name "*.rs" | sort | head -20 || true
    fi

    echo ""
    info "降级标记统计（cpp2rust-todo）："
    grep -r "cpp2rust-todo" "${VC_RUST_SRC}" 2>/dev/null \
        | grep -oE '\[[^]]*\]' | sort | uniq -c | sort -rn \
        || echo "  （无降级标记）"
}

# =============================================================================
# § 5a. 校验 / 兜底生成项目的 build.rs
#   · 工具产物已自包含（含 cc_build.include + cc_build.file）→ 信任（方案 A）。
#   · 未自包含（旧版工具 / 未捕获 .opts）→ 退回脚本就地补全（方案 B 兜底）。
# =============================================================================
vc_check_build_rs() {
    step "§ 5a. 校验 build.rs（方案 A：工具自动注入头路径与实现）"

    local build_rs="${VC_RUST_PROJECT}/build.rs"
    local lib_crate="${LIB_NAME//-/_}"

    if [ ! -f "${build_rs}" ]; then
        warn "未找到 ${build_rs}，跳过 build.rs 校验/补全"
        return
    fi

    info "工具生成的 build.rs："
    sed 's/^/    /' "${build_rs}"

    local build_meta="${VC_OUTPUT_DIR}/meta/build-meta.json"
    if [ -f "${build_meta}" ]; then
        info "编译元数据 meta/build-meta.json（方案 A 落盘）："
        sed 's/^/    /' "${build_meta}"
    fi

    if grep -q 'cc_build.include' "${build_rs}" && grep -q 'cc_build.file' "${build_rs}"; then
        ok "build.rs 已由工具自动注入头路径 + 实现（方案 A 生效，跳过就地补全）"
        return
    fi

    warn "工具 build.rs 未注入头/实现路径（可能未捕获 .opts），退回脚本就地补全（方案 B 兜底）"

    # 收集所有含 hicc 宏的单元 .rs（排除 lib.rs / mod.rs），生成 .rust_file 行
    local rust_file_lines=""
    while IFS= read -r rs; do
        local rel="${rs#"${VC_RUST_PROJECT}"/}"
        rust_file_lines="${rust_file_lines}        .rust_file(\"${rel}\")
"
    done < <(find -L "${VC_RUST_PROJECT}/src" -name "*.rs" \
                 ! -name "lib.rs" ! -name "mod.rs" | sort)

    # include 路径转换为本地工具链可识别的形式（Windows: D:/a/...）
    local include_lines=""
    for inc in "${VC_INCLUDE_PATHS[@]}"; do
        local inc_bp
        inc_bp="$(to_build_path "${inc}")"
        include_lines="${include_lines}    cc_build.include(\"${inc_bp}\");
"
    done

    # 仅带真实实现 .cpp 的库注入源文件以提供 extern \"C\" 实现符号；
    # header-only 库的驱动方法仅声明，无需（也无法）注入实现单元。
    local source_lines=""
    if [ "${HEADER_ONLY:-0}" != "1" ]; then
        for src in "${VC_SOURCE_PATHS[@]}"; do
            local src_bp
            src_bp="$(to_build_path "${src}")"
            source_lines="${source_lines}    cc_build.file(\"${src_bp}\");
"
        done
    fi

    cat > "${build_rs}" << EOF
fn main() {
    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    // 与源编译保持一致的 C++ 标准（脚本 CXX_STD=${CXX_STD}）
    cc_build.std("${CXX_STD}");
    // ① 头文件包含路径（hicc 生成的胶水 C++ #include 需要）
${include_lines}    // ② 编译库实现单元，提供 extern "C" 实现符号（header-only 库为空）
${source_lines}
    // 逐文件注册含 hicc 宏的单元，生成 _hicc_export_methods_* 导出函数
    build
${rust_file_lines}        .compile("${lib_crate}");

    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
}
EOF

    info "兜底补全后的 build.rs："
    sed 's/^/    /' "${build_rs}"
    ok "build.rs 已就地补全（注入头路径 + 实现 + 逐文件注册）"
}

# =============================================================================
# § 5b. cargo check — 验证生成的 Rust 项目可编译
# =============================================================================
vc_cargo_check() {
    step "§ 5b. cargo check（验证生成的 Rust 项目语法与类型正确）"

    if [ ! -f "${VC_RUST_PROJECT}/Cargo.toml" ]; then
        warn "未找到 ${VC_RUST_PROJECT}/Cargo.toml，跳过 cargo check"
        return
    fi

    info "在 ${VC_RUST_PROJECT} 中运行 cargo check ..."

    echo "──── Cargo.toml hicc-std 依赖检查 ────"
    if grep -q 'hicc-std' "${VC_RUST_PROJECT}/Cargo.toml"; then
        ok "Cargo.toml 已包含 hicc-std 依赖"
    else
        warn "Cargo.toml 未找到 hicc-std 依赖（可能影响 STL 类型绑定）"
    fi
    cat "${VC_RUST_PROJECT}/Cargo.toml"

    if (cd "${VC_RUST_PROJECT}" && cargo check 2>&1); then
        ok "cargo check 通过 ✓"
        VC_CARGO_CHECK="OK"
    else
        warn "cargo check 失败 — 生成的 FFI 代码存在编译错误（可能触发工具尚未实现的 codegen 路径）"
        VC_CARGO_CHECK="FAIL"
        if [ "${STRICT_CARGO}" = "1" ]; then SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1)); fi
    fi
}

# =============================================================================
# § 5c. cargo test — 验证生成的冒烟测试通过
# =============================================================================
vc_cargo_test() {
    step "§ 5c. cargo test（验证生成的冒烟测试可编译、可链接、可运行）"

    local smoke_file="${VC_RUST_PROJECT}/tests/smoke.rs"
    if [ -f "${smoke_file}" ]; then
        info "检测到冒烟测试文件：${smoke_file}"
        if (cd "${VC_RUST_PROJECT}" && cargo test 2>&1); then
            ok "cargo test 通过 ✓（生成的冒烟测试全部通过）"
            VC_CARGO_TEST="OK"
        else
            warn "cargo test 失败 — 请检查上方链接日志（可能触发工具尚未实现的 codegen 路径）"
            VC_CARGO_TEST="FAIL"
            if [ "${STRICT_CARGO}" = "1" ]; then SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1)); fi
        fi
    else
        info "未生成 tests/smoke.rs（可能 init 阶段无 pub class 类型），跳过 cargo test"
        VC_CARGO_TEST="SKIP"
    fi
}

# =============================================================================
# § 6. FFI 验证（import_lib! / import_class! / link_name 一致性 / #include 探测）
# =============================================================================
vc_verify_ffi() {
    step "§ 6. FFI 验证"

    # ── 6a. 验证源目标文件中的导出符号 ─────────────────────────────────────────
    echo -e "\n${BOLD}6a. 源目标文件导出符号（nm 验证）${NC}"
    VC_NM_CACHE=$(mktemp)
    find "${VC_OBJ_DIR}" -name "*.o" 2>/dev/null \
        | xargs -r nm --defined-only -f posix 2>/dev/null > "${VC_NM_CACHE}" || true

    local extern_c_count
    extern_c_count=$(grep -c ' T ' "${VC_NM_CACHE}" 2>/dev/null || true)
    extern_c_count=${extern_c_count:-0}
    info "源目标文件中 T 段定义符号数：${extern_c_count}"
    if [ "${extern_c_count}" -gt 0 ]; then
        echo "──── 部分符号（前 20 条，T 段）────"
        awk '$2 == "T" { print $1 }' "${VC_NM_CACHE}" | head -20 || true
        ok "源文件包含导出符号"
    else
        info "未找到 T 段符号（header-only 驱动仅声明方法属正常）"
    fi

    # ── 6b. 验证生成 Rust 代码中的 FFI 声明 ──────────────────────────────────────
    echo -e "\n${BOLD}6b. 生成 Rust 代码中的 FFI 声明（import_lib! / import_class!）${NC}"
    if [ ! -d "${VC_RUST_SRC}" ]; then
        warn "Rust 源码目录不存在：${VC_RUST_SRC}"
        return
    fi

    local import_lib_files import_class_files
    import_lib_files=$(grep -rl "hicc::import_lib!" "${VC_RUST_SRC}" 2>/dev/null | wc -l || true)
    import_class_files=$(grep -rl "hicc::import_class!" "${VC_RUST_SRC}" 2>/dev/null | wc -l || true)
    info "包含 import_lib! 绑定的文件数：${import_lib_files}"
    info "包含 import_class! 绑定的文件数：${import_class_files}"
    if [ "${import_lib_files}" -gt 0 ] || [ "${import_class_files}" -gt 0 ]; then
        ok "生成代码包含 import_lib! / import_class! 块（FFI 绑定存在）✓"
    else
        info "生成代码中未找到 import_lib! / import_class! 块（可能仅生成 cpp! 头块）"
    fi

    # ── 6c. link_name 一致性检查（不应含路径分隔符 /）─────────────────────────
    echo ""
    echo "──── link_name 一致性检查（不应含路径分隔符 /）────"
    local link_names
    link_names=$(grep -roh '#!\[link_name = "[^"]*"\]' "${VC_RUST_SRC}" 2>/dev/null \
        | grep -oE '"[^"]*"' | tr -d '"' | sort -u || true)
    if [ -n "${link_names}" ]; then
        local bad_links=0
        while IFS= read -r ln; do
            if echo "${ln}" | grep -q '/'; then
                echo -e "  ${RED}✗ link_name 含路径分隔符：${ln}${NC}"
                bad_links=$((bad_links + 1))
            else
                echo -e "  ${GREEN}✓ ${ln}${NC}"
            fi
        done < <(echo "${link_names}")
        if [ "${bad_links}" -eq 0 ]; then
            ok "所有 link_name 均为纯文件名（link_name 一致性 通过）"
        else
            warn "${bad_links} 个 link_name 含路径分隔符，请检查提取器输出"
        fi
    else
        info "未找到 #![link_name = ...] 声明（可能无 import_lib! 块）"
    fi

    # ── 6d. cpp! 块 #include 探测 ──────────────────────────────────────────────
    echo ""
    echo "──── cpp! 块 #include 探测 ────"
    local hdr_includes
    hdr_includes=$(grep -rh '#include' "${VC_RUST_SRC}" 2>/dev/null | grep -v '//' | wc -l || true)
    info "生成代码中 #include 指令数：${hdr_includes}"
    if [ "${hdr_includes}" -gt 0 ]; then
        ok "cpp! 块包含头文件 include（头文件探测路径已生效）"
        grep -rh '#include' "${VC_RUST_SRC}" 2>/dev/null | grep -v '//' | sort -u | head -10 || true
    else
        info "cpp! 块无 #include 指令（可能未探测到对应头文件）"
    fi
}

# =============================================================================
# § 7. 生成结果汇报
# =============================================================================
vc_report() {
    step "§ 7. 生成结果汇报"

    echo ""
    echo -e "${BOLD}┌─────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BOLD}│             cpp2rust-demo FFI 验证结果                  │${NC}"
    echo -e "${BOLD}└─────────────────────────────────────────────────────────┘${NC}"
    echo ""
    echo -e "  ${BOLD}项目：${NC}      ${PROJECT_LABEL:-${LIB_NAME}}"
    echo -e "  ${BOLD}feature：${NC}   ${FEATURE}"
    echo -e "  ${BOLD}子模块：${NC}    ${VC_SUBMODULE_DIR}"
    echo -e "  ${BOLD}输出目录：${NC}  ${VC_OUTPUT_DIR}"
    echo ""
    echo -e "  ${BOLD}捕获预处理文件数：${NC}  ${VC_CAPTURED}"
    echo -e "  ${BOLD}生成 Rust 文件数：${NC}  ${VC_RS_FILES}"
    echo ""

    if [ -d "${VC_RUST_SRC}" ]; then
        local import_lib_files import_class_files total_fn_bindings
        import_lib_files=$(grep -rl "hicc::import_lib!" "${VC_RUST_SRC}" 2>/dev/null | wc -l || true)
        import_class_files=$(grep -rl "hicc::import_class!" "${VC_RUST_SRC}" 2>/dev/null | wc -l || true)
        total_fn_bindings=$(grep -roh '#\[cpp(func = "[^"]*")\]' "${VC_RUST_SRC}" 2>/dev/null | wc -l || true)
        import_lib_files=$(echo "${import_lib_files}" | tr -d '[:space:]')
        import_class_files=$(echo "${import_class_files}" | tr -d '[:space:]')
        total_fn_bindings=$(echo "${total_fn_bindings}" | tr -d '[:space:]')
        echo -e "  ${BOLD}import_lib! FFI 绑定文件数：${NC}  ${import_lib_files}"
        echo -e "  ${BOLD}import_class! 绑定文件数：${NC}    ${import_class_files}"
        echo -e "  ${BOLD}FFI 函数绑定总数：${NC}          ${total_fn_bindings}"

        # 结构性判定：至少生成一处 import_lib! 或 import_class! 才算成功捕获 FFI。
        # ALLOW_NO_FFI=1（如 sqlite3 纯 C 接口，工具可能不生成绑定）时仅告警不计错。
        if [ "${import_lib_files:-0}" -eq 0 ] && [ "${import_class_files:-0}" -eq 0 ]; then
            if [ "${ALLOW_NO_FFI:-0}" = "1" ]; then
                echo -e "  ${YELLOW}⚠ 未生成 FFI 绑定（纯 C 接口预期内，仅告警）${NC}"
            else
                echo -e "  ${RED}✗ 未生成任何 FFI 绑定（结构性失败）${NC}"
                SCRIPT_ERRORS=$((SCRIPT_ERRORS + 1))
            fi
        fi
    fi

    # cargo check / cargo test 结果矩阵（真实库可能触发工具尚未实现的 codegen 路径）
    _vc_status_line() {
        case "$1" in
            OK)   echo -e "${GREEN}✓ 通过${NC}" ;;
            FAIL) echo -e "${YELLOW}⚠ 失败（已记录，详见上方日志）${NC}" ;;
            *)    echo -e "${CYAN}- 跳过${NC}" ;;
        esac
    }
    echo -e "  ${BOLD}cargo check：${NC}  $(_vc_status_line "${VC_CARGO_CHECK}")"
    echo -e "  ${BOLD}cargo test：${NC}   $(_vc_status_line "${VC_CARGO_TEST}")"
    if [ "${STRICT_CARGO}" = "1" ]; then
        echo -e "  ${CYAN}（STRICT_CARGO=1：cargo check/test 失败计入错误）${NC}"
    fi

    local todo_count
    todo_count=$(grep -r "cpp2rust-todo" "${VC_RUST_SRC}" 2>/dev/null | wc -l | tr -d '[:space:]') || todo_count=0
    todo_count="${todo_count:-0}"
    if [ "${todo_count}" -gt 0 ]; then
        echo -e "  ${YELLOW}⚠ 降级标记（需手动完善）：${todo_count} 处${NC}"
        echo "  → 搜索 'cpp2rust-todo' 查看详情：grep -rn cpp2rust-todo ${VC_RUST_SRC}"
    else
        echo -e "  ${GREEN}✓ 无降级标记${NC}"
    fi

    echo ""
    ok "${LIB_NAME} 验证脚本执行完毕！"

    if [ "${SCRIPT_ERRORS}" -gt 0 ]; then
        fail "验证脚本发现 ${SCRIPT_ERRORS} 个错误，请检查上方 [FAIL] / [WARN] 输出并修复"
    fi
}

# =============================================================================
# vc_run —— 按七阶段骨架顺序执行全部阶段（per-library 脚本调用入口）
# =============================================================================
vc_run() {
    : "${LIB_NAME:?per-library 脚本必须设置 LIB_NAME}"
    if [ "${SKIP_SUBMODULE:-0}" != "1" ]; then
        : "${SUBMODULE:?per-library 脚本必须设置 SUBMODULE}"
    fi
    FEATURE="${FEATURE:-${LIB_NAME//-/_}_ffi}"
    CXX_STD="${CXX_STD:-c++17}"

    vc_check_env
    vc_install_tool
    vc_locate_sources
    vc_compile_sources
    vc_init
    vc_merge
    vc_check_build_rs
    vc_cargo_check
    vc_cargo_test
    vc_verify_ffi
    vc_report
}
