# 050_rapidjson_direct - RapidJSON Direct 模式绑定

真实第三方库（rapidjson）的 Direct 模式验证示例。

## C++ 特性

- Template instantiation（`GenericValue<UTF8<>, MemoryPoolAllocator<>>` → `Value`）
- Internal typedef（`MemberIterator`, `Member` 等）
- `= delete` 构造函数（`Document(const Document&) = delete`）
- 大量方法（30+ 方法）
- 多 TU（document.h + writer.h + prettywriter.h）

## 说明

本 example 使用 `references/rapidjson/` submodule 的头文件，不含手写 C++ 类。
`rust_hicc/src/lib.rs` 需手动编写（工具对 rapidjson 的 template/typedef 处理不完美）。
