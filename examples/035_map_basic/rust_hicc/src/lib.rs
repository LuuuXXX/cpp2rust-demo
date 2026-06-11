hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <map>
    #include <string>
    #include <cstring>

    #include "map_basic.h"
}

hicc::import_class! {
    #[cpp(class = "StringIntMap", destroy = "string_int_map_delete")]
    pub class StringIntMap {
        #[cpp(method = "bool insert(const char* key, int val)")]
        pub fn insert(&mut self, key: *const i8, val: i32) -> bool;

        #[cpp(method = "int get(const char* key) const")]
        pub fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "void set(const char* key, int val)")]
        pub fn set(&mut self, key: *const i8, val: i32);

        #[cpp(method = "bool erase(const char* key)")]
        pub fn erase(&mut self, key: *const i8) -> bool;

        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "IntStringMap", destroy = "int_string_map_delete")]
    pub class IntStringMap {
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "map_basic"]

    class StringIntMap;
    class IntStringMap;

    #[cpp(func = "StringIntMap* string_int_map_new()")]
    pub fn string_int_map_new() -> StringIntMap;

    #[cpp(func = "IntStringMap* int_string_map_new()")]
    pub fn int_string_map_new() -> IntStringMap;
}
