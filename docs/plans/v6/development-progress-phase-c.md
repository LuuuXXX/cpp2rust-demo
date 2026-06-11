# v6 开发进展记录 — Phase C：`@make_proxy` 代理工厂骨架

> 本文档记录 v6 方案（见 `automated-cpp2rust-ffi-v6.md`）中 **Phase C（高级映射）**
> 阶段的开发目标、详细方案、详细进展与后续计划。全程使用简体中文。
>
> 前序进展：
> - `development-progress.md`（Phase A/B：模板类 / 模板函数泛型骨架）
> - `development-progress-phase-b-plus.md`（Phase B 增强：模板实例化别名）
> - `development-progress-phase-b-plus2.md`（Phase B 增强（续）：实例化追踪扩展 + 构造工厂骨架）
> - `development-progress-phase-b-plus3.md`（Phase B 增强（再续）：显式实例化追踪）
> - `development-progress-phase-b-plus4.md`（Phase B 增强（收尾）：局部变量声明实例化追踪）
>
> 已合并的相关 PR：[#137](https://github.com/LuuuXXX/cpp2rust-demo/pull/137)、
> [#139](https://github.com/LuuuXXX/cpp2rust-demo/pull/139)、
> [#140](https://github.com/LuuuXXX/cpp2rust-demo/pull/140)、
> [#141](https://github.com/LuuuXXX/cpp2rust-demo/pull/141)、
> [#142](https://github.com/LuuuXXX/cpp2rust-demo/pull/142)、
> [#143](https://github.com/LuuuXXX/cpp2rust-demo/pull/143)。

---

## 1. 开发目标

v6 方案 §7 的 **Phase C（高级映射）** 包含三项：

1. 抽象类 → `#[interface]` Trait —— **已在 `class_spec.rs` 落地**（`is_interface` 判定，参与默认产物）；
2. 私有析构 → `destroy = "..."` 属性 —— **已在 `ClassSpec.destroy_fn` / 生成器落地**；
3. 可选 `@make_proxy` 工厂（让 Rust 侧实现 C++ 抽象类）—— **本阶段实现**；
4. RTTI 场景 → `@dynamic_cast` —— 列入后续计划。

因此本阶段聚焦 Phase C 中尚未落地的 **`@make_proxy` 代理工厂骨架**：

> 基于已映射的 `#[interface]`，为「**继承 C++ 抽象接口的具体类**」派生结合
> `#[interface(name = ...)]` 的 `@make_proxy` 工厂骨架，使 Rust 侧可通过组合模式
> 实现 C++ 抽象类（参见 `references/hicc/examples/interface`）。

具体目标：

1. **识别代理目标**：在当前编译单元中识别「非抽象、可实例化，且继承自某个纯虚接口基类」的
   具体类，并定位其**直接接口基类**作为 `#[interface(name = ...)]` 的目标。
2. **派生工厂骨架**：由该具体类的公有构造函数（排除拷贝 / 移动构造）派生 `@make_proxy`
   工厂，第一个参数固定为 Rust 实现类（`hicc::Interface<具体类>`），其后接构造函数参数。
3. **硬约束（来自 v6 方案 §1.3 / §9）**：
   - `init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构完全不变；
   - **默认产物逐字节不变** —— 本阶段能力由**新增**环境变量开关 `CPP2RUST_GEN_PROXY`
     控制，默认关闭。

---

## 2. 详细方案

### 2.1 新增开关，保证默认产物不变

`@make_proxy` 代理工厂骨架不属于模板能力，故引入**独立**的环境变量
**`CPP2RUST_GEN_PROXY`**（判定逻辑与 `CPP2RUST_GEN_TEMPLATES` 一致，抽取为
`generator::hicc_codegen::env_switch_enabled` 复用）：

- 默认关闭，仅当取值为 `1` / `true` / `yes` / `on`（忽略大小写）时启用；
- 提取器**始终**构建代理工厂规格（开销极小），便于测试与未来扩展；
- 生成器仅在 `proxy_enabled()` 为真时输出代理工厂骨架；
- 关闭时不输出任何代理相关内容，因此所有既有 L1 黄金 / L2 / L3 / L4 / L5 基线不受影响。

> 说明：`#[interface]` 本身已是默认产物的一部分（纯虚接口类的既有映射），本阶段**不改动**
> 该行为；新增的仅是「具体类 → `@make_proxy` 工厂」这一可选骨架。

### 2.2 接口判定的复用

文件：`src/extractor/class_spec.rs`

将原 `build_class_spec` 内联的 `is_interface` 判定抽取为公共辅助
`is_interface_class(ci, all_classes)`（其所有 public 非 ctor/dtor 方法、含继承均为纯虚，
且至少有一个这样的方法），供 Phase C 识别接口基类复用。`build_class_spec` 改为调用该
辅助，逻辑等价，默认产物不变。

### 2.3 代理目标识别与工厂派生

文件：`src/extractor/proxy_spec.rs`（新增）

`build_proxy_factories(ast)`：

1. 仅纳入来自当前编译单元（`is_from_current_file`）、**非抽象**（`!is_abstract`，可实例化）
   且**本身不是纯虚接口**的具体类；
2. 在其直接基类中按声明顺序查找第一个满足 `is_interface_class` 的接口基类，作为
   `#[interface(name = ...)]` 的目标；无接口基类则跳过；
3. 收集该类的**公有构造函数**，排除拷贝 / 移动构造（唯一参数为本类引用 `const Foo&` /
   `Foo&&`）；对每个剩余构造函数派生一个 `ProxyFactorySpec`：
   - **Rust 工厂名**：`new_rust_<类名 snake_case>`（多个构造函数时追加序号 `_0` / `_1`）；
   - **C++ 工厂签名**：`<类名> @make_proxy<<类名>>(<构造函数参数类型列表>)`；
   - **Rust 参数**：构造函数参数映射为 Rust 类型（生成时位于 `intf` 参数之后）。

### 2.4 IR 与生成器

- IR（`src/ffi_model.rs`）：新增 `ProxyFactorySpec`
  （`rust_name` / `concrete_class` / `interface_name` / `cpp_sig` / `params`），
  `FfiSpec` 新增 `proxy_factories` 字段。
- 生成器（`src/generator/hicc_codegen.rs`）：新增 `emit_proxy_factory`，在 `import_lib!`
  块内输出代理工厂骨架；并把代理工厂纳入「`import_lib!` 是否需要生成」的判定，避免仅有
  代理工厂时整块被跳过。所有输出由 `proxy_enabled()` 开关裁决。形如：

  ```rust
  // cpp2rust-todo[PROXY]: @make_proxy 工厂骨架 —— 使 Rust 侧可实现 C++ 接口 Bar；
  // 需确认构造函数参数类型列表与 @make_proxy 一致，Rust 实现类经 hicc::Interface<Baz> 传入。
  #[cpp(func = "Baz @make_proxy<Baz>()")]
  #[interface(name = "Bar")]
  fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;
  ```

  代理工厂需用户在 Rust 侧提供接口实现类，故每个工厂均附 `cpp2rust-todo[PROXY]` 提示
  （符合 v6 方案 §8 的降级策略）。

### 2.5 测试

- 单元测试（`src/extractor/proxy_spec.rs`）：
  - `derives_proxy_factory_for_concrete_class_with_interface_base`：继承接口的具体类
    派生默认构造的代理工厂；
  - `maps_ctor_params`：构造函数参数映射到 Rust 类型并保留在 `@make_proxy` 类型列表；
  - `skips_class_without_interface_base`：不继承接口的类不派生；
  - `skips_interface_itself`：纯虚接口类自身不派生；
  - `excludes_copy_ctor_and_indexes_multiple`：拷贝构造被排除、多构造追加序号。
- 集成测试（`tests/proxy_gen_tests.rs`）：对含接口链 `Foo`→`Bar`→具体类 `Baz` 与普通类
  `Plain` 的 C++ 源码，分别验证：
  - **默认关闭**时不输出任何 `@make_proxy` / `cpp2rust-todo[PROXY]` / `new_rust_baz`；
  - **开启开关**时输出 `#[cpp(func = "Baz @make_proxy<Baz>()")]`、
    `#[interface(name = "Bar")]`、`fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;`，
    且不为普通类 `Plain` 派生代理工厂。

---

## 3. 详细进展

| 阶段 | 状态 | 说明 |
|------|------|------|
| IR：`ProxyFactorySpec` / `FfiSpec.proxy_factories` | ✅ 已完成 | 新增代理工厂规格 IR |
| 接口判定抽取 `is_interface_class` | ✅ 已完成 | `class_spec.rs`，`build_class_spec` 复用，逻辑等价 |
| 提取器：代理目标识别 + 工厂派生 | ✅ 已完成 | `proxy_spec::build_proxy_factories`（继承接口的具体类 + 公有构造函数） |
| 生成器：代理工厂骨架输出 | ✅ 已完成 | `emit_proxy_factory`，受 `CPP2RUST_GEN_PROXY` 控制 |
| 单元测试 | ✅ 已完成 | 5 个新单测（接口基类 / 参数映射 / 排除项 / 多构造序号） |
| 集成测试 | ✅ 已完成 | `proxy_gen_tests` 验证默认关闭 / 开启两态 |
| 回归验证（lib / L1 黄金 / 模板生成） | ✅ 已完成 | 259 lib 单测 + 52 L1 黄金 + 模板 / 代理生成测试全绿，默认产物逐字节不变 |

**回归验证结论**：开关默认关闭时，L1 黄金（52）与 lib 单测（259，含本阶段新增 5 个）
全部通过，确认默认产物与改动前逐字节一致；开启 `CPP2RUST_GEN_PROXY` 后，继承接口的
具体类 `Baz` 能正确生成结合 `#[interface(name = "Bar")]` 的 `@make_proxy` 工厂骨架。

---

## 4. 后续计划

以下为 v6 方案中尚未落地的阶段，建议作为后续独立 PR 推进（与既有节奏一致：先做风险最低、
可独立验证的垂直切片）：

- **Phase C（续）**：RTTI 场景 → `@dynamic_cast` 绑定（针对具体源 / 目标类型对生成
  `#[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]` 骨架，替代 v5 的
  整数枚举绕过方案）；以及私有析构 `destroy = "..."` 在更多场景下的自动检测增强。
- **Phase E（examples 改造）**：将 024/025/026/027 + 虚函数 / STL 选定示例从「手写包装
  降级」升级为「原生 hicc 模板 / 接口映射」，并为每个改造示例补充 `tests/smoke.rs`；
  同步更新各示例 README。涉及 `main.rs` → `lib.rs` + `main.rs` 的结构调整与 L1 黄金提取
  目标变更，需分批、逐示例验证，风险较高。
- **Phase F（测试 / CI）**：L1 黄金适配模板骨架 / 别名 / 工厂 / 代理片段；新增 `smoke` job 与
  `gen-verify` 端到端 job（`init` → 生成目录 `cargo test`）。
- **Phase G（剩余文档）**：随 examples 改造同步更新各示例 README 的「模板 / 接口映射 +
  冒烟测试」说明。

**风险提示**：代理工厂当前生成的仍是「骨架」——需用户在 Rust 侧提供接口实现类，并确认
`@make_proxy` 参数类型列表与构造函数一致；含类类型参数的构造函数仍需用户结合 hicc 类型
补全。代理能力默认关闭的设计为上述后续阶段提供了安全的灰度通道：可在开关开启下先验证
生成质量，再决定是否纳入黄金基线。
