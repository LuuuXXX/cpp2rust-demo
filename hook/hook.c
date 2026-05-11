#define _GNU_SOURCE
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define MAX_CMD_LEN 16384
#define MAX_PATH_LEN 8192

static const char *CPP2RUST_PROJECT_ROOT = "CPP2RUST_PROJECT_ROOT";
static const char *CPP2RUST_FEATURE_ROOT = "CPP2RUST_FEATURE_ROOT";
static const char *CPP2RUST_CC = "CPP2RUST_CC";
static const char *CPP2RUST_CC_SKIP = "CPP2RUST_CC_SKIP";
static const char *CPP2RUST_DEBUG = "CPP2RUST_DEBUG";

static const char *cc_names[] = {"gcc", "g++", "clang", "clang++", "cc", "c++"};

#define DBG(...)                                                               \
  do {                                                                         \
    if (debug_enabled()) {                                                     \
      dprintf(2, "[cpp2rust-hook] " __VA_ARGS__);                              \
    }                                                                          \
  } while (0)

static int debug_enabled(void) {
  static int initialized = 0;
  static int enabled = 0;
  if (!initialized) {
    enabled = getenv(CPP2RUST_DEBUG) != NULL;
    initialized = 1;
  }
  return enabled;
}

static const char *path_basename(const char *path) {
  if (!path)
    return path;
  const char *slash = strrchr(path, '/');
  return slash ? slash + 1 : path;
}

static int is_name_match(const char *name, const char *pattern) {
  size_t plen = strlen(pattern);
  if (strncmp(name, pattern, plen) != 0)
    return 0;
  if (name[plen] == '\0')
    return 1;
  if (name[plen] == '-' && isdigit((unsigned char)name[plen + 1]))
    return 1;
  return 0;
}

static int is_compiler(const char *name) {
  const char *base = path_basename(name);
  if (!base)
    return 0;

  const char *cc = getenv(CPP2RUST_CC);
  if (cc) {
    return strcmp(path_basename(cc), base) == 0;
  }

  for (size_t i = 0; i < sizeof(cc_names) / sizeof(cc_names[0]); ++i) {
    if (is_name_match(base, cc_names[i])) {
      return 1;
    }
  }
  return 0;
}

static char *path_from(const char *env_name) {
  const char *val = getenv(env_name);
  if (!val)
    return NULL;
  return realpath(val, NULL);
}

static int read_proc_cmdline(char ***out_argv) {
  int fd = open("/proc/self/cmdline", O_RDONLY);
  if (fd < 0)
    return -1;

  char buf[MAX_CMD_LEN];
  ssize_t n = read(fd, buf, sizeof(buf) - 1);
  close(fd);
  if (n <= 0)
    return -1;
  buf[n] = '\0';

  int argc = 0;
  for (ssize_t i = 0; i < n; ++i) {
    if (buf[i] == '\0')
      argc++;
  }
  if (argc <= 0)
    return -1;

  char **argv = malloc((size_t)(argc + 1) * sizeof(char *));
  if (!argv)
    return -1;

  int idx = 0;
  char *p = buf;
  char *end = buf + n;
  while (idx < argc && p < end) {
    argv[idx] = strdup(p);
    if (!argv[idx]) {
      for (int j = 0; j < idx; ++j)
        free(argv[j]);
      free(argv);
      return -1;
    }
    idx++;
    p += strlen(p) + 1;
  }
  argv[idx] = NULL;
  *out_argv = argv;
  return argc;
}

static void free_cmdline(char **argv, int argc) {
  if (!argv)
    return;
  for (int i = 0; i < argc; ++i)
    free(argv[i]);
  free(argv);
}

static int under_prefix(const char *path, const char *prefix) {
  size_t plen = strlen(prefix);
  if (strncmp(path, prefix, plen) != 0)
    return 0;
  return path[plen] == '\0' || path[plen] == '/';
}

static int is_cpp_file(const char *path) {
  const char *dot = strrchr(path, '.');
  if (!dot)
    return 0;
  return strcmp(dot, ".cc") == 0 || strcmp(dot, ".cpp") == 0 ||
         strcmp(dot, ".cxx") == 0 || strcmp(dot, ".c++") == 0 ||
         strcmp(dot, ".C") == 0 || strcmp(dot, ".h") == 0 ||
         strcmp(dot, ".hpp") == 0 || strcmp(dot, ".hh") == 0 ||
         strcmp(dot, ".hxx") == 0;
}

static int mkdir_p(const char *path) {
  char tmp[MAX_PATH_LEN];
  if (snprintf(tmp, sizeof(tmp), "%s", path) >= (int)sizeof(tmp)) {
    return -1;
  }
  size_t len = strlen(tmp);
  if (len == 0)
    return -1;
  if (tmp[len - 1] == '/') {
    tmp[len - 1] = '\0';
  }

  for (char *p = tmp + 1; *p; ++p) {
    if (*p == '/') {
      *p = '\0';
      if (mkdir(tmp, 0777) != 0 && errno != EEXIST) {
        return -1;
      }
      *p = '/';
    }
  }
  if (mkdir(tmp, 0777) != 0 && errno != EEXIST) {
    return -1;
  }
  return 0;
}

static const char *strip_prefix(const char *path, const char *prefix) {
  size_t prefix_len = strlen(prefix);
  if (strncmp(path, prefix, prefix_len) != 0) {
    return NULL;
  }
  if (prefix[prefix_len - 1] == '/') {
    return &path[prefix_len];
  }
  if (path[prefix_len] == '/') {
    return &path[prefix_len + 1];
  }
  if (path[prefix_len] == '\0') {
    return &path[prefix_len];
  }
  return NULL;
}

static void save_options(const char *path, int argc, char *argv[]) {
  int fd = open(path, O_CREAT | O_TRUNC | O_WRONLY, 0644);
  if (fd < 0)
    return;
  for (int i = 0; i < argc; ++i) {
    dprintf(fd, "\"%s\" ", argv[i]);
  }
  close(fd);
}

static int parse_args(int argc, char *argv[], char *extracted[], char *files[]) {
  int cnt = 0;
  int fpos = 0;
  for (int i = 1; i < argc; ++i) {
    char *arg = argv[i];
    if (!arg)
      continue;

    if (arg[0] != '-') {
      if (is_cpp_file(arg) && access(arg, R_OK) == 0) {
        files[fpos++] = realpath(arg, NULL);
      }
      continue;
    }

    if (arg[1] == 'I' || arg[1] == 'D' || arg[1] == 'U') {
      extracted[cnt++] = arg;
      if (!arg[2]) {
        ++i;
        if (i < argc) {
          extracted[cnt++] = argv[i];
        }
      }
    } else if (strcmp(&arg[1], "include") == 0 || strcmp(&arg[1], "isystem") == 0 ||
               strcmp(&arg[1], "iquote") == 0) {
      extracted[cnt++] = arg;
      ++i;
      if (i < argc) {
        extracted[cnt++] = argv[i];
      }
    } else if (strncmp(&arg[1], "std=", 4) == 0 || strncmp(&arg[1], "stdlib=", 7) == 0 ||
               strcmp(arg, "-fshort-enums") == 0) {
      extracted[cnt++] = arg;
    }
  }
  return cnt;
}

static void append_cpp2rust_suffix(const char *rel_path, char *out, size_t out_size) {
  if (snprintf(out, out_size, "%s", rel_path) >= (int)out_size) {
    out[0] = '\0';
    return;
  }

  if (strlen(out) + strlen(".cpp2rust") + 1 > out_size) {
    out[0] = '\0';
    return;
  }
  strcat(out, ".cpp2rust");
}

static void preprocess_file(const char *cc, int argc, char *argv[], const char *file,
                            const char *project_root, const char *feature_root) {
  const char *rel_path = strip_prefix(file, project_root);
  if (!rel_path)
    return;

  char rel_cpp2rust[MAX_PATH_LEN];
  append_cpp2rust_suffix(rel_path, rel_cpp2rust, sizeof(rel_cpp2rust));
  if (!rel_cpp2rust[0])
    return;

  char full_path[MAX_PATH_LEN];
  if (snprintf(full_path, sizeof(full_path), "%s/cpp/%s", feature_root, rel_cpp2rust) >=
      (int)sizeof(full_path)) {
    return;
  }

  char *filename = strrchr(full_path, '/');
  if (!filename)
    return;

  *filename = '\0';
  if (mkdir_p(full_path) != 0) {
    *filename = '/';
    return;
  }
  *filename = '/';

  char opts_path[MAX_PATH_LEN];
  if (snprintf(opts_path, sizeof(opts_path), "%s.opts", full_path) < (int)sizeof(opts_path)) {
    save_options(opts_path, argc, argv);
  }

  DBG("preprocessing %s -> %s\n", file, full_path);

  pid_t pid = fork();
  if (pid == 0) {
    const char *new_argv[argc + 10];
    int pos = 0;
    new_argv[pos++] = cc;
    new_argv[pos++] = "-E";
    new_argv[pos++] = "-C";
    new_argv[pos++] = file;
    new_argv[pos++] = "-o";
    new_argv[pos++] = full_path;
    new_argv[pos++] = "-P";
    for (int i = 0; i < argc; ++i) {
      new_argv[pos++] = argv[i];
    }
    new_argv[pos++] = NULL;
    execvp(cc, (char *const *)new_argv);
    _exit(127);
  } else if (pid > 0) {
    int status = 0;
    waitpid(pid, &status, 0);
  }
}

static void discover_files(int argc, char *argv[], const char *project_root,
                           const char *feature_root) {
  if (getenv(CPP2RUST_CC_SKIP))
    return;

  char **flags = calloc((size_t)argc, sizeof(char *));
  char **files = calloc((size_t)argc, sizeof(char *));
  if (!flags || !files)
    goto fail;

  int cnt = parse_args(argc, argv, flags, files);
  if (!files[0])
    goto fail;

  setenv(CPP2RUST_CC_SKIP, "1", 0);

  for (int i = 0; i < argc; ++i) {
    if (!files[i])
      break;
    if (!under_prefix(files[i], project_root))
      continue;
    preprocess_file(argv[0], cnt, flags, files[i], project_root, feature_root);
  }

fail:
  if (files) {
    for (char **f = files; *f; ++f) {
      free(*f);
    }
    free(files);
  }
  if (flags)
    free(flags);
}

__attribute__((constructor)) static void cpp2rust_hook(void) {
  char *project_root = path_from(CPP2RUST_PROJECT_ROOT);
  if (!project_root)
    return;

  char *feature_root = path_from(CPP2RUST_FEATURE_ROOT);
  if (!feature_root) {
    free(project_root);
    return;
  }

  char **argv = NULL;
  int argc = read_proc_cmdline(&argv);
  if (argc < 1) {
    free(project_root);
    free(feature_root);
    return;
  }

  DBG("invoked as %s (argc=%d)\n", argv[0], argc);
  if (is_compiler(argv[0])) {
    discover_files(argc, argv, project_root, feature_root);
  }

  free_cmdline(argv, argc);
  free(project_root);
  free(feature_root);
}
