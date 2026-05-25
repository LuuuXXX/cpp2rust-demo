/*
 * C++ AST Generation Hook
 * LD_PRELOAD library for intercepting C++ compilation and generating AST JSON.
 */

#define _GNU_SOURCE
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define MAX_PATH_LEN 8192
#define MAX_CMD_LEN 32768
#define AST_GUARD_ENV "C2RUST_AST_HELPER"

static const char *C2RUST_PROJECT_ROOT = "C2RUST_PROJECT_ROOT";
static const char *C2RUST_FEATURE_ROOT = "C2RUST_FEATURE_ROOT";
static const char *C2RUST_CXX = "C2RUST_CXX";
static const char *C2RUST_DEBUG_ENV = "C2RUST_DEBUG";

static const char *cxx_names[] = {"g++", "clang++", "c++"};

#define DBG(...)             \
    do {                     \
        if (getenv(C2RUST_DEBUG_ENV)) { \
            dprintf(2, "[cpp-hook] " __VA_ARGS__); \
        }                    \
    } while (0)

static inline const char *path_basename(const char *path) {
    if (!path) return path;
    const char *slash = strrchr(path, '/');
    return slash ? slash + 1 : path;
}

static inline int is_name_match(const char *name, const char *pattern) {
    size_t plen = strlen(pattern);
    if (strncmp(name, pattern, plen) != 0) return 0;
    if (name[plen] == '\0') return 1;
    if (name[plen] == '-' && isdigit((unsigned char)name[plen + 1])) return 1;
    return 0;
}

static inline int is_matched(const char *name, const char **names, int len) {
    for (int i = 0; i < len; ++i) {
        if (is_name_match(name, names[i])) return 1;
    }
    return 0;
}

static inline int is_cxx_compiler(const char *name) {
    const char *cxx = getenv(C2RUST_CXX);
    const char *base = path_basename(name);
    if (!base) return 0;
    if (!cxx) {
        int n = (int)(sizeof(cxx_names) / sizeof(cxx_names[0]));
        return is_matched(base, cxx_names, n);
    }
    return strcmp(path_basename(cxx), base) == 0;
}

static char *realpath_from_env(const char *env) {
    const char *path = getenv(env);
    if (!path) return NULL;
    return realpath(path, NULL);
}

static int read_proc_cmdline(char ***out_argv) {
    int fd = open("/proc/self/cmdline", O_RDONLY);
    if (fd < 0) return -1;
    char buf[MAX_CMD_LEN];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) return -1;
    buf[n] = '\0';

    int argc = 0;
    for (ssize_t i = 0; i < n; ++i) {
        if (buf[i] == '\0') argc++;
    }
    if (argc == 0) return -1;

    char **argv = (char **)calloc((size_t)argc + 1, sizeof(char *));
    if (!argv) return -1;
    int idx = 0;
    char *p = buf;
    while (idx < argc && p < buf + n) {
        argv[idx++] = strdup(p);
        p += strlen(p) + 1;
    }
    argv[idx] = NULL;
    *out_argv = argv;
    return idx;
}

static void free_cmdline(char **argv, int argc) {
    for (int i = 0; i < argc; ++i) free(argv[i]);
    free(argv);
}

static inline int is_cppfile(const char *file) {
    size_t len = strlen(file);
    if (len < 4) return 0;
    return strcmp(file + len - 4, ".cpp") == 0 || strcmp(file + len - 4, ".cc") == 0 || strcmp(file + len - 3, ".cxx") == 0;
}

static const char *strip_prefix(const char *path, const char *prefix) {
    size_t prefix_len = strlen(prefix);
    if (strncmp(path, prefix, prefix_len) != 0) return NULL;
    if (prefix[prefix_len - 1] == '/') return path + prefix_len;
    if (path[prefix_len] == '/') return path + prefix_len + 1;
    return NULL;
}

static void ensure_parent_dir(const char *file_path) {
    char tmp[MAX_PATH_LEN];
    snprintf(tmp, sizeof(tmp), "%s", file_path);
    for (char *p = tmp + 1; *p; ++p) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0777);
            *p = '/';
        }
    }
}

static int should_keep_flag(const char *arg) {
    if (!arg || !*arg) return 0;
    if (arg[0] == '\0') return 0;
    if (strcmp(arg, "-c") == 0) return 0;
    if (strcmp(arg, "-shared") == 0) return 0;
    if (strcmp(arg, "-o") == 0) return 0;
    return arg[0] == '-';
}

static void save_options(const char *opts_path, int argc, char *argv[], const char *cfile) {
    ensure_parent_dir(opts_path);
    FILE *out = fopen(opts_path, "w");
    if (!out) return;
    for (int i = 1; i < argc; ++i) {
        const char *arg = argv[i];
        if (!arg) continue;
        if (strcmp(arg, cfile) == 0) continue;
        if (strcmp(arg, "-o") == 0) {
            ++i;
            continue;
        }
        if (should_keep_flag(arg)) {
            fprintf(out, "%s\n", arg);
        }
    }
    fclose(out);
}

static void append_filtered_args(char *buffer, size_t capacity, int argc, char *argv[], const char *cfile) {
    size_t pos = strlen(buffer);
    for (int i = 1; i < argc; ++i) {
        const char *arg = argv[i];
        if (!arg) continue;
        if (strcmp(arg, cfile) == 0) continue;
        if (strcmp(arg, "-o") == 0) {
            ++i;
            continue;
        }
        if (!should_keep_flag(arg)) continue;
        pos += (size_t)snprintf(buffer + pos, capacity - pos, " '%s'", arg);
        if (pos >= capacity) return;
    }
}

static void generate_ast_json(int argc, char *argv[], const char *cfile, const char *project_root,
                              const char *feature_root) {
    const char *path = strip_prefix(cfile, project_root);
    if (!path) {
        DBG("skip %s: not under %s\n", cfile, project_root);
        return;
    }

    char json_path[MAX_PATH_LEN];
    char opts_path[MAX_PATH_LEN];
    snprintf(json_path, sizeof(json_path), "%s/ast/%s.json", feature_root, path);
    snprintf(opts_path, sizeof(opts_path), "%s/ast/%s.opts", feature_root, path);
    ensure_parent_dir(json_path);
    ensure_parent_dir(opts_path);
    save_options(opts_path, argc, argv, cfile);

    char clang_cmd[MAX_CMD_LEN];
    snprintf(clang_cmd, sizeof(clang_cmd), "env -u LD_PRELOAD %s=1 clang++ -Xclang -ast-dump=json -fsyntax-only -std=c++17", AST_GUARD_ENV);
    append_filtered_args(clang_cmd, sizeof(clang_cmd), argc, argv, cfile);
    size_t pos = strlen(clang_cmd);
    snprintf(clang_cmd + pos, sizeof(clang_cmd) - pos, " '%s' > '%s'", cfile, json_path);

    DBG("capturing %s -> %s\n", cfile, json_path);
    pid_t pid = fork();
    if (pid == 0) {
        execl("/bin/sh", "sh", "-c", clang_cmd, (char *)NULL);
        _exit(127);
    }
    if (pid > 0) {
        int status = 0;
        waitpid(pid, &status, 0);
    }
}

static void discover_cppfiles(int argc, char *argv[], const char *project_root, const char *feature_root) {
    for (int i = 1; i < argc; ++i) {
        char *arg = argv[i];
        if (!arg || arg[0] == '-' || !is_cppfile(arg) || access(arg, R_OK) != 0) continue;
        char *real = realpath(arg, NULL);
        if (!real) continue;
        generate_ast_json(argc, argv, real, project_root, feature_root);
        free(real);
    }
}

__attribute__((constructor)) static void cpp_hook(void) {
    if (getenv(AST_GUARD_ENV)) return;

    char *project_root = realpath_from_env(C2RUST_PROJECT_ROOT);
    if (!project_root) return;
    char *feature_root = realpath_from_env(C2RUST_FEATURE_ROOT);
    if (!feature_root) {
        free(project_root);
        return;
    }

    char **argv = NULL;
    int argc = read_proc_cmdline(&argv);
    if (argc > 0 && is_cxx_compiler(argv[0])) {
        discover_cppfiles(argc, argv, project_root, feature_root);
    }

    if (argv) free_cmdline(argv, argc > 0 ? argc : 0);
    free(project_root);
    free(feature_root);
}
