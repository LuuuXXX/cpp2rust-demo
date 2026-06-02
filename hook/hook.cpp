/*
 * 相关环境变量定义:
 * 1. CPP2RUST_PROJECT_ROOT: 工程的根目录，必须存在.
 * 2. CPP2RUST_FEATURE_ROOT: 构建的每个target都对应一个Feature, 必须存在
 * 3. CPP2RUST_CC: 编译程序的名字，如果不指定，则为g++/clang++/c++之一.
 * 4. CPP2RUST_DEBUG: 设置为1时将调试信息输出到stderr.
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/file.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#include <vector>

#define MAX_PATH_LEN 8192
#define MAX_CMD_LEN 16384

static const char* CPP2RUST_PROJECT_ROOT = "CPP2RUST_PROJECT_ROOT";
static const char* CPP2RUST_FEATURE_ROOT = "CPP2RUST_FEATURE_ROOT";
static const char* CPP2RUST_CC = "CPP2RUST_CC";
static const char* CPP2RUST_LD = "CPP2RUST_LD";
static const char* CPP2RUST_CC_SKIP = "CPP2RUST_CC_SKIP";
static const char* CPP2RUST_LD_SKIP = "CPP2RUST_LD_SKIP";
static const char* CPP2RUST_DEBUG_ENV = "CPP2RUST_DEBUG";

static const char* cc_names[] = {"g++", "clang++", "c++"};
static const char* ld_names[] = {"ld", "lld"};

#define DBG(...) do { \
        if (getenv(CPP2RUST_DEBUG_ENV)) { \
                dprintf(2, "[cpp2rust-hook] " __VA_ARGS__); \
        } \
} while (0)

static const char* path_basename(const char* path) {
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

static inline int is_compiler(const char* name) {
        const char* cc = getenv(CPP2RUST_CC);
        const char* base = path_basename(name);
        if (!base) return 0;
        if (!cc) {
                int n = (int)(sizeof(cc_names) / sizeof(cc_names[0]));
                int r = is_matched(base, cc_names, n);
                DBG("is_compiler: base='%s' list-match=%d (checked %d names)\n", base, r, n);
                return r;
        } else {
                int r = strcmp(path_basename(cc), base) == 0;
                DBG("is_compiler: base='%s' env-match=%d (CPP2RUST_CC='%s')\n", base, r, cc);
                return r;
        }
}

static inline int is_linker(const char* name) {
        const char* ld = getenv(CPP2RUST_LD);
        const char* base = path_basename(name);
        if (!base) return 0;
        if (!ld) {
                int n = (int)(sizeof(ld_names) / sizeof(ld_names[0]));
                int r = is_matched(base, ld_names, n);
                DBG("is_linker: base='%s' list-match=%d (checked %d names)\n", base, r, n);
                return r;
        } else {
                int r = strcmp(path_basename(ld), base) == 0;
                DBG("is_linker: base='%s' env-match=%d (CPP2RUST_LD='%s')\n", base, r, ld);
                return r;
        }
}

static inline char* path_from(const char* env) {
        const char* path = getenv(env);
        if (!path) {
                return 0;
        }
        return realpath(path, 0);
}

static int read_proc_cmdline(char*** out_argv) {
        int fd = open("/proc/self/cmdline", O_RDONLY);
        if (fd < 0) {
                DBG("open /proc/self/cmdline failed: errno=%d\n", errno);
                return -1;
        }

        char buf[MAX_CMD_LEN];
        ssize_t n = read(fd, buf, sizeof(buf) - 1);
        close(fd);
        if (n <= 0) {
                DBG("read /proc/self/cmdline returned %zd errno=%d\n", n, errno);
                return -1;
        }
        buf[n] = '\0';

        int argc = 0;
        for (ssize_t i = 0; i < n; i++) {
                if (buf[i] == '\0') argc++;
        }
        if (argc == 0) return -1;

        char** argv = static_cast<char**>(malloc((size_t)(argc + 1) * sizeof(char*)));
        if (!argv) {
                DBG("malloc failed for argv (%d entries)\n", argc);
                return -1;
        }

        int idx = 0;
        char* p = buf;
        char* end = buf + n;
        while (idx < argc && p < end) {
                char* s = strdup(p);
                if (!s) {
                        DBG("strdup failed at arg %d\n", idx);
                        for (int i = 0; i < idx; i++) free(argv[i]);
                        free(argv);
                        return -1;
                }
                argv[idx++] = s;
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

static inline int has_suffix(const char* file, const char* suffix) {
        size_t flen = strlen(file);
        size_t slen = strlen(suffix);
        return flen > slen && strcmp(file + flen - slen, suffix) == 0;
}

static inline int is_cppfile(const char* file) {
        static const char* suffixes[] = {".cpp", ".cc", ".cxx", ".c++", ".C", ".cp"};
        for (size_t i = 0; i < sizeof(suffixes) / sizeof(suffixes[0]); ++i) {
                if (has_suffix(file, suffixes[i])) {
                        return 1;
                }
        }
        return 0;
}

static int parse_args(int argc, char* argv[], char* extracted[], char* cfiles[]) {
    int cnt = 0;
    for (int i = 1; i < argc; ++i) {
        char* arg = argv[i];
        if (arg[0] != '-') {
                if (is_cppfile(arg) && access(arg, R_OK) == 0) {
                    *cfiles = realpath(arg, 0);
                    ++cfiles;
                }
                continue;
        }
        if (arg[1] == 'I' || arg[1] == 'D' || arg[1] == 'U') {
                if (arg[2]) {
                        extracted[cnt++] = arg;
                } else {
                        extracted[cnt++] = arg;
                        ++i;
                        if (i < argc) {
                                extracted[cnt++] = argv[i];
                        }
                }
        } else if (strcmp(&arg[1], "include") == 0 || strcmp(&arg[1], "isystem") == 0 || strcmp(&arg[1], "iquote") == 0) {
                extracted[cnt++] = arg;
                ++i;
                if (i < argc) {
                    extracted[cnt++] = argv[i];
                }
        } else if (strncmp(&arg[1], "std=", 4) == 0 || strcmp(&arg[1], "fshort-enums") == 0) {
                extracted[cnt++] = arg;
        }
    }

    return cnt;
}

static const char* strip_prefix(const char* path, const char* prefix) {
        int prefix_len = strlen(prefix);
        if (strncmp(path, prefix, prefix_len)) {
               return 0;
        } else if (prefix[prefix_len - 1] == '/') {
                return &path[prefix_len];
        } else if (path[prefix_len] == '/') {
                return &path[prefix_len + 1];
        } else {
                return 0;
        }
}

static void save_options(const char* path, int argc, char* argv[]) {
        int fd = open(path, O_CREAT | O_TRUNC | O_WRONLY, 0644);
        if (fd == -1) {
                return;
        }
        for (int i = 0; i < argc; ++i) {
            dprintf(fd, "\"%s\" ", argv[i]);
        }
        close(fd);
}

static void preprocess_cppfile(const char* cc, int argc, char* argv[], const char* cppfile, const char* project_root, const char* feature_root) {
        const char* path = strip_prefix(cppfile, project_root);
        if (!path) {
                DBG("strip_prefix: skipping %s (not under %s)\n", cppfile, project_root);
                return;
        }

        char full_path[MAX_PATH_LEN];
        int full_path_len = snprintf(full_path, sizeof(full_path), "%s/c/%s.cpp2rust", feature_root, path);
        if (full_path_len >= (int)sizeof(full_path)) return;

        char* filename = strrchr(full_path, '/');
        if (!filename) return;

        *filename = 0;
        char cmd[MAX_CMD_LEN];
        int cmd_len = snprintf(cmd, sizeof(cmd), "mkdir -p \"%s\"", full_path);
        if (cmd_len >= MAX_CMD_LEN) return;
        system(cmd);
        *filename = '/';

        int len = snprintf(&full_path[full_path_len], sizeof(full_path) - full_path_len, ".opts");
        if (len < (int)(sizeof(full_path) - full_path_len)) {
                save_options(full_path, argc, argv);
        }
        full_path[full_path_len] = 0;

        DBG("preprocessing %s -> %s\n", cppfile, full_path);

        pid_t pid = fork();
        if (pid == 0) {
            std::vector<char*> new_argv;
            new_argv.reserve((size_t)argc + 8);
            new_argv.push_back(const_cast<char*>(cc));
            new_argv.push_back(const_cast<char*>("-E"));
            new_argv.push_back(const_cast<char*>("-C"));
            new_argv.push_back(const_cast<char*>(cppfile));
            new_argv.push_back(const_cast<char*>("-o"));
            new_argv.push_back(full_path);
            for (int i = 0; i < argc; ++i) {
                    new_argv.push_back(argv[i]);
            }
            new_argv.push_back(nullptr);
            execvp(cc, new_argv.data());
            _exit(127);
        } else if (pid != -1) {
                int wstatus = 0;
                waitpid(pid, &wstatus, 0);
                if (!WIFEXITED(wstatus) || WEXITSTATUS(wstatus) != 0) {
                        DBG("preprocessing failed for %s: exit=%d\n",
                            cppfile, WIFEXITED(wstatus) ? WEXITSTATUS(wstatus) : -1);
                }
        } else {
                DBG("fork failed: errno=%d\n", errno);
        }
}

static void discover_cppfile(int argc, char* argv[], const char* project_root, const char* feature_root) {
        if (argc <= 0) return;

        int cnt = 0;
        char** cflags = static_cast<char**>(calloc((size_t)argc, sizeof(char*)));
        char** cfiles = static_cast<char**>(calloc((size_t)argc, sizeof(char*)));
        if (!cflags || !cfiles) {
                DBG("calloc failed for cflags/cfiles\n");
                goto fail;
        }

        if (getenv(CPP2RUST_CC_SKIP)) goto fail;

        cnt = parse_args(argc, argv, cflags, cfiles);
        if (!cfiles[0]) {
                DBG("no C++ files found in argv[0..%d]\n", argc - 1);
                goto fail;
        }

        setenv(CPP2RUST_CC_SKIP, "1", 0);

        for (int i = 0; i < argc; ++i) {
                const char* file = cfiles[i];
                if (!file) break;
                preprocess_cppfile(argv[0], cnt, cflags, file, project_root, feature_root);
        }
fail:
        if (cfiles) {
                for (char** cf = cfiles; *cf; ++cf) free(*cf);
                free(cfiles);
        }
        if (cflags) free(cflags);
}

char* get_file(char* path) {
        char* deli = strrchr(path, '/');
        return deli ? deli + 1 : path;
}

char* get_static_lib(char* path, const char* project_root) {
        char* real_path = realpath(path, 0);
        if (!real_path) return 0;
        const char* tmp = strip_prefix(real_path, project_root);
        free(real_path);
        if (!tmp) return 0;

        char* lib = get_file(path);
        if (strncmp(lib, "lib", 3) != 0) return 0;
        int len = strlen(lib);
        if (len > 5 && strcmp(&lib[len - 2], ".a") != 0) return 0;
        return lib;
}

static void target_save(char* libs[], int cnt, const char* feature_root) {
        if (cnt == 0) return;

        char buf[MAX_CMD_LEN];
        char* content = 0;
        ssize_t content_len = 0;

        setenv(CPP2RUST_LD_SKIP, "1", 0);

        int len = snprintf(buf, MAX_CMD_LEN, "mkdir -p %s/c", feature_root);
        if (len >= MAX_CMD_LEN) {
                dprintf(2, "command is too long: %s...\n", buf);
                return;
        }
        system(buf);

        len = snprintf(buf, MAX_CMD_LEN, "%s/c/targets.list", feature_root);
        if (len >= MAX_CMD_LEN) {
                dprintf(2, "path is too long: %s...\n", buf);
                return;
        }

        int fd = open(buf, O_CREAT | O_RDWR, 0666);
        if (fd == -1) {
                dprintf(2, "failed to open file: %s...\n", buf);
                return;
        }

        if (flock(fd, LOCK_EX) != 0) {
                dprintf(2, "failed to lock file: %s, errno = %d\n", buf, errno);
                goto fail;
        }

        content = &buf[len + 1];
        content_len = read(fd, content, MAX_CMD_LEN - len - 1);
        if (content_len == -1) {
                dprintf(2, "failed to read file: %s, errno = %d\n", buf, errno);
                goto fail;
        }
        if (content_len > 0) {
            content[content_len - 1] = 0;
        } else {
            content[0] = 0;
        }

        lseek(fd, 0, SEEK_END);

        for (int i = 0; i < cnt; ++i) {
            if (content_len == 0 || !strstr(content, libs[i])) {
                dprintf(fd, "%s\n", libs[i]);
            }
        }
fail:
        close(fd);
}

static void discover_target(int argc, char* argv[], const char* project_root, const char* feature_root) {
        if (argc <= 0) return;

        char** libs = static_cast<char**>(calloc((size_t)argc, sizeof(char*)));
        if (!libs) return;
        int pos = 0;

        if (getenv(CPP2RUST_LD_SKIP)) goto fail;

        for (int i = 1; i < argc; ++i) {
                char* static_lib = get_static_lib(argv[i], project_root);
                if (static_lib) {
                        libs[pos++] = static_lib;
                } else if (strncmp(argv[i], "-o", 2) == 0) {
                        if (argv[i][2] == 0 && i < argc - 1) {
                            libs[pos++] = get_file(argv[i + 1]);
                        } else if (argv[i][2]) {
                            libs[pos++] = get_file(&argv[i][2]);
                        }
                }
        }
        target_save(libs, pos, feature_root);
fail:
        free(libs);
}

__attribute__((constructor)) static void cpp2rust_hook(void) {
        char* project_root = 0;
        char* feature_root = 0;
        char** argv = NULL;
        int argc = -1;

        project_root = path_from(CPP2RUST_PROJECT_ROOT);
        if (!project_root) {
                return;
        }

        feature_root = path_from(CPP2RUST_FEATURE_ROOT);
        if (!feature_root) {
                goto fail;
        }

        argc = read_proc_cmdline(&argv);
        if (argc < 1) {
                DBG("failed to read /proc/self/cmdline\n");
                goto fail;
        }

        DBG("invoked as %s (argc=%d)\n", argv[0], argc);

        if (is_compiler(argv[0])) {
               discover_cppfile(argc, argv, project_root, feature_root);
        } else if (is_linker(argv[0])) {
               discover_target(argc, argv, project_root, feature_root);
        } else {
               DBG("not a compiler or linker: %s\n", argv[0]);
        }
fail:
        if (argv) free_cmdline(argv, argc > 0 ? argc : 0);
        if (project_root) free(project_root);
        if (feature_root) free(feature_root);
}
