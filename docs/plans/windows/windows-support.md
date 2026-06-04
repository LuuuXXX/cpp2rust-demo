# Windows 支持设计文档

## 1. 背景与目标

cpp2rust-demo 的核心是"编译拦截"——在 C++ 构建过程中拦截编译器调用，对每个 `.cpp` 文件执行预处理，产出 `.cpp2rust` 文件供后续 AST 解析使用。

在 Linux/macOS 上，通过 `LD_PRELOAD` 将 `libhook.so` 注入编译器进程，利用 `__attribute__((constructor))` 在编译器启动时执行预处理逻辑，代码透明地传递给真实编译器，对构建系统完全无感知。

**目标**：在 Windows 上实现等价的编译拦截能力，且**用户接口完全不变**：

```
# 用户只需这样调用（Windows 与 Linux 完全相同）
cpp2rust-demo init -- <BUILD_CMD>
cpp2rust-demo merge
```

---

## 2. 方案选型

### 2.1 备选方案对比

| 方案 | 可行性 | 缺点 |
|------|--------|------|
| Windows DLL 注入（`SetWindowsHookEx`/`CreateRemoteThread`） | 理论可行 | 需要以管理员权限运行；Windows Defender 必然告警；实现复杂度极高 |
| CMake `CMAKE_CXX_COMPILER_LAUNCHER` | 仅支持 CMake 项目 | 需修改 CMakeLists.txt；对 MSBuild/Makefile/nmake 项目无效 |
| 修改 PATH 注入 wrapper exe | ✅ **无需特殊权限**，支持所有构建系统 | 需要为每个编译器名创建别名；构建系统不能用绝对路径调用编译器 |
| 替换系统 PATH 中的编译器 | 危险：全局污染 | 用户 shell 的 PATH 被永久修改，不可接受 |

### 2.2 最终选择：PATH 首位注入（进程局部）

**核心原则**：所有 PATH 修改通过 `Command::env()` 作用于子进程，父 shell 不受影响。

```
cpp2rust-demo init -- msbuild MyProject.sln
      │
      ▼（run_with_hook_windows）
创建 tempdir（自动 drop 时清理）
  tempdir/cl.exe         → hook-wrapper.exe 的副本
  tempdir/clang-cl.exe   → hook-wrapper.exe 的副本
  tempdir/g++.exe        → hook-wrapper.exe 的副本
  tempdir/clang++.exe    → hook-wrapper.exe 的副本
      │
      ▼ PATH = tempdir; 原始 PATH
      子进程: msbuild MyProject.sln
        │
        ▼ 调用 cl.exe（从 tempdir 找到 hook-wrapper.exe）
        hook-wrapper.exe（被当作 cl.exe 调用）
          │ 提取预处理参数，调用真实 cl.exe 预处理
          │ → .cpp2rust 文件
          │ 透传给真实 cl.exe 编译
          ▼
        编译正常完成
```

---

## 3. hook-wrapper 与 hook.cpp 对应关系

| hook.cpp（Unix）| hook-wrapper/src/main.rs（Windows）|
|----------------|-----------------------------------|
| `__attribute__((constructor)) cpp2rust_hook()` | `fn main()` |
| `/proc/self/cmdline` 读取当前进程参数 | `std::env::args()` |
| `getenv("LD_PRELOAD")` 判断是否在 hook 上下文 | `argv[0]` 文件名判断被调用为哪个编译器 |
| 读取 `CPP2RUST_PROJECT_ROOT` 等环境变量 | 读取相同环境变量（语义完全一致）|
| `fork() + execvp()` 调用真实编译器 | `std::process::Command::new(real_compiler)` |
| `find_real_compiler()` 跳过 hook 自身 | `find_real_compiler()` 跳过自身目录（按 canonical path 比较）|
| `preprocess_cppfile()` 调用 `g++ -E -C` | `preprocess_file()` 根据编译器风格选择命令 |
| `parse_args()` 解析 `-I`/`-D`/`-std=` | `extract_preprocess_flags()` 支持 MSVC `/I`/`/D`/`/std:` |
| `CPP2RUST_CC_SKIP` 防递归 | 相同机制 |

---

## 4. 编译器工具链行为差异

### 4.1 MSVC 风格（cl.exe / clang-cl.exe）

- 参数以 `/` 开头（`/I<dir>`、`/D<macro>`、`/std:c++17`）
- 预处理命令：`cl.exe /P /C /Fi<output> <source>`
  - `/P`：预处理到文件
  - `/C`：保留注释（等价 `-C`）
  - `/Fi<output>`：指定输出文件名
  - **注意**：**不**使用 `/EP`（`/EP` 会去掉 `#line` 行号标记）；保留行号标记是必要的，libclang 依赖它们识别系统头文件（`is_in_system_header()`），从而过滤掉 MSVC 内置类型定义，避免污染提取结果。

### 4.2 GNU 风格（g++.exe / clang++.exe，MinGW-w64）

- 参数以 `-` 开头（与 Linux 完全相同）
- 预处理命令：`g++ -E -C -o <output> <source>`（与 Linux 完全相同）
- 主要来自 MSYS2/MinGW-w64 环境

### 4.3 clang-cl 特殊性

- clang-cl 接受 MSVC 风格参数（`/I`/`/D` 等）
- 工具将其归类为 MSVC 风格，使用 `/P /C /Fi<output>` 预处理命令（与 cl.exe 一致）
- 虽然 clang-cl 的预处理输出格式与 clang 兼容，但保留 `#line` 行号标记的需求与 cl.exe 相同，因此不使用 `/EP`

---

## 5. build.rs 集成机制

```
构建时（cargo build，仅 Windows 目标）：
  build.rs
    │ 调用 cargo build --release（在 hook-wrapper/ 目录）
    │ → hook-wrapper/target/release/hook-wrapper.exe
    │
    ▼ cargo:rustc-env=CPP2RUST_HOOK_WRAPPER_EXE=<absolute_path>

运行时（capture.rs，#[cfg(windows)]）：
  const HOOK_WRAPPER_BYTES: &[u8] = include_bytes!(env!("CPP2RUST_HOOK_WRAPPER_EXE"));
  ensure_hook_wrapper_exe() 将字节写入 %LOCALAPPDATA%\cpp2rust-demo\hook\hook-wrapper.exe
```

非 Windows 构建完全不触发 hook-wrapper 编译（`build.rs` 中 `if target_os != "windows" { return; }`）。

---

## 6. 目录链接（merger/mod.rs）

`merge` 命令在 `rust_dir/` 下需要 `src → src.2` 的目录链接（或 junction）。

| 方案 | 权限要求 | 说明 |
|------|----------|------|
| `std::os::windows::fs::symlink_dir` | 需要开发者模式（Win10 1803+）或管理员 | 首选 |
| `mklink /J`（目录 junction） | 无需特殊权限 | 回退方案，通过 `cmd /c mklink /J` 调用 |
| `copy_dir_all` | 无需权限 | 最终兜底，无法增量更新但功能正确 |

---

## 7. 已知限制

| 限制 | 说明 |
|------|------|
| 构建系统使用绝对路径调用编译器 | 如 MSBuild 硬编码 `C:\Program Files\MSVC\cl.exe`，PATH wrapper 失效；此时需用 `clang-cl` 替代 |
| Windows Defender | hook-wrapper.exe 是未签名的未知 exe，首次运行时可能触发实时防护扫描；建议将工具数据目录加入白名单 |
| ARM Windows（arm64-pc-windows-msvc） | 理论可支持（代码无 x86 假设），但未系统测试 |
| MSVC 输出编码 | cl.exe 默认 GBK/CP936 输出，错误信息可能乱码；工具仅解析文件路径，不受影响 |
| L3/L4/L5 测试 | 需要运行 C++ 编译和链接，首阶段在 Windows CI 上标注 `#[ignore]`，待环境完善后开放 |

---

## 8. 与现有代码的关系

- **`src/capture.rs`**：通过 `#[cfg(unix)]` / `#[cfg(windows)]` 分离两套实现，公共接口 `build_hook()` 和 `run_with_hook()` 签名不变，调用方 `src/main.rs` 无需修改。
- **`src/merger/mod.rs`**：新增 `#[cfg(windows)] fn make_dir_link()`，替换原有 `#[cfg(not(unix))] return Err(...)` 存根。
- **`hook-wrapper/`**：独立 crate，通过 `build.rs` 构建并嵌入主 binary，主 crate 的 `Cargo.toml` 不需要直接依赖它。
