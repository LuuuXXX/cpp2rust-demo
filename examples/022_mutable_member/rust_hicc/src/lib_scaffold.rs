// 此文件为 cpp2rust-demo 工具对 022_mutable_member 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 DataFetcher 生成 import_class! + make_unique 工厂，
// 使用 std::make_unique<T>(args) 模板函数绑定（无法直接通过 hicc 导出模板，
// 实际 lib.rs 用手动 C++ 包装函数替代）。
hicc::cpp! {
    #include <cstring>
    #include "mutable_member.h"
}

hicc::import_class! {
    #[cpp(class = "DataFetcher")]
    pub class DataFetcher {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "int getCacheCount() const")]
        pub fn get_cache_count(&self) -> i32;

        #[cpp(method = "void refresh()")]
        pub fn refresh(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "mutable_member"]

    class DataFetcher;

    #[cpp(func = "std::unique_ptr<DataFetcher> std::make_unique<DataFetcher>(const char*)")]
    pub unsafe fn data_fetcher_new_with_n(n: *const i8) -> DataFetcher;
}
