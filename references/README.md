# references/ — 参考与 E2E 依赖

本目录收纳「参考实现」与「真实项目 E2E 依赖」。除 `rapidjson-refactoring` 外，其余均为 Git 子模块
（见仓库根 `.gitmodules`），首次使用前需初始化：

```bash
git submodule update --init references/<name>
# 或一次性初始化 E2E 所需子模块：make submodules
```

## 子模块清单

| 路径 | 用途 |
|------|------|
| `hicc` | hicc 运行时/宏库（绑定生成目标） |
| `hicc-usages` | hicc 用法范例（去 shim 直出形态、行为级冒烟样板、`tools/` AST 辅助脚本来源） |
| `c2rust-demo` | C → Rust 迁移参考 |
| `tinyxml2` / `pugixml` / `nlohmann-json` / `fmtlib` / `sqlite` | 真实库 E2E 依赖（各自有 `tests/<lib>_e2e_test.rs` 与 `.github/workflows/e2e-<lib>.yml`） |

## `rapidjson-refactoring` 为何保留 vendored（不子模块化）

`rapidjson-refactoring/` **不是** rapidjson 上游的干净镜像，而是一套围绕 rapidjson 的
**重构工作区**，包含工具链产出与本仓特有的派生物：

- `rapidjson_legacy/`（被 E2E 取作 C++ 源根 + include）、`rapidjson_sys/`（含 `shim/`）、
  `rapidjson-rs/`、`baseline/`、`inventory/`、`reports/`、`docs/`、独立 `Cargo.toml`/`Cargo.lock`。

`tests/rapidjson_e2e_test.rs`、`tests/multi_feature_e2e_test.rs`、`tests/l5_nm_symbol_tests.rs`
与 `.github/workflows/e2e-rapidjson.yml` 均按上述固定相对路径取数。由于它没有对应的独立上游仓
可指向（内容是本项目特有的重构资产，而非可外链的纯第三方库），子模块化会割裂这套工作区并
破坏 E2E 取数路径，**收益为负**。因此明确决策：**`rapidjson-refactoring` 保持 vendored**，
其余真实库依赖一律子模块化。

## 本地 AST 可追溯工具

`scripts/dump_ast.sh` + `scripts/filter_ast.py`（源自 `hicc-usages/tools/`）可对某示例转储
精简 AST，用于人工核对工具抽取的 IR：

```bash
make dump-ast DIR=examples/006_class_basic/cpp
# 产物写入 examples/006_class_basic/ast/（已被 .gitignore 忽略，绝不入库）
```
