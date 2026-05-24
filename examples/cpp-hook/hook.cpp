/*
 * C++ AST Generation Hook
 * LD_PRELOAD library for intercepting C++ compilation and generating AST JSON
 *
 * Usage:
 *   C2RUST_PROJECT_ROOT=/path/to/project \
 *   C2RUST_FEATURE_ROOT=/path/to/output \
 *   LD_PRELOAD=/path/to/hook.so <c++-compiler> <args>
 */

#define _GNU_SOURCE
#include <errno.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/file.h>
#include <sys/wait.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>

#define MAX_PATH_LEN 8192
#define MAX_CMD_LEN 16384

static const char* C2RUST_PROJECT_ROOT = "C2RUST_PROJECT_ROOT";
static const char* C2RUST_FEATURE_ROOT = "C2RUST_FEATURE_ROOT";
static const char* C2RUST_CXX = "C2RUST_CXX";
static const char* C2RUST_DEBUG_ENV = "C2RUST_DEBUG";

static const char* cxx_names[] = {"g++", "clang++", "c++"};

/* Debug logging */
#define DBG(...) do { \
        if (getenv(C2RUST_DEBUG_ENV)) { \
                dprintf(2, "[cpp-hook] " __VA_ARGS__); \
        } \
} while (0)

static inline const char* path_basename(const char* path) {
        if (!path) return path;
        const char* slash = strrchr(path, '/');
        return slash ? slash + 1 : path;
}

static inline int is_name_match(const char* name, const char* pattern) {
        size_t plen = strlen(pattern);
        if (strncmp(name, pattern, plen) != 0) return 0;
        if (name[plen] == '\0') return 1;
        if (name[plen] == '-' && isdigit((unsigned char)name[plen + 1])) return 1;
        return 0;
}

static inline int is_matched(const char* name, const char** names, int len) {
        for (int i = 0; i < len; ++i) {
                if (is_name_match(name, names[i])) {
                        return 1;
                }
        }
        return 0;
}

static inline int is_cxx_compiler(const char* name) {
        const char* cxx = getenv(C2RUST_CXX);
        const char* base = path_basename(name);
        if (!base) return 0;
        if (!cxx) {
                int n = (int)(sizeof(cxx_names) / sizeof(cxx_names[0]));
                return is_matched(base, cxx_names, n);
        } else {
                return strcmp(path_basename(cxx), base) == 0;
        }
}

static char* path_from(const char* env) {
        const char* path = getenv(env);
        if (!path) return NULL;
        return realpath(path, NULL);
}

static int read_proc_cmdline(char*** out_argv) {
        int fd = open("/proc/self/cmdline", O_RDONLY);
        if (fd < 0) return -1;

        char buf[MAX_CMD_LEN];
        ssize_t n = read(fd, buf, sizeof(buf) - 1);
        close(fd);
        if (n <= 0) return -1;
        buf[n] = '\0';

        int argc = 0;
        for (ssize_t i = 0; i < n; i++) {
                if (buf[i] == '\0') argc++;
        }
        if (argc == 0) return -1;

        char** argv = malloc((size_t)(argc + 1) * sizeof(char*));
        if (!argv) return -1;

        int idx = 0;
        char* p = buf;
        while (idx < argc && p < buf + n) {
                argv[idx++] = strdup(p);
                p += strlen(p) + 1;
        }
        argv[idx] = NULL;

        *out_argv = argv;
        return idx;
}

static void free_cmdline(char** argv, int argc) {
        for (int i = 0; i < argc; i++) free(argv[i]);
        free(argv);
}

static inline int is_cppfile(const char* file) {
        int len = strlen(file);
        return len > 4 && strcmp(&file[len - 4], ".cpp") == 0;
}

static const char* strip_prefix(const char* path, const char* prefix) {
        int prefix_len = strlen(prefix);
        if (strncmp(path, prefix, prefix_len) != 0) return NULL;
        if (prefix[prefix_len - 1] == '/') return &path[prefix_len];
        if (path[prefix_len] == '/') return &path[prefix_len + 1];
        return NULL;
}

static void generate_ast_json(const char* cxx, int argc, char* argv[], const char* cfile,
                              const char* project_root, const char* feature_root) {
        const char* path = strip_prefix(cfile, project_root);
        if (!path) {
                DBG("strip_prefix: skipping %s (not under %s)\n", cfile, project_root);
                return;
        }

        char full_path[MAX_PATH_LEN];
        int base_len = snprintf(full_path, sizeof(full_path), "%s/ast/%s", feature_root, path);
        if (base_len >= (int)sizeof(full_path)) return;

        char* filename = strrchr(full_path, '/');
        if (!filename) return;
        *filename = 0;

        char cmd[MAX_CMD_LEN];
        int cmd_len = snprintf(cmd, sizeof(cmd), "mkdir -p \"%s\"", full_path);
        if (cmd_len >= MAX_CMD_LEN) return;
        system(cmd);
        *filename = '/';

        // Change extension to .json
        char* dot = strrchr(full_path, '.');
        if (dot && dot > filename) {
                strcpy(dot, ".json");
        } else {
                full_path[base_len] = '\0';
                strcat(full_path, ".json");
        }

        DBG("generating AST JSON: %s -> %s\n", cfile, full_path);

        // Build clang command for AST dump
        char clang_cmd[MAX_CMD_LEN];
        int pos = 0;
        pos += snprintf(clang_cmd + pos, sizeof(clang_cmd) - pos, "clang++ -Xclang -ast-dump=json -fsyntax-only -std=c++17");
        for (int i = 0; i < argc; i++) {
                pos += snprintf(clang_cmd + pos, sizeof(clang_cmd) - pos, " %s", argv[i]);
        }
        pos += snprintf(clang_cmd + pos, sizeof(clang_cmd) - pos, " %s > \"%s\"", cfile, full_path);

        DBG("running: %s\n", clang_cmd);

        pid_t pid = fork();
        if (pid == 0) {
                FILE* f = popen(clang_cmd, "r");
                if (f) pclose(f);
                _exit(0);
        } else if (pid != -1) {
                int wstatus;
                waitpid(pid, &wstatus, 0);
        }
}

static void discover_cppfile(int argc, char* argv[], const char* project_root, const char* feature_root) {
        if (argc <= 0) return;

        char** cfiles = NULL;
        int count = 0;

        for (int i = 1; i < argc; ++i) {
                char* arg = argv[i];
                if (arg[0] != '-' && is_cppfile(arg) && access(arg, R_OK) == 0) {
                        char* real = realpath(arg, NULL);
                        if (real) {
                                if (!cfiles) cfiles = malloc(argc * sizeof(char*));
                                cfiles[count++] = real;
                        }
                }
        }

        if (count == 0) {
                DBG("no .cpp files found\n");
                return;
        }

        for (int i = 0; i < count; i++) {
                generate_ast_json(argv[0], argc, argv, cfiles[i], project_root, feature_root);
                free(cfiles[i]);
        }
        free(cfiles);
}

__attribute__((constructor)) static void cpp_hook(void) {
        char* project_root = NULL;
        char* feature_root = NULL;
        char** argv = NULL;
        int argc = -1;

        project_root = path_from(C2RUST_PROJECT_ROOT);
        if (!project_root) return;

        feature_root = path_from(C2RUST_FEATURE_ROOT);
        if (!feature_root) goto fail;

        argc = read_proc_cmdline(&argv);
        if (argc < 1) goto fail;

        DBG("invoked as %s (argc=%d)\n", argv[0], argc);

        if (is_cxx_compiler(argv[0])) {
                discover_cppfile(argc, argv, project_root, feature_root);
        }

fail:
        if (argv) free_cmdline(argv, argc > 0 ? argc : 0);
        if (project_root) free(project_root);
        if (feature_root) free(feature_root);
}
