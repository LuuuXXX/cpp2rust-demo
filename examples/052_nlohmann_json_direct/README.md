# 052_nlohmann_json_direct - nlohmann-json Direct 模式绑定

真实第三方库（nlohmann-json）的 Direct 模式验证示例。

## C++ 特性

- 单头文件（`json.hpp`）
- Template class（`basic_json<ObjectType, ArrayType, StringType, BooleanType, NumberIntegerType, NumberUnsignedType, NumberFloatType, AllocatorType, JSONSerializer>` → `json`）
- 现代 C++（C++17, `if constexpr`, `std::optional`）
- 大量方法（50+ 方法）

## 说明

本 example 使用 `references/nlohmann-json/` submodule 的单头文件，不含手写 C++ 类。
`rust_hicc/src/lib.rs` 需手动编写（工具对 nlohmann-json 的 template 处理不完美）。
