# hicc-usages 甄别与对照

本文件记录对参考实现 [`references/hicc-usages`](https://github.com/LuuuXXX/hicc-usages)
（子模块）的分析、采纳、甄别修正与有意分歧，作为「先分析甄别再优化到当前项目」这一
要求的可复核记录。

> **一句话定位**：`hicc-usages` 是「48 个 C++ 特性 × hicc Safe FFI 映射」的**人工参考样板**
> （`examples/{NNN}/` 内含 C++ 项目 + 过滤后 AST + 手写 hicc crate）；本仓 `cpp2rust-demo`
> 是把同一映射**自动化**的工具（`init` + `merge` 由 libclang AST 直接生成等价 hicc crate）。
> 两者示例编号一一对应（`001_hello_world` … `048_summary`），但 hicc-usages 的产物是手写，
> 本仓 `examples/*/rust_hicc/` 须与**工具实际输出**逐段一致（L1 黄金）。

---

## 1. 已采纳（直接借鉴到本仓）

| 主题 | hicc-usages 来源 | 本仓落地 |
|------|------------------|----------|
| **去 shim 直出形态** | 各示例 `rust_hicc/src/lib.rs` 用 `#[cpp(class="ns::T")]` + `make_unique` 工厂，无 `extern "C"` | `src/extractor/hicc_direct.rs` 自动判定 idiomatic 命名空间类并直出；`examples/006–048` 全部采用 |
| **命名空间自由函数 `import_lib!`** | `001–005` 以 `namespace <feat>_ns` 内自由函数 + `#[cpp(func="ns::fn()")]` 绑定 | `examples/001–005` 去 `extern "C"`，与之对齐 |
| **行为级冒烟样板** | `rust_hicc/tests/smoke.rs` 手写「构造→调用→`assert_eq!`」 | `src/generator/smoke_test_gen.rs` 自动生成同形态断言（标量 setter/getter 双值往返） |
| **AST 可追溯工具** | `tools/dump_ast.sh` + `tools/filter_ast.py` | 复制为 `scripts/dump_ast.sh` + `scripts/filter_ast.py`，配 `make dump-ast` |

---

## 2. 甄别修正（hicc-usages 不完全正确，本仓已纠正）

> 这些是「hicc-usages 不一定完全正确」的具体印证：直接照搬会编译失败或运行期崩溃，
> 本仓在自动化时做了纠正，并固化为黄金基线与记忆。

1. **派生类不得重复声明继承的引用返回方法（否则运行期 SIGSEGV）**
   - hicc-usages `013_inheritance_single`：`Dog` 的 `import_class!` 块**声明了**继承自
     `Animal` 的 `const std::string& name() const`。
   - 实测：派生类块内声明继承的引用返回方法会因错误的 `this` 偏移在运行期 SIGSEGV。
   - 本仓修正：继承的访问器**只在基类块声明**，派生类块只绑自身方法
     （见 `examples/013_inheritance_single/rust_hicc/src/lib.rs` 的 `Dog` 块不含 `name()`）。

2. **`make_unique` 工厂的指针/标量实参须追加 `&&` 形成转发引用**
   - 漏掉指针（如 `const char*`）的 `&&` 会致 hicc `cpp!` 展开期 `no matching function` 报错。
   - 本仓修正：`hicc_direct::make_unique_arg_type` 对「不含 `&` 的标量/指针」一律追加 `&&`，
     仅左值/右值引用原样（见 `examples/026_template_specialization`）。

3. **方法签名中对已导出命名空间类的裸引用须补全命名空间限定**
   - 如 `UniqueVector &` 须限定为 `class_move_ns::UniqueVector &`，否则 hicc 按全局作用域
     解析类型名编译失败。
   - 本仓修正：`hicc_direct::qualify_class_types`（见 `examples/009_class_move` 的 `move_from`）。

4. **友元 / 命名空间作用域自由函数会触发 extern-C 误判而禁用直出**
   - 命名空间作用域自由函数会被 collector 标记 `is_extern_c`，触发 `has_extern_c_bridge`
     从而退回旧 shim 路径。
   - 本仓修正：把这类函数（如友元 `operator<<` / `audit_total`）改为**类体内 inline 定义**，
     使其不被收集，从而保留直出（见 `examples/020_friend_function`）。

---

## 3. 有意分歧（本仓特有，不照搬 hicc-usages）

- **测试体系**：本仓有 L1–L6 + L_smoke 七层自动化测试与多平台 CI 矩阵；hicc-usages 为人工样板，
  无等价 harness。本仓 `examples/*/rust_hicc/` 必须与**工具输出**逐段一致（L1 黄金），
  因此不能直接采用 hicc-usages 的手写措辞 / 排版。
- **命名空间类命名兼容**：`ClassInfo.name` 对命名空间类仍保留扁平化前缀
  （如 `class_basic_ns_Counter`）以兼容旧 extern-C 路径的 `used_classes` 匹配，hicc 直出另用
  `simple_name` + `namespace` 还原 `ns::T`；hicc-usages 无此双轨约束。
- **vendored 决策**：真实库 E2E 依赖一律子模块化，唯 `rapidjson-refactoring` 保留 vendored
  （理由见 `references/README.md`）；hicc-usages 仅含手写示例，不涉及真实库 E2E。

---

## 4. 对应关系速查

| 维度 | hicc-usages | cpp2rust-demo（本仓） |
|------|-------------|----------------------|
| 产物来源 | 人工从 AST 手写 hicc crate | `init`+`merge` 工具自动生成 |
| 示例编号 | `001`–`048` | `001`–`048`（一一对应） |
| AST | `examples/*/ast/user-ast.json`（提交入库） | `make dump-ast` 临时转储（`.gitignore` 忽略） |
| 冒烟测试 | 手写 `tests/smoke.rs` | 工具生成 + 各示例手写补强 |
| 验证 | 无自动化 harness | L1–L6 + L_smoke + 多平台 CI |

> 子模块初始化：`git submodule update --init references/hicc-usages`。
