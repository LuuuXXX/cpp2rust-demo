hicc::cpp! {
    #include <iostream>
    #include <cstring>

    #include "mutable_member.h"
}

hicc::import_class! {
    #[cpp(class = "DataFetcher", destroy = "datafetcher_delete")]
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

    #[cpp(func = "DataFetcher* datafetcher_new(const char*)")]
    pub unsafe fn datafetcher_new(name: *const i8) -> DataFetcher;
}
