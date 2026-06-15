# 051_pugixml_direct - pugixml Direct 模式绑定

真实第三方库（pugixml）的 Direct 模式验证示例。

## C++ 特性

- 多层继承（`xml_document : xml_node`）
- 拷贝语义（`xml_document` non-copyable, `xml_node` value semantics）
- 命名空间（`pugi::`）
- 大量方法（20+ 方法）

## 说明

本 example 使用 `references/pugixml/` submodule 的头文件，不含手写 C++ 类。
`rust_hicc/src/lib.rs` 需手动编写（工具对 pugixml 的继承/命名空间处理可能不完美）。
