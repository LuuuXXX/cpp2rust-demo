# clang AST 处理说明

`cpp2rust-demo` 会对选中的 `*.cpp2rust` 中间件执行：

```bash
clang -Xclang -ast-dump=json -fsyntax-only -x c++ -std=c++14 <file>.cpp2rust
```

并将原始 AST JSON 保存到：

```text
.cpp2rust/<feature>/ast/<stem>.ast.json
```

## 提取内容

`ast.rs` 对 clang 输出的 JSON 树进行深度优先遍历，从以下 AST 节点类型中提取信息：

| AST 节点类型 | 提取产物 | 说明 |
|-------------|---------|------|
| `FunctionDecl` | `FunctionIR`（自由函数） | 命名空间限定名、参数类型/返回类型、重载序号 |
| `CXXMethodDecl` | `FunctionIR`（类方法） | `const`/`static`/`virtual`/`pure virtual` / `&&` 标记 |
| `CXXConstructorDecl` | `CtorIR` | 主构造函数 (`ctor="..."`) + 额外构造工厂函数 |
| `CXXRecordDecl` | `ClassIR` | 公有方法、字段、基类、虚函数分类（全纯虚/混合/普通） |
| `ClassTemplateSpecializationDecl` | `ClassIR`（模板特化） | 需 AliasRegistry 中存在别名才能解锁 |
| `FunctionTemplateDecl` | `FunctionIR`（函数模板实例化） | 仅提取 AST 中可见的 concrete specialization |
| `TypedefDecl` / `TypeAliasDecl` | `AliasIR` + AliasRegistry 注册 | 简单类型别名生成 `pub type`；模板别名解锁模板提取 |
| `EnumDecl` | `EnumIR` | `enum` / `enum class`（scoped）；含重复值时额外生成 `pub const` |
| `FieldDecl` | `FieldIR` | 公有非静态实例字段，生成 `#[cpp(field)]` 读写访问器 |
| `VarDecl`（静态/全局） | `GlobalVarIR` | 命名空间级全局变量与类静态数据成员，生成 `#[cpp(data)]` |

## 过滤规则

提取过程会主动过滤以下内容（不产生任何输出）：

- **系统命名空间**：`std::`、`__gnu_cxx::`、`__1::`、`__detail::` 等（`is_system_namespace()`）
- **下划线前缀名称**：`__` 开头的 `FunctionDecl` / `CXXRecordDecl`（编译器内部实现）
- **复制/移动构造函数**：自动识别 `const T&` / `T&&` 签名并跳过
- **析构函数**：跳过并标记 `hicc_limitation`
- **运算符重载**：跳过提取，但收集 `OperatorShimIR` 并写入 `operator_shims.hpp` starter
- **类内 typedef / using**：`collect_alias_nodes()` 遇到 `CXXRecordDecl` 等类节点时**不递归**，防止类内 `typedef`（如 `rebind::other`）污染顶层别名注册表

## 函数类型字符串解析（`parse_fn_qual_type`）

clang 对函数类型 `qualType` 字段有两种序列化形式，取决于返回类型是否为指针：

| 情形 | clang 输出示例 | 分隔符 |
|------|--------------|--------|
| 非指针返回 | `"void (void *)"` | `" ("` — 返回类型与 `(` 之间有空格 |
| 指针返回 | `"void *(size_t)"` | 无空格 — `*` 直接连接 `(` |

`parse_fn_qual_type()` 通过查找字符串中第一个 `(` 来定位参数列表的起始位置（这在任何合法的 clang 函数类型字符串中都是参数列表分隔符），两种形式均可正确解析。

早期实现搜索 `" ("` 作为分隔符，导致 `void*(size_t)` 格式的函数（如 `void* Malloc(size_t)`、`void* Realloc(void*, size_t, size_t)` 等）被误判为 `unsupported_type` 而静默跳过，影响所有指针返回类型的实例方法提取。

## AliasRegistry

模板类提取依赖 `AliasRegistry`，在首次 AST 遍历时收集所有 `TypedefDecl`/`TypeAliasDecl`，
维护三张映射：

1. 裸模板名 → 所有别名（1:N，如 `"GenericDocument"` → `["Document", "FastDoc"]`）
2. 别名 → 完整限定类型（`"Document"` → `"rapidjson::GenericDocument<...>"`）
3. 完整限定类型 → 首个别名（精确反向查找）

`resolve_transitive()` 在收集完毕后执行传递性闭合，使 `using B = A; using A = T<...>` 这类链式别名也能正确解锁模板提取。

## 目的

AST 提取结果用于自动生成：

- `hicc::import_lib!` 自由函数与静态方法声明
- `hicc::import_class!` 类方法、字段、构造函数声明
- 枚举定义（`#[repr(C)] pub enum`）和类型别名（`pub type`）
- `init-interface-report.md` 接口清单（已提取 + 已跳过条目）
- `operator_shims.hpp` 运算符重载 shim starter
- `@dynamic_cast` / `@placement_new` / `hicc::RustAny` 注释骨架建议
