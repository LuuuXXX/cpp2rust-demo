hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>
    #include <unordered_map>

    #include "shared_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "SharedData")]
    pub class SharedData {
        #[cpp(method = "int useCount() const")]
        pub fn use_count(&self) -> i32;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "SharedData* clone() const")]
        pub fn clone(&self) -> *mut SharedData;

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "Cache")]
    pub class Cache {
        #[cpp(method = "SharedData* get(const char* name)")]
        pub fn get(&mut self, name: *const i8) -> *mut SharedData;
    }
}

hicc::import_lib! {
    #![link_name = "shared_ptr"]

    class SharedData;
    class Cache;

    #[cpp(func = "std::unique_ptr<SharedData> std::make_unique<SharedData>(const char*)")]
    pub unsafe fn shared_data_new_with_n(n: *const i8) -> SharedData;

    #[cpp(func = "std::unique_ptr<Cache> hicc::make_unique<Cache>()")]
    pub fn cache_new() -> Cache;
}
