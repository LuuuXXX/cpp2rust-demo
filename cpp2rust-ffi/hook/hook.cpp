/**
 * hook.cpp - C++ Compiler Interceptor for LD_PRELOAD
 *
 * 通过 LD_PRELOAD 拦截 C++ 编译器调用，对每个 .cpp 源文件执行预处理，
 * 输出宏展开后的代码（.c2rust 文件），用于后续 AST 解析。
 *
 * 基于 hook.c（预处理逻辑）和 cpp-hook/hook.cpp（C++ 编译器检测）合并。
 *
 * 环境变量：
 *   C2RUST_FEATURE_ROOT  捕获产物输出目录（必填）
 *   C2RUST_PROJECT_ROOT  C++ 项目根目录（必填）
 *   C2RUST_CXX           C++ 编译器路径（可选，默认自动检测）
 *   C2RUST_DEBUG         设为 1 时输出调试日志
 */

#define _GNU_SOURCE
#include <dlfcn.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <stdarg.h>
#include <errno.h>
#include <libgen.h>
#include <limits.h>

// 函数指针类型
typedef int (*execvp_fn)(const char *file, char *const argv[]);
typedef int (*execve_fn)(const char *pathname, char *const argv[], char *const envp[]);

// 支持的 C++ 编译器列表（包含版本化变体）
static const char *CXX_COMPILERS[] = {
    "g++", "clang++", "c++",
    "g++-10", "g++-11", "g++-12", "g++-13", "g++-14",
    "clang++-13", "clang++-14", "clang++-15", "clang++-16",
    "clang++-17", "clang++-18",
    NULL
};

// 支持的 C++ 文件扩展名
static const char *CXX_EXTENSIONS[] = {
    ".cpp", ".cc", ".cxx", ".c++", ".C", ".cp", NULL
};

static int g_debug = 0;

static void debug_log(const char *fmt, ...) {
    if (!g_debug) return;
    va_list ap;
    va_start(ap, fmt);
    fprintf(stderr, "[c2rust-hook] ");
    vfprintf(stderr, fmt, ap);
    fprintf(stderr, "\n");
    va_end(ap);
}

/**
 * 检查文件名是否是 C++ 源文件
 */
static int is_cxx_source(const char *filename) {
    if (!filename) return 0;
    const char *dot = strrchr(filename, '.');
    if (!dot) return 0;
    for (int i = 0; CXX_EXTENSIONS[i]; i++) {
        if (strcmp(dot, CXX_EXTENSIONS[i]) == 0) return 1;
    }
    return 0;
}

/**
 * 检查程序名是否是 C++ 编译器
 */
static int is_cxx_compiler(const char *prog) {
    if (!prog) return 0;
    // 取 basename
    const char *base = strrchr(prog, '/');
    base = base ? base + 1 : prog;
    for (int i = 0; CXX_COMPILERS[i]; i++) {
        if (strcmp(base, CXX_COMPILERS[i]) == 0) return 1;
    }
    return 0;
}

/**
 * 从 argv 中找到第一个 C++ 源文件
 */
static const char *find_source_file(char *const argv[]) {
    for (int i = 1; argv[i]; i++) {
        if (is_cxx_source(argv[i])) return argv[i];
    }
    return NULL;
}

/**
 * 创建目录（递归）
 */
static int mkdirs(const char *path) {
    char tmp[PATH_MAX];
    snprintf(tmp, sizeof(tmp), "%s", path);
    size_t len = strlen(tmp);
    if (len > 0 && tmp[len - 1] == '/') tmp[len - 1] = '\0';

    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0755);
            *p = '/';
        }
    }
    return mkdir(tmp, 0755);
}

/**
 * 提取 argv 中的 -I 和 -D 参数（用于预处理）
 */
static void collect_preprocessor_args(char *const argv[], char ***out_args, int *out_count) {
    int count = 0;
    // 先数数量
    for (int i = 1; argv[i]; i++) {
        if (strncmp(argv[i], "-I", 2) == 0 ||
            strncmp(argv[i], "-D", 2) == 0 ||
            strncmp(argv[i], "-std=", 5) == 0 ||
            strncmp(argv[i], "-include", 8) == 0) {
            count++;
            if (strlen(argv[i]) == 2) count++; // 参数在下一个 argv
        }
    }

    char **args = (char **)calloc(count + 1, sizeof(char *));
    int idx = 0;

    for (int i = 1; argv[i]; i++) {
        if (strncmp(argv[i], "-I", 2) == 0 ||
            strncmp(argv[i], "-D", 2) == 0 ||
            strncmp(argv[i], "-std=", 5) == 0) {
            args[idx++] = argv[i];
            if ((strncmp(argv[i], "-I", 2) == 0 || strncmp(argv[i], "-D", 2) == 0)
                && strlen(argv[i]) == 2 && argv[i + 1]) {
                args[idx++] = argv[i + 1];
            }
        } else if (strcmp(argv[i], "-include") == 0 && argv[i + 1]) {
            args[idx++] = argv[i];
            args[idx++] = argv[i + 1];
        }
    }
    args[idx] = NULL;
    *out_args = args;
    *out_count = idx;
}

/**
 * 运行预处理，将结果写入 output_path
 * 使用 -E -C -P 选项（展开宏，保留注释，不输出行号）
 */
static int run_preprocess(
    const char *cxx,
    const char *source_file,
    char *const extra_args[],
    int extra_count,
    const char *output_path
) {
    // 构建预处理命令
    int argc = 0;
    char **args = (char **)calloc(extra_count + 20, sizeof(char *));

    args[argc++] = (char *)cxx;
    args[argc++] = (char *)"-E";
    args[argc++] = (char *)"-C";
    args[argc++] = (char *)"-P";
    args[argc++] = (char *)"-x";
    args[argc++] = (char *)"c++";

    // 添加额外参数（-I, -D 等）
    for (int i = 0; i < extra_count; i++) {
        args[argc++] = extra_args[i];
    }

    args[argc++] = (char *)source_file;
    args[argc++] = (char *)"-o";
    args[argc++] = (char *)output_path;
    args[argc] = NULL;

    debug_log("Running preprocess: %s -> %s", source_file, output_path);

    pid_t pid = fork();
    if (pid == 0) {
        // 子进程
        execvp(cxx, args);
        perror("execvp");
        _exit(1);
    } else if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        free(args);
        return WIFEXITED(status) ? WEXITSTATUS(status) : -1;
    } else {
        free(args);
        return -1;
    }
}

/**
 * 核心：拦截 execvp 调用
 */
int execvp(const char *file, char *const argv[]) {
    static execvp_fn real_execvp = NULL;
    if (!real_execvp) {
        real_execvp = (execvp_fn)dlsym(RTLD_NEXT, "execvp");
    }

    // 初始化调试标志
    static int initialized = 0;
    if (!initialized) {
        const char *debug_env = getenv("C2RUST_DEBUG");
        g_debug = debug_env && strcmp(debug_env, "1") == 0;
        initialized = 1;
    }

    debug_log("execvp intercepted: %s", file ? file : "(null)");

    // 检查是否是 C++ 编译器
    if (!is_cxx_compiler(file)) {
        // 也检查环境变量 C2RUST_CXX
        const char *env_cxx = getenv("C2RUST_CXX");
        if (!env_cxx || !is_cxx_compiler(file)) {
            goto passthrough;
        }
    }

    {
        // 获取环境变量
        const char *feature_root = getenv("C2RUST_FEATURE_ROOT");
        const char *project_root = getenv("C2RUST_PROJECT_ROOT");

        if (!feature_root || !project_root) {
            debug_log("C2RUST_FEATURE_ROOT or C2RUST_PROJECT_ROOT not set, skipping");
            goto passthrough;
        }

        // 找到源文件
        const char *source_file = find_source_file(argv);
        if (!source_file) {
            debug_log("No C++ source file found in args");
            goto passthrough;
        }

        debug_log("Intercepted compilation of: %s", source_file);

        // 获取 C++ 编译器（优先用环境变量）
        const char *cxx = getenv("C2RUST_CXX");
        if (!cxx) cxx = file;

        // 获取源文件的相对路径
        char source_rel[PATH_MAX];
        if (source_file[0] == '/') {
            // 绝对路径：尝试计算相对于 project_root 的路径
            size_t pr_len = strlen(project_root);
            if (strncmp(source_file, project_root, pr_len) == 0) {
                snprintf(source_rel, sizeof(source_rel), "%s", source_file + pr_len);
                if (source_rel[0] == '/') memmove(source_rel, source_rel + 1, strlen(source_rel));
            } else {
                snprintf(source_rel, sizeof(source_rel), "%s", source_file);
            }
        } else {
            snprintf(source_rel, sizeof(source_rel), "%s", source_file);
        }

        // 构建输出路径：feature_root/c/<source_rel>.c2rust
        char output_path[PATH_MAX * 2];
        snprintf(output_path, sizeof(output_path), "%s/c/%s.c2rust", feature_root, source_rel);

        // 创建输出目录
        char output_dir[PATH_MAX * 2];
        snprintf(output_dir, sizeof(output_dir), "%s", output_path);
        char *last_slash = strrchr(output_dir, '/');
        if (last_slash) {
            *last_slash = '\0';
            mkdirs(output_dir);
        }

        // 收集预处理参数
        char **pp_args = NULL;
        int pp_count = 0;
        collect_preprocessor_args(argv, &pp_args, &pp_count);

        // 运行预处理
        int ret = run_preprocess(cxx, source_file, pp_args, pp_count, output_path);
        if (ret != 0) {
            debug_log("Preprocessing failed for %s (exit %d)", source_file, ret);
        } else {
            debug_log("Preprocessed: %s -> %s", source_file, output_path);
        }

        free(pp_args);

        // 更新 targets.list
        char targets_path[PATH_MAX * 2];
        snprintf(targets_path, sizeof(targets_path), "%s/targets.list", feature_root);
        FILE *f = fopen(targets_path, "a");
        if (f) {
            fprintf(f, "%s\n", output_path);
            fclose(f);
        }
    }

passthrough:
    return real_execvp(file, argv);
}
