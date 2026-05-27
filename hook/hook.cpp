#include <dlfcn.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <libgen.h>
#include <limits.h>
#include <errno.h>

extern char **environ;

static const char* CPP_COMPILERS[] = {
    "g++", "clang++", "c++",
    "g++-12", "g++-13", "g++-14",
    "clang++-16", "clang++-17", "clang++-18",
    NULL
};

static const char* CPP_EXTENSIONS[] = {
    ".cpp", ".cc", ".cxx", ".c++", ".C", ".cp",
    NULL
};

static int debug_mode = 0;

static void debug_log(const char* fmt, ...) {
    if (!debug_mode) return;
    va_list ap;
    va_start(ap, fmt);
    fprintf(stderr, "[c2rust-hook] ");
    vfprintf(stderr, fmt, ap);
    fprintf(stderr, "\n");
    va_end(ap);
}

static int is_cpp_compiler(const char* prog) {
    const char* base = strrchr(prog, '/');
    base = base ? base + 1 : prog;
    for (int i = 0; CPP_COMPILERS[i]; i++) {
        if (strcmp(base, CPP_COMPILERS[i]) == 0) return 1;
    }
    return 0;
}

static int is_cpp_file(const char* filename) {
    for (int i = 0; CPP_EXTENSIONS[i]; i++) {
        size_t flen = strlen(filename);
        size_t elen = strlen(CPP_EXTENSIONS[i]);
        if (flen >= elen && strcmp(filename + flen - elen, CPP_EXTENSIONS[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

static const char* get_feature_root(void) {
    const char* root = getenv("C2RUST_FEATURE_ROOT");
    return root ? root : ".c2rust/v5";
}

static const char* get_project_root(void) {
    const char* root = getenv("C2RUST_PROJECT_ROOT");
    return root ? root : ".";
}

static const char* get_cxx(void) {
    const char* cxx = getenv("C2RUST_CXX");
    if (cxx && access(cxx, X_OK) == 0) return cxx;
    if (access("/usr/bin/g++", X_OK) == 0) return "g++";
    if (access("/usr/bin/clang++", X_OK) == 0) return "clang++";
    return "c++";
}

static void mkdirs(const char* path) {
    char tmp[PATH_MAX];
    snprintf(tmp, sizeof(tmp), "%s", path);
    for (char* p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0755);
            *p = '/';
        }
    }
    mkdir(tmp, 0755);
}

static void build_output_path(const char* src_file, char* out_path, size_t out_size) {
    const char* feature_root = get_feature_root();
    const char* project_root = get_project_root();

    char abs_src[PATH_MAX] = {0};
    char abs_proj[PATH_MAX] = {0};

    if (!realpath(src_file, abs_src)) {
        snprintf(abs_src, sizeof(abs_src), "%s", src_file);
    }
    if (!realpath(project_root, abs_proj)) {
        snprintf(abs_proj, sizeof(abs_proj), "%s", project_root);
    }

    const char* rel = abs_src;
    size_t proj_len = strlen(abs_proj);
    if (strncmp(abs_src, abs_proj, proj_len) == 0 && abs_src[proj_len] == '/') {
        rel = abs_src + proj_len + 1;
    }

    snprintf(out_path, out_size, "%s/c/%s.c2rust", feature_root, rel);
}

static void record_target(const char* output_file) {
    if (!output_file) return;

    const char* feature_root = get_feature_root();
    char targets_path[PATH_MAX];
    snprintf(targets_path, sizeof(targets_path), "%s/targets.list", feature_root);

    mkdirs(feature_root);

    FILE* f = fopen(targets_path, "a");
    if (!f) {
        debug_log("Failed to open %s: %s", targets_path, strerror(errno));
        return;
    }

    fprintf(f, "%s\n", output_file);
    fclose(f);
}

static void run_preprocessor(char* const argv[], const char* src_file) {
    char out_path[PATH_MAX];
    build_output_path(src_file, out_path, sizeof(out_path));

    char dir_copy[PATH_MAX];
    snprintf(dir_copy, sizeof(dir_copy), "%s", out_path);
    mkdirs(dirname(dir_copy));

    int argc = 0;
    while (argv[argc]) argc++;

    char** pp_args = (char**)malloc((argc + 8) * sizeof(char*));
    if (!pp_args) return;

    int n = 0;
    pp_args[n++] = (char*)get_cxx();
    pp_args[n++] = (char*)"-E";
    pp_args[n++] = (char*)"-C";
    pp_args[n++] = (char*)"-P";

    int skip_next = 0;
    for (int i = 1; argv[i]; i++) {
        if (skip_next) {
            skip_next = 0;
            continue;
        }
        if (strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "-MF") == 0 || strcmp(argv[i], "-MT") == 0 || strcmp(argv[i], "-MQ") == 0) {
            skip_next = 1;
            continue;
        }
        if (strcmp(argv[i], "-c") == 0 || strcmp(argv[i], "-MD") == 0 || strcmp(argv[i], "-MMD") == 0 || strcmp(argv[i], "-MP") == 0) {
            continue;
        }
        if (strcmp(argv[i], src_file) == 0) {
            continue;
        }
        pp_args[n++] = argv[i];
    }

    pp_args[n++] = (char*)src_file;
    pp_args[n++] = (char*)"-o";
    pp_args[n++] = out_path;
    pp_args[n] = NULL;

    debug_log("Preprocessing %s -> %s", src_file, out_path);

    pid_t pid = fork();
    if (pid == 0) {
        setenv("C2RUST_HOOK_BYPASS", "1", 1);
        execvp(pp_args[0], pp_args);
        perror("c2rust-hook: execvp preprocessor failed");
        _exit(1);
    }

    if (pid > 0) {
        int status = 0;
        waitpid(pid, &status, 0);
        if (WIFEXITED(status) && WEXITSTATUS(status) != 0) {
            debug_log("Preprocessor failed for %s (exit %d)", src_file, WEXITSTATUS(status));
        }
    }

    free(pp_args);
}

typedef int (*execve_fn)(const char*, char* const[], char* const[]);
typedef int (*execvp_fn)(const char*, char* const[]);

static int should_intercept(const char* pathname, char* const argv[]) {
    if (getenv("C2RUST_HOOK_BYPASS") != NULL) return 0;
    if (!pathname || !argv) return 0;
    return is_cpp_compiler(pathname);
}

static void intercept_compiler_call(const char* pathname, char* const argv[]) {
    if (!should_intercept(pathname, argv)) return;

    debug_mode = (getenv("C2RUST_DEBUG") != NULL);

    const char* output = NULL;
    for (int i = 1; argv[i]; i++) {
        if (strcmp(argv[i], "-o") == 0 && argv[i + 1]) {
            output = argv[i + 1];
            i++;
            continue;
        }
        if (is_cpp_file(argv[i])) {
            run_preprocessor(argv, argv[i]);
        }
    }

    if (output) {
        record_target(output);
    }
}

int execve(const char* pathname, char* const argv[], char* const envp[]) {
    static execve_fn real_execve = NULL;
    if (!real_execve) {
        real_execve = (execve_fn)dlsym(RTLD_NEXT, "execve");
    }

    intercept_compiler_call(pathname, argv);
    return real_execve(pathname, argv, envp);
}

int execvp(const char* file, char* const argv[]) {
    static execvp_fn real_execvp = NULL;
    if (!real_execvp) {
        real_execvp = (execvp_fn)dlsym(RTLD_NEXT, "execvp");
    }

    intercept_compiler_call(file, argv);
    return real_execvp(file, argv);
}
