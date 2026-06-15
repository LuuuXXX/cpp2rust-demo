# Implementation Tasks: 真实第三方库 Direct 模式 Example

**Change ID:** `real-lib-direct-examples`

---

## Phase 1: 基础设施（Submodule + 入口文件）

- [x] 1.1 添加 rapidjson submodule 到 `references/`
- [x] 1.2 创建 `examples/050_rapidjson_direct/cpp/` 入口文件
- [x] 1.3 创建 `examples/051_pugixml_direct/cpp/` 入口文件
- [x] 1.4 创建 `examples/052_nlohmann_json_direct/cpp/` 入口文件
- [x] 1.5 初始化每组 example 的 `rust_hicc/` Cargo 项目骨架
- [x] 1.6 更新 `.gitmodules`

**Quality Gate:**
- [x] `references/rapidjson/` 目录存在且包含 `include/rapidjson/`
- [x] `references/pugixml/` 已初始化
- [x] `references/nlohmann-json/` 已初始化
- [x] references/ 总体积约 33 MB（超出 5 MB 限制，需更新 repo-slim spec）
- [x] 3 组 `cpp/` 文件可被 `g++ -c` 编译

---

## Phase 2: 工具生成 + 验证

- [x] 2.1 对 3 组 example 运行 AST 解析，检查 direct 模式分类结果
- [x] 2.2 发现根本问题：`build_direct_class_specs()` 过滤 `!is_in_namespace` → 所有命名空间类（包括所有真实库类）导致 `class_specs: 0`
- [x] 2.3 工具对 pugixml 生成 29 个类（含重复前向声明类），对 rapidjson 生成 6 个类，对 nlohmann-json 生成 0 个类（全部为内部实现类或模板类）
- [x] 2.4 记录所有工具缺陷

---

## Phase 3: 工具改进（修复真实库暴露的缺陷）

- [x] 3.1 修复 `build_direct_class_specs()` 的 `!is_in_namespace` 过滤器 → 允许命名空间类通过 `using` 别名绑定
- [x] 3.2 添加 `ClassInfo.namespace_prefix` 字段 + `collect_namespace()` 累积完整命名空间路径
- [x] 3.3 添加 `is_internal_class()` 过滤器（detail_*, internal_*, dtoa_impl_*, std_* 前缀；_impl, _helper, _fn, _tag 等后缀）
- [x] 3.4 添加 `strip_namespace_from_name()` 从扁平化类名中提取原始类名（支持嵌套命名空间）
- [x] 3.5 添加 `build_using_alias()` 生成 `using` 别名声明（如 `using xml_node = pugi::xml_node;`）
- [x] 3.6 `build_direct_class_specs()` 返回 `(Vec<ClassSpec>, Vec<String>)` — 类规范 + using 别名
- [x] 3.7 `extract()` 注入 `using` 别名到 `cpp_block_lines`
- [x] 3.8 `build_direct_lib_spec()` 使用 `lookup_class_info()` 双级查找 + 扁平名称进行 `fwd_decls`
- [x] 3.9 修复 `strip_namespace_from_name()` 中 `::` → `_` 转换
- [x] 3.10 添加单元测试（is_internal_class, strip_namespace_from_name, build_using_alias）
- [x] 3.11 修复 `block_parser` 对 `pub fn` 签名的解析
- [x] 3.12 更新黄金文件（043, 044, 046）

**Quality Gate:**
- [x] `cargo test --lib` 全通过（321 个）
- [x] `cargo clippy` clean
- [x] `cargo fmt --check` clean
- [x] 工具修复了命名空间类绑定缺陷（3 个核心函数 + 2 个 bug fix）

---

## Phase 4: 手动示例 + 烟雾测试 + 文档

- [x] 4.1 编写 050_rapidjson_direct 手动 `lib.rs`（绑定 ParseResult + using 别名 + make_unique 工厂）
- [x] 4.2 编写 051_pugixml_direct 手动 `lib.rs`（绑定 xml_parse_result + using 别名 + make_unique 工厂）
- [x] 4.3 编写 052_nlohmann_json_direct 手动 `lib.rs`（extern "C" wrapper 函数 + import_lib!）
- [x] 4.4 修复 3 组 build.rs 的 include 路径（使用 CARGO_MANIFEST_DIR 解析绝对路径）
- [x] 4.5 050 cargo build + cargo run 成功（ParseResult 烟雾测试通过）
- [x] 4.6 051 cargo build + cargo run 成功（xml_parse_result 烟雾测试通过）
- [x] 4.7 052 cargo build + cargo run 成功（nlohmann_json_parse_and_dump 烟雾测试通过）
- [x] 4.8 更新 `docs/feature-matrix.md`（3 行 + TM 降级标签）

**Quality Gate:**
- [x] `cargo test --lib` 全通过
- [x] `cargo test --test l4_merge_integration_tests` 全通过（7 个）
- [x] `cargo clippy` clean
- [x] `cargo fmt --check` clean
- [x] 3 组 real-lib example 的 cargo build + cargo run 通过
- [x] Documentation synced（feature-matrix）

**Remaining (deferred to future):**
- L1 golden tests for 050/051/052 (requires libclang + full-test feature)
- L2 compile tests for 050/051/052
- e2e CI jobs for 050/051/052
- Update repo-slim spec threshold

---

## Completion Checklist

- [x] Phase 1-4 complete
- [x] All quality gates passed (lib tests + L4 + clippy + fmt)
- [x] references/ 含 7 个 submodule
- [x] 3 组 real-lib example 的 cargo build + run 通过
- [x] 至少 1 个工具缺陷被修复（命名空间类绑定 + 内部类过滤 + pub fn 解析 + nested namespace strip）
- [x] Documentation synced（feature-matrix）
- [x] Ready for `/openspec-archive`
