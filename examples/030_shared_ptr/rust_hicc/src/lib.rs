hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>
    #include <unordered_map>

    #include "shared_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "SharedData", destroy = "shareddata_delete")]
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
    #[cpp(class = "Cache", destroy = "cache_delete")]
    pub class Cache {
        #[cpp(method = "SharedData* get(const char* name)")]
        pub fn get(&mut self, name: *const i8) -> *mut SharedData;
    }
}

hicc::import_lib! {
    #![link_name = "shared_ptr"]

    class SharedData;
    class Cache;

    #[cpp(func = "SharedData* shareddata_new(const char*)")]
    pub unsafe fn shareddata_new(name: *const i8) -> SharedData;

    #[cpp(func = "Cache* cache_new()")]
    pub fn cache_new() -> Cache;

    #[cpp(func = "SharedData* cache_get(Cache* c, const char*)")]
    pub unsafe fn cache_get(c: *mut Cache, name: *const i8) -> *mut SharedData;
}
