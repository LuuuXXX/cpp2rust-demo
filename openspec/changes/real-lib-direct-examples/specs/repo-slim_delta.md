# Delta: Repo Structure

**Change ID:** `real-lib-direct-examples`
**Affects:** references/, examples/, tests/, CI, docs

---

## ADDED

### Requirement: 真实第三方库 Direct 模式 Example

新增 3 组使用真实第三方库（而非手写迷你类）的 direct 模式 example，验证工具在复杂真实 C++ API 下的表现。

#### Scenario: rapidjson direct 模式 example (050)
- GIVEN `examples/050_rapidjson_direct/cpp/` 仅含 `#include` 引入 rapidjson 头文件的入口文件
- AND rapidjson 提供 `Document`, `Value` 等含 template instantiation + internal typedef + =delete ctor 的复杂类
- WHEN `cpp2rust-demo init` 以 direct 模式处理该项目
- THEN 生成 `rust_hicc/src/lib.rs`（或手动补充的 `lib.rs`）通过 `cargo check`
- AND `tests/smoke.rs` 含 ≥ 1 个 assert 测试

#### Scenario: pugixml direct 模式 example (051)
- GIVEN `examples/051_pugixml_direct/cpp/` 仅含 `#include` 引入 pugixml 头文件的入口文件
- AND pugixml 提供 `xml_document`, `xml_node` 等含多层继承 + 拷贝语义的类
- WHEN `cpp2rust-demo init` 以 direct 模式处理该项目
- THEN 生成 `rust_hicc/src/lib.rs`（或手动补充的 `lib.rs`）通过 `cargo check`
- AND `tests/smoke.rs` 含 ≥ 1 个 assert 测试

#### Scenario: nlohmann-json direct 模式 example (052)
- GIVEN `examples/052_nlohmann_json_direct/cpp/` 仅含 `#include` 引入 nlohmann-json 单头文件的入口文件
- AND nlohmann-json 提供 `basic_json` 等含 template + 现代 C++ 特性的类
- WHEN `cpp2rust-demo init` 以 direct 模式处理该项目
- THEN 生成 `rust_hicc/src/lib.rs`（或手动补充的 `lib.rs`）通过 `cargo check`
- AND `tests/smoke.rs` 含 ≥ 1 个 assert 测试

---

### Requirement: rapidjson submodule

`references/` 新增 rapidjson submodule，作为 050 example 的 C++ 源码来源。

#### Scenario: rapidjson submodule 初始化
- THEN `references/rapidjson/` 目录存在
- AND `references/rapidjson/include/rapidjson/` 含 rapidjson 头文件
- AND `git submodule update --init references/rapidjson` 成功

---

### Requirement: 真实库 e2e CI job

新增 3 个 e2e CI job，验证 real-lib example 的 init → generate → cargo check 全流程。

#### Scenario: rapidjson e2e CI job
- THEN `.github/workflows/ci.yml` 含 `rapidjson-e2e` job
- AND job 步骤：checkout submodule → init → generate → cargo check

#### Scenario: pugixml e2e CI job
- THEN `.github/workflows/ci.yml` 含 `pugixml-e2e` job

#### Scenario: nlohmann-json e2e CI job
- THEN `.github/workflows/ci.yml` 含 `nlohmann-json-e2e` job

---

## MODIFIED

### Requirement: references/ 瘦身阈值

`references/` 总体积阈值从 ≤ 3 MB 调至 ≤ 35 MB，允许新增 rapidjson submodule 及现有 7 个 submodule（约 33 MB 总体积）。

#### Scenario: references 总体积限制
- GIVEN rapidjson submodule 约 4.4 MB + pugixml 3.3 MB + nlohmann-json 20 MB + tinyxml2 3.7 MB + 其他约 2 MB = 总约 33 MB
- THEN `references/` 总体积 ≤ 35 MB
- AND 阈值从 3 MB 放宽至 35 MB

---

## REMOVED

（无）
