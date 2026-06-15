// 013_inheritance_single 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「单继承命名空间类」默认生成的 hicc 骨架。
// 本示例 Animal/Dog 的构造函数与成员均以 `std::string` 为参数/返回，工具默认
// 不自动映射 `std::string`（需 `hicc_std::string` 手写补全），故默认支架仅含
// `cpp!` 头块；完整绑定见手写 `lib.rs`。

hicc::cpp! {
    #include "inheritance_single.h"
}
