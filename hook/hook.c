#define _GNU_SOURCE
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/file.h>
#include <sys/stat.h>
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

static int debug_enabled(void) {
  static int initialized = 0;
  static int enabled = 0;
  if (!initialized) {
    enabled = getenv(CPP2RUST_DEBUG) != NULL;
    initialized = 1;
  }
  return enabled;
}

#define DBG(...)                                                              \
  do {                                                                        \
    if (debug_enabled()) {                                                    \
      dprintf(2, "[cpp2rust-hook] " __VA_ARGS__);                             \
    }                                                                         \
  } while (0)

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

static int has_header_ext(const char *path) {
  const char *dot = strrchr(path, '.');
  if (!dot)
    return 0;
  return strcmp(dot, ".h") == 0 || strcmp(dot, ".hpp") == 0 ||
         strcmp(dot, ".hh") == 0 || strcmp(dot, ".hxx") == 0;
}

static int has_cpp_ext(const char *path) {
  const char *dot = strrchr(path, '.');
  if (!dot)
    return 0;
  return strcmp(dot, ".c") == 0 || strcmp(dot, ".cc") == 0 ||
         strcmp(dot, ".cpp") == 0 || strcmp(dot, ".cxx") == 0 ||
         strcmp(dot, ".c++") == 0 || strcmp(dot, ".C") == 0;
}

static int under_prefix(const char *path, const char *prefix) {
  size_t plen = strlen(prefix);
  if (strncmp(path, prefix, plen) != 0)
    return 0;
  return path[plen] == '\0' || path[plen] == '/';
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

static int contains_exact_line(const char *text, ssize_t n, const char *line) {
  if (!text || n <= 0 || !line)
    return 0;
  size_t line_len = strlen(line);
  const char *cur = text;
  const char *end = text + n;
  while (cur < end) {
    const char *nl = memchr(cur, '\n', (size_t)(end - cur));
    const char *line_end = nl ? nl : end;
    size_t cur_len = (size_t)(line_end - cur);
    if (cur_len == line_len && strncmp(cur, line, line_len) == 0) {
      return 1;
    }
    if (!nl)
      break;
    cur = nl + 1;
  }
  return 0;
}

static void save_captured_headers(char **headers, int count,
                                  const char *feature_root) {
  if (count <= 0)
    return;

  char list_path[MAX_PATH_LEN];
  if (snprintf(list_path, sizeof(list_path), "%s/meta/captured_headers.list",
               feature_root) >= (int)sizeof(list_path)) {
    return;
  }

  char meta_dir[MAX_PATH_LEN];
  if (snprintf(meta_dir, sizeof(meta_dir), "%s/meta", feature_root) >=
      (int)sizeof(meta_dir)) {
    return;
  }
  if (mkdir_p(meta_dir) != 0) {
    return;
  }

  int fd = open(list_path, O_CREAT | O_RDWR, 0666);
  if (fd < 0)
    return;

  if (flock(fd, LOCK_EX) != 0) {
    close(fd);
    return;
  }

  char existing[MAX_CMD_LEN];
  ssize_t n = read(fd, existing, sizeof(existing) - 1);
  if (n < 0)
    n = 0;
  existing[n] = '\0';

  lseek(fd, 0, SEEK_END);
  for (int i = 0; i < count; ++i) {
    if (!headers[i])
      continue;
    if (!contains_exact_line(existing, n, headers[i])) {
      dprintf(fd, "%s\n", headers[i]);
    }
  }

  close(fd);
}

static void discover_headers(int argc, char **argv, const char *project_root,
                             const char *feature_root) {
  char **captured = calloc((size_t)argc, sizeof(char *));
  if (!captured)
    return;

  int pos = 0;
  for (int i = 1; i < argc; ++i) {
    const char *arg = argv[i];
    if (!arg || arg[0] == '-')
      continue;
    if (!has_header_ext(arg))
      continue;
    if (access(arg, R_OK) != 0)
      continue;

    char *abs = realpath(arg, NULL);
    if (!abs)
      continue;
    if (!under_prefix(abs, project_root)) {
      free(abs);
      continue;
    }
    captured[pos++] = abs;
  }

  save_captured_headers(captured, pos, feature_root);

  for (int i = 0; i < pos; ++i)
    free(captured[i]);
  free(captured);
}

static int parse_cpp_args(int argc, char *argv[], char *extracted[],
                          char *cppfiles[], int *cpp_count) {
  int cnt = 0;
  int files = 0;
  for (int i = 1; i < argc; ++i) {
    char *arg = argv[i];
    if (!arg)
      continue;

    if (arg[0] != '-') {
      if (has_cpp_ext(arg) && access(arg, R_OK) == 0) {
        *cppfiles = realpath(arg, NULL);
        if (*cppfiles) {
          ++cppfiles;
          ++files;
        }
      }
      continue;
    }

    if (arg[1] == 'I' || arg[1] == 'D' || arg[1] == 'U') {
      extracted[cnt++] = arg;
      if (!arg[2] && i + 1 < argc) {
        extracted[cnt++] = argv[++i];
      }
    } else if (strcmp(&arg[1], "include") == 0 || strcmp(&arg[1], "isystem") == 0 ||
               strcmp(&arg[1], "iquote") == 0 || strcmp(&arg[1], "isysroot") == 0) {
      extracted[cnt++] = arg;
      if (i + 1 < argc) {
        extracted[cnt++] = argv[++i];
      }
    } else if (strncmp(&arg[1], "std=", 4) == 0 || strcmp(&arg[1], "fshort-enums") == 0) {
      extracted[cnt++] = arg;
    }
  }
  if (cpp_count) {
    *cpp_count = files;
  }
  return cnt;
}

static const char *strip_prefix(const char *path, const char *prefix) {
  size_t plen = strlen(prefix);
  if (strncmp(path, prefix, plen) != 0) {
    return NULL;
  }
  if (prefix[plen - 1] == '/') {
    return &path[plen];
  }
  if (path[plen] == '/') {
    return &path[plen + 1];
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

static void preprocess_cpp_file(const char *cc, int argc, char *argv[],
                                const char *cppfile, const char *project_root,
                                const char *feature_root) {
  const char *rel = strip_prefix(cppfile, project_root);
  if (!rel)
    return;

  char full_path[MAX_PATH_LEN];
  int full_len = snprintf(full_path, sizeof(full_path), "%s/cpp/%s2rust",
                          feature_root, rel);
  if (full_len <= 0 || full_len >= (int)sizeof(full_path))
    return;

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
  if (snprintf(opts_path, sizeof(opts_path), "%s.opts", full_path) <
      (int)sizeof(opts_path)) {
    save_options(opts_path, argc, argv);
  }

  pid_t pid = fork();
  if (pid == 0) {
    char *new_argv[argc + 8];
    int pos = 0;
    new_argv[pos++] = (char *)cc;
    new_argv[pos++] = "-E";
    new_argv[pos++] = "-C";
    new_argv[pos++] = (char *)cppfile;
    new_argv[pos++] = "-o";
    new_argv[pos++] = full_path;
    for (int i = 0; i < argc; ++i) {
      new_argv[pos++] = argv[i];
    }
    new_argv[pos++] = NULL;
    execvp(cc, new_argv);
    _exit(127);
  } else if (pid > 0) {
    int wstatus = 0;
    waitpid(pid, &wstatus, 0);
  }
}

static void discover_cpp_files(int argc, char **argv, const char *project_root,
                               const char *feature_root) {
  char **cflags = calloc((size_t)argc, sizeof(char *));
  char **cppfiles = calloc((size_t)argc, sizeof(char *));
  if (!cflags || !cppfiles)
    goto fail;

  int file_count = 0;
  int cnt = parse_cpp_args(argc, argv, cflags, cppfiles, &file_count);
  if (file_count == 0)
    goto fail;

  for (int i = 0; i < file_count; ++i) {
    preprocess_cpp_file(argv[0], cnt, cflags, cppfiles[i], project_root,
                        feature_root);
  }

fail:
  if (cppfiles) {
    for (char **cf = cppfiles; *cf; ++cf)
      free(*cf);
    free(cppfiles);
  }
  if (cflags) {
    free(cflags);
  }
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

  DBG("invoked: %s (argc=%d)\n", argv[0], argc);
  if (is_compiler(argv[0])) {
    if (!getenv(CPP2RUST_CC_SKIP)) {
      setenv(CPP2RUST_CC_SKIP, "1", 0);
      discover_headers(argc, argv, project_root, feature_root);
      discover_cpp_files(argc, argv, project_root, feature_root);
    }
  }

  free_cmdline(argv, argc);
  free(project_root);
  free(feature_root);
}
