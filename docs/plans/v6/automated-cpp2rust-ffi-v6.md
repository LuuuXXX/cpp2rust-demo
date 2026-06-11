# C++ 到 Rust Safe FFI 自动化工具 - 方案 v6

> 在 v5（LD_PRELOAD 编译拦截 + libclang 解析 + hicc 三段式生成）的基础上，
> **充分发挥 hicc 的原生能力**：让模板类 / 模板方法 / 接口（虚函数）等声明不再被跳过，
> 并补齐"生成即验证"的冒烟测试闭环。
>
> **硬约束**：不改变现有使用方法（`init` + `merge` 两个命令），全程简体中文。

---

## 1. 背景与问题定位

### 1.1 v5 已经完成的工作

- `hook.cpp` + `capture.rs`：LD_PRELOAD 拦截，产出 `.cpp2rust` 预处理文件。
- `ast_parser/`：libclang 解析，按 `is_in_system_header()` 过滤系统头。
- `extractor/` + `postprocessor/` + `generator/`：生成 `hicc::cpp!` / `hicc::import_class!` / `hicc::import_lib!` 三段式 Rust FFI。
- `examples/001–048`：48 个 C++ 特性示例，配套 L1（黄金）/L2（编译）/L3（运行）/L4（真实库 E2E）/L5（nm 符号）测试与 CI。

### 1.2 当前的核心缺口（本次优化要解决的）

通过通读 `references/hicc/reference.md`、`references/hicc/examples/*` 与当前源码，定位到信息被"跳过"的根因：

| 缺口 | 当前代码现状 | 后果 |
|------|------------|------|
| **模板类声明被丢弃** | `src/ast_parser/mod.rs:236` 的 `EntityKind::ClassTemplate` 分支只把模板源码文本记进 `template_class_ranges`，原样塞进 `hicc::cpp!`，**不生成 `import_class!` 泛型绑定** | 用户在 Rust 侧无法以 `Stack<T>` 形式调用，只能依赖手写的具体包装类（如 025 的 `IntStack`/`DoubleStack`） |
| **模板函数被完全忽略** | `src/ast_parser/mod.rs:264` 的 `EntityKind::FunctionTemplate => {}` 是空分支 | 函数模板（如 `do_swap<T>`）完全不进入 FFI，只能靠手写 `extern "C"` 包装（如 024 的 `swap_int`） |
| **接口/虚函数未走 hicc `#[interface]`** | 抽象类只生成前向声明，不映射为 hicc 的 `#[interface]` Trait | 用户无法在 Rust 侧实现 C++ 抽象类（`@make_proxy`），也读不懂虚函数的对标关系 |
| **缺少"生成即验证"** | `init` 不再生成冒烟测试（见 commit 59851fab 移除 `smoke_test_gen`）；只能靠仓库内手写的 `rust_hicc` 验证 | 工具实际生成的脚手架是否真能编译/运行，缺乏自动验证手段 |

> **结论**：hicc 本身**完全支持**模板类、模板函数、接口（见 §2 能力对照），v5 出于"只关注实例化结果"的早期假设把这些声明跳过了。
> v6 的主线就是把这些声明利用起来，既提升 FFI 覆盖度，也让生成代码与原 C++ 一一对标，便于用户理解。

### 1.3 v6 的定位

- **完全向后兼容**：`init` / `merge` 命令、参数、`.cpp2rust/<feature>/` 目录结构均不变。
- **增量增强**：新增模板类/模板函数/接口的映射策略，新增冒烟测试生成与验证，不删除既有降级策略（降级仍作为兜底）。

---

## 2. hicc 能力对照（充分发挥 hicc 的依据）

下表把 hicc `reference.md` 与 `examples/` 中验证过的能力，对应到 v6 要新增/增强的生成策略。

| hicc 能力 | 参考出处 | v6 生成策略 |
|----------|---------|------------|
| **模板类绑定** `#[cpp(class = "template<class T, class Allocator> std::vector<T, Allocator>")] pub class vector<T>` | `reference.md` §"Rust中定义C++模板类"；`hicc-std` 全量实现 | 模板类 → `import_class!` 泛型 class，成员方法用 `T` / `T::OutputRef<'_>` 映射；构造函数走 `import_lib!` |
| **POD 实例化** `vector<hicc::Pod<i32>>`；**类实例化** `vector<string>` | `reference.md` §模板类；`hicc-std/src/std_test/*` | 为每个被实例化的具体类型生成 `pub class VecInt = vector<hicc::Pod<i32>>;` 类型别名 + `#[member]` 工厂函数 |
| **模板函数** `ret func<T,...>(arg,...)` | `reference.md` §"Rust中调用C++模板函数" | 模板函数 → `import_lib!` 中按实例化类型声明 `#[cpp(func = "void do_swap<int>(int*, int*)")]` |
| **接口/虚函数继承** `#[interface]` + `@make_proxy` + `hicc::Interface<T>` | `reference.md` §"继承C++抽象类"；`examples/interface` | 抽象类/纯虚类 → `#[interface]` Trait；可选生成 `@make_proxy` 工厂，让 Rust 实现 C++ 抽象类 |
| **`dynamic_cast`** `@dynamic_cast` | `reference.md` §dynamic_cast；`examples/dynamic_cast` | RTTI 场景优先生成 `@dynamic_cast` 绑定，替代 v5 的整数枚举绕过方案 |
| **私有析构** `#[cpp(class=..., destroy=...)]` | `reference.md` §私有析构；`examples/destroy` | 检测到私有析构时生成 `destroy = "..."` 属性（025 已在用） |
| **成员/全局变量读写** `#[cpp(field=...)]` / `#[cpp(data=...)]` | `reference.md` §读写C++变量；`examples/datas` | 字段/静态成员 → `field`/`data`，返回 `&T` / `&'static T` |
| **异常捕获** `hicc::Exception<T>` | `reference.md` §捕获C++异常；`examples/class` | 可能抛异常的函数返回 `hicc::Exception<T>` |
| **缺省参数/忽略返回值** | `reference.md` §忽略缺省参数/返回值 | 缺省参数在 `func=` 中保留完整类型列表，Rust 侧省略；可忽略返回值 |
| **`hicc::cpp!` 内联适配** `SelfMethods` | `reference.md` §灵活适配 | 无法自动映射的接口，生成 `hicc::cpp!` 内联 shim 兜底（保留 v5 降级路径） |
| **`std::function` / 闭包** | `reference.md` §传递std::function；`examples/functional` | 有状态可调用对象优先尝试 hicc 的 `std::function` ↔ Rust 闭包通道，降级仍走 class wrapper |
| **冒烟测试模式** 绑定与 `#[test]` 同 crate，`cargo test` 验证 | `hicc-std/src/std_test/*` | 见 §4 冒烟测试生成 |

---

## 3. examples 优化

### 3.1 目标

1. 把当前"用手写 `extern "C"` 包装类绕过模板"的示例（024/025/026/027/028 等）升级为**直接对标 hicc 原生模板能力**的写法，让 Rust 代码能看出与原 C++ 模板的对应关系。
2. 为每个示例补充**转换后的冒烟测试**，验证生成的 Rust FFI 真的能跑通（不仅是编译）。
3. 保留旧的"降级写法"作为对照，避免破坏既有 L1/L2/L3 基线。

### 3.2 重点改造示例清单

| 示例 | 现状 | v6 目标写法 | 对应 hicc 能力 |
|------|------|-----------|--------------|
| 024 template_function | 手写 `swap_int`/`swap_double` extern "C" | `import_lib!` 中 `#[cpp(func = "void do_swap<int>(int*, int*)")]` 直接绑定模板函数实例 | 模板函数 |
| 025 template_class | 手写 `IntStack`/`DoubleStack` 包装类 | `import_class!` 泛型 `class Stack<T>` + `StackInt = Stack<hicc::Pod<i32>>` 类型别名 | 模板类 + POD 实例化 |
| 026 template_specialization | 偏特化包装 | 偏特化路径用泛型 class + 具体实例化别名表达 | 模板类偏特化 |
| 027 template_instantiation | 显式实例化包装 | `template class Foo<int>;` → 别名 + 工厂 | 显式实例化 |
| 015–018 virtual_* | opaque 指针调用 | 抽象基类 → `#[interface]` Trait；演示 Rust 侧实现 + `@make_proxy` | 接口继承 |
| 023 typeid_rtti | 整数枚举绕过 | 优先 `@dynamic_cast` 绑定 | dynamic_cast |
| 034–038 STL 容器 | wrapper 类 | 对标 `hicc-std` 写法，用泛型 class + Pod/class 实例化别名 | 模板类（STL） |

> **改造原则**：每个被改造示例的 `rust_hicc/src/main.rs` 仍是"FFI 脚手架 + 手写 `fn main()` 演示"，
> 工具只负责生成脚手架段落（L1 黄金比对范围不变）。`fn main()` 同步更新为新写法的演示。

### 3.3 每个示例新增冒烟测试

在每个 `examples/NNN_*/rust_hicc/` 下新增 `tests/smoke.rs`（集成测试），用生成的 FFI 绑定做最小可运行断言：

```
examples/NNN_xxx/rust_hicc/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs        # 生成的 FFI 脚手架（无 fn main）——供测试 use
│   └── main.rs       # 现有演示（保留）
└── tests/
    └── smoke.rs      # 新增：use 生成的绑定，#[test] 断言行为
```

- 冒烟测试**只依赖生成的 FFI 接口**，对返回值/状态做 `assert_eq!`，对标 `hicc-std/src/std_test/*` 的风格。
- 为支持 `tests/` 引用绑定，示例从"仅 `main.rs`"调整为"`lib.rs`（绑定）+ `main.rs`（演示 `use` lib）"。这是结构微调，需同步更新 `build.rs` 的 `rust_file` 列表与 L1 黄金提取目标（见 §6 兼容性）。

---

## 4. 冒烟测试生成（验证生成的 Rust FFI）

### 4.1 设计来源

参考 hicc 自身的验证方式（`hicc-std/src/std_test/*`）：**测试与 FFI 绑定在同一 crate 内**，通过 `cargo test` 链接 C++ 静态库并真实调用，验证 ABI 与行为。

### 4.2 在 `init` 阶段生成冒烟测试

> 历史背景：v5 早期版本曾在 `src/generator/smoke_test_gen.rs` 生成 `smoke_test.rs`，后于 commit 59851fab 移除。v6 以更稳健的形式恢复该能力。

- **生成位置**：`.cpp2rust/<feature>/rust/tests/smoke.rs`（与生成的 `rust/src/` 平级，符合 Cargo 集成测试约定）。
- **生成内容**：基于 `FfiSpec` 为每个可安全自动调用的接口生成"构造 → 调用 → 基本断言"骨架：
  - 工厂函数 → 创建实例。
  - 无副作用的 const 方法 / getter → 调用并断言返回类型可用（非空、可打印）。
  - 无法自动断言行为的接口 → 生成 `// cpp2rust-todo[SMOKE]: 请补充断言` 占位，保证编译通过、提示用户补全。
- **可控开关**：通过环境变量（如 `CPP2RUST_GEN_SMOKE`）控制是否生成，**默认开启但幂等**（已存在则不覆盖用户修改）。这样不改变 `init` 命令签名（兼容硬约束）。
- **生成器落点**：新增 `src/generator/smoke_test_gen.rs`，由 `project_generator` 在写出 `rust/` 时调用（参照 `write_smoke_test` 的历史接口）。

### 4.3 验证闭环

`init` 生成后，用户（或 CI）执行：

```bash
cd .cpp2rust/<feature>/rust
cargo test        # 编译 FFI + 链接 C++ 库 + 运行冒烟断言
```

冒烟测试通过 = 生成的 Rust FFI 在 ABI 与基本行为上可用。

---

## 5. 文档优化

| 文档 | 优化内容 |
|------|---------|
| `docs/INTRODUCTION.md` | 新增"模板类 / 模板函数 / 接口"映射章节；更新数据流图说明这些声明不再被跳过；补充冒烟测试验证闭环说明 |
| `docs/references/hicc.md` | 补全 hicc 模板类、模板函数、`#[interface]`、`@dynamic_cast`、冒烟测试模式的速查，与 `reference.md` 对齐 |
| `README.md` | 在特性支持表中更新模板类/模板函数/虚函数从"⚠️ 包装降级"升级为"✅ 原生 hicc 映射"；新增"如何运行生成的冒烟测试"小节 |
| 每个改造示例的 `examples/NNN_*/README.md` | 更新"C++ 代码 / Rust FFI / 运行结果"，新增"冒烟测试"说明；保证 L3 的 README 运行结果与新 `main.rs` 一致 |
| `docs/plans/v6/`（本目录） | 本方案文档；后续可追加 `migration-from-v5.md` 说明改造 diff |

> 文档全部使用简体中文，风格与现有 `INTRODUCTION.md` 一致。

---

## 6. 测试与 CI 优化

### 6.1 测试层次扩展

在现有 L1–L5 基础上新增/增强：

| 层次 | 现状 | v6 调整 |
|------|------|--------|
| L1 黄金 | 比对 `main.rs` 中 hicc 三段 | 适配 `lib.rs`（绑定）为黄金来源；新增模板类/模板函数/接口示例的黄金片段 |
| L2 编译 | `cargo build` 各 `rust_hicc` | 覆盖改造后示例（含 `lib.rs`/`tests/`）能编译 |
| L3 运行 | `cargo run` 比对 README | 改造示例的 README 运行结果同步更新 |
| **L_smoke（新增）** | 无 | 对改造示例运行 `cargo test`（`tests/smoke.rs`），验证生成式绑定行为 |
| L6（新增，生成验证） | 无 | 端到端：对选定示例跑 `init` → 对生成的 `.cpp2rust/<feature>/rust` 跑 `cargo test`，验证**工具实际输出**（而非手写黄金）可运行 |
| L4/L5 真实库/符号 | rapidjson 等 | 不变，仅确认模板增强未回归 |

### 6.2 CI（`.github/workflows/ci.yml`）调整

- 新增 `smoke` job：`cargo test --features full-test`（或专用 feature）运行示例冒烟测试，作为合并门禁之一。
- 新增 `gen-verify` job：构建 release 二进制 → 在样例 C++ 项目上 `init` → 对生成目录 `cargo test`，闭环验证生成器输出。
- 复用现有的多平台矩阵（Linux / Windows MinGW / MSVC）；模板/接口相关示例若在某平台有已知 hicc 限制（参考 L3 中 macOS 虚函数跳过的先例），用同样的 `#[cfg]` / `cfg_attr(ignore)` 做平台跳过，并在 PR 描述中记录。
- 保持现有 `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 三道门禁不变。

### 6.3 验收标准

1. 所有现有 L1–L5 测试不回归。
2. 改造示例（024/025/026/027 + 虚函数/STL 选定项）的 L1/L2/L3/L_smoke 全绿。
3. L6 生成验证：至少对模板类、模板函数、接口三类各 1 个示例，`init` 产物 `cargo test` 通过。
4. `init` / `merge` 命令行为、参数、输出目录结构与 v5 完全一致。

---

## 7. 实现阶段划分

> 测试驱动：每个 Phase 完成的标准是"相关测试全绿"。

| 阶段 | 内容 | 依赖 | 产出 |
|------|------|------|------|
| **Phase A** | AST 层补齐：`ClassTemplate` 提取泛型 class 结构（成员/泛型参数）、`FunctionTemplate` 提取模板函数签名、实例化点收集 | 无 | `ast_parser` 新增模板信息字段 |
| **Phase B** | 提取器/生成器：模板类 → `import_class!` 泛型 + 实例化别名；模板函数 → `import_lib!` 实例化绑定；抽象类 → `#[interface]` | A | `generator` 新策略 |
| **Phase C** | 接口/`@make_proxy`、`@dynamic_cast`、私有析构等高级映射增强 | B | 高级特性映射 |
| **Phase D** | 冒烟测试生成器 `smoke_test_gen.rs` + `project_generator` 接入（幂等、可开关） | B | 生成 `tests/smoke.rs` |
| **Phase E** | examples 改造（024/025/026/027 + 虚函数/STL）+ 每示例 `tests/smoke.rs` + README | B,C,D | 升级后的示例 |
| **Phase F** | 测试/CI：黄金更新、L_smoke、L6 gen-verify、CI job | E | 绿色 CI |
| **Phase G** | 文档：INTRODUCTION / hicc.md / README 更新 | E,F | 文档对齐 |

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 模板成员方法签名含复杂依赖类型（`T::OutputRef`） | 自动映射可能不准 | 优先映射简单 `T`/`&T`/`*T`；复杂场景降级到 `hicc::cpp!` 内联 shim（保留 v5 兜底） |
| 模板函数实例化点跨 TU 不可见 | 漏绑定 | 以"实际被实例化的类型"为准（沿用 v5 实例化追踪）；模板声明仅用于生成泛型骨架，不强行枚举所有类型 |
| 示例从 `main.rs` 改为 `lib.rs`+`main.rs` 破坏 L1 黄金 | 测试回归 | L1 黄金提取目标同步改为 `lib.rs`；分批改造，逐示例验证 |
| 冒烟测试默认生成可能影响既有 `init` 输出对比测试 | 多 feature 集成测试回归 | 幂等生成 + 环境变量开关；更新受影响的集成测试断言 |
| `@make_proxy` / `@dynamic_cast` 平台差异 | 某些平台运行崩溃 | 参照 L3 macOS 虚函数先例做平台跳过，记录在案 |
| hicc 版本能力差异 | 绑定不被 hicc 支持 | 锁定 `hicc`/`hicc-build` 版本；以 `references/hicc` 实测能力为准 |

---

## 9. 不做的事（范围边界）

- 不改变 `init` / `merge` 两命令的使用方式与参数。
- 不追求 100% 自动消除所有降级（`[OP]`/`[VA]`/`[LM]` 等仍保留兜底策略）。
- 不生成业务语义等价的 Rust 实现，只生成 FFI 绑定与冒烟验证。
- 不引入与 hicc 无关的新依赖。
