hicc::cpp! {
    #include <cstring>

    #include "mutable_member.h"
    std::unique_ptr<DataFetcher> _cpp2rust_make_unique_data_fetcher_with_n(const char* n) { return std::make_unique<DataFetcher>(n); }
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

    #[cpp(func = "std::unique_ptr<DataFetcher> _cpp2rust_make_unique_data_fetcher_with_n(const char*)")]
    pub unsafe fn data_fetcher_new_with_n(n: *const i8) -> DataFetcher;
}
