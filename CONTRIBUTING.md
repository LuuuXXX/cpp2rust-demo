# Contributing to cpp2rust-demo

感谢你对本项目的关注！以下是参与贡献的基本指南。

## 前提条件

- **Rust**：stable 工具链（`rustup install stable`）
- **libclang**：解析 C++ AST 所必需
  - Linux：`sudo apt-get install libclang-dev`
- **C++ 编译器**：`g++` 或 `clang++`（用于预处理和 L3/L4 测试）

## 克隆与构建

```bash
git clone https://github.com/LuuuXXX/cpp2rust-demo.git
cd cpp2rust-demo
cargo build
```

## 运行测试

| 目的 | 命令 |
|------|------|
| 基本单元测试（不需要 libclang） | `cargo test` |
| L1 黄金文件测试（需要 libclang） | `cargo test --test l1_golden_tests --features full-test -- --test-threads=1` |
| L2 编译测试 | `cargo test --test l2_compile_tests` |
| L3 运行测试（需要 libclang + g++） | `cargo test --test l3_run_tests --features full-test -- --test-threads=1` |
| L4 E2E 测试（需要 libclang + g++） | `cargo test --test rapidjson_e2e_test -- --test-threads=1` |
| L5 nm 符号验证测试（需要 nm） | `cargo test -- --ignored` |
| 更新 L1 黄金文件 | `cargo test --test l1_golden_tests update_all_goldens --features full-test` |

更多详情见 [DEVELOPMENT.md](DEVELOPMENT.md)。

## 代码风格

- 遵循 `rustfmt` 默认格式（`cargo fmt` 格式化后提交）
- 新增功能请附带单元测试
- 模块顶部的 `//!` 文档注释请注明所属 Phase（Phase 1～6）

## 提交 PR

1. Fork 本仓库并创建特性分支
2. 确保 `cargo build` 和 `cargo test` 均通过
3. PR 描述中说明改动目的和测试覆盖情况
4. 所有 CI 检查通过后等待 Review

## 添加新的真实项目验证脚本

如果你为新的 C++ 库添加了 E2E 测试（如 `tests/<project>_e2e_test.rs`），建议同步添加验证脚本：

1. **参考模板**：复制 `usage/verify-tinyxml2-ffi.sh` 作为起点（单文件项目最简单）
2. **调整 § 2**：修改项目源文件定位逻辑（`references/<project>` 路径）
3. **调整 § 3**：修改编译命令（根据项目构建方式：make / cmake / 单文件 g++）
4. **调整 § 4**：修改 init 构建命令（确保 LD_PRELOAD 拦截生效）
5. **测试**：本地运行 `bash usage/verify-<project>-ffi.sh` 确认通过
6. **更新文档**：在 `usage/README.md` 的"§ 三"章节表格中添加新行
7. **更新 `verify-all.sh`**：在 `PROJECTS` 数组中追加项目名

验证脚本遵循"7 段式"结构（详见任意现有脚本注释）。

