#define _GNU_SOURCE
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/file.h>
#include <unistd.h>

#define MAX_CMD_LEN 16384
#define MAX_PATH_LEN 8192

static const char *CPP2RUST_PROJECT_ROOT = "CPP2RUST_PROJECT_ROOT";
static const char *CPP2RUST_FEATURE_ROOT = "CPP2RUST_FEATURE_ROOT";
static const char *CPP2RUST_CC = "CPP2RUST_CC";
static const char *CPP2RUST_CC_SKIP = "CPP2RUST_CC_SKIP";
static const char *CPP2RUST_DEBUG = "CPP2RUST_DEBUG";

static const char *cc_names[] = {"gcc", "g++", "clang", "clang++", "cc", "c++"};

#define DBG(...)                                                              \
  do {                                                                        \
    if (getenv(CPP2RUST_DEBUG)) {                                             \
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

static int under_prefix(const char *path, const char *prefix) {
  size_t plen = strlen(prefix);
  if (strncmp(path, prefix, plen) != 0)
    return 0;
  return path[plen] == '\0' || path[plen] == '/';
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

  char mkdir_cmd[MAX_PATH_LEN + 32];
  if (snprintf(mkdir_cmd, sizeof(mkdir_cmd), "mkdir -p \"%s/meta\"",
               feature_root) >= (int)sizeof(mkdir_cmd)) {
    return;
  }
  system(mkdir_cmd);

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
    if (strstr(existing, headers[i]) == NULL) {
      dprintf(fd, "%s\n", headers[i]);
    }
  }

  close(fd);
}

static void discover_headers(int argc, char **argv, const char *project_root,
                             const char *feature_root) {
  if (getenv(CPP2RUST_CC_SKIP))
    return;
  setenv(CPP2RUST_CC_SKIP, "1", 0);

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
    discover_headers(argc, argv, project_root, feature_root);
  }

  free_cmdline(argv, argc);
  free(project_root);
  free(feature_root);
}
