//! 后处理器模块（Phase 4）
//!
//! 在 `FfiSpec` 从 AST 提取完毕后、代码生成开始前，对 IR 进行自动化修正。
//!
//! ## 触发时机
//!
//! 后处理器在 `extractor::extract()` 末尾按固定顺序调用：
//! 先 [`diamond_handler`]，再 [`operator_handler`]。
//!
//! ## 子模块分工
//!
//! - [`diamond_handler`]：检测并处理 C++ 菱形虚继承（diamond virtual inheritance）。
//!   当一个类通过多条路径继承同一个虚基类时，hicc 的布局计算无法正确处理；
//!   该处理器为受影响的类生成 const-ptr shim 函数，并在 `FfiSpec` 中替换原有绑定。
//! - [`operator_handler`]：将 C++ 运算符重载转换为命名 shim 函数（如 `{class}_add`、
//!   `{class}_eq`），并从 `import_class!` 中移除含类类型参数的方法。
//!   触发条件：函数名以 `{class_snake}_` 为前缀且匹配 `BINARY_OPS`/`UNARY_OPS` 列表。
//!
//! ## 扩展指引
//!
//! 新增后处理器时，在本模块添加 `pub mod xxx_handler;` 声明，并在
//! `extractor::extract()` 末尾的后处理器链中调用 `xxx_handler::apply(&mut spec, ast, &functions)`。

pub mod diamond_handler;
pub mod operator_handler;
