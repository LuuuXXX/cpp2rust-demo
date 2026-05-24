use crate::{string, unordered_map, unordered_multimap};

hicc::import_lib! {
    #![link_name = "hicc_std_unordered_map"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/unordered_map.hpp>
    }
    class string;

    /// 对应`std::unordered_map<std::string, std::string>`
    pub class UnorderedMapStringString = unordered_map<string, string>;
    /// 对应`std::unordered_map<std::string, int>`
    pub class UnorderedMapStringInt = unordered_map<string, hicc::Pod<i32>>;
    /// 对应`std::unordered_map<int, std::string>`
    pub class UnorderedMapIntString = unordered_map<hicc::Pod<i32>, string>;
    /// 对应`std::unordered_map<int, int>`
    pub class UnorderedMapIntInt = unordered_map<hicc::Pod<i32>, hicc::Pod<i32>>;

    #[member(class = UnorderedMapStringString, method = new)]
    #[cpp(func = "std::unordered_map<std::string, std::string> hicc::make_constructor<std::unordered_map<std::string, std::string>>()")]
    pub fn unordered_map_string_string_new() -> UnorderedMapStringString;
    #[member(class = UnorderedMapStringInt, method = new)]
    #[cpp(func = "std::unordered_map<std::string, int> hicc::make_constructor<std::unordered_map<std::string, int>>()")]
    pub fn unordered_map_string_int_new() -> UnorderedMapStringInt;
    #[member(class = UnorderedMapIntString, method = new)]
    #[cpp(func = "std::unordered_map<int, std::string> hicc::make_constructor<std::unordered_map<int, std::string>>()")]
    pub fn unordered_map_int_string_new() -> UnorderedMapIntString;
    #[member(class = UnorderedMapIntInt, method = new)]
    #[cpp(func = "std::unordered_map<int, int> hicc::make_constructor<std::unordered_map<int, int>>()")]
    pub fn unordered_map_int_int_new() -> UnorderedMapIntInt;

    /// 对应`std::unordered_multimap<std::string, std::string>`
    pub class UnorderedMultiMapStringString = unordered_multimap<string, string>;
    /// 对应`std::unordered_multimap<std::string, int>`
    pub class UnorderedMultiMapStringInt = unordered_multimap<string, hicc::Pod<i32>>;
    /// 对应`std::unordered_multimap<int, std::string>`
    pub class UnorderedMultiMapIntString = unordered_multimap<hicc::Pod<i32>, string>;
    /// 对应`std::unordered_multimap<int, int>`
    pub class UnorderedMultiMapIntInt = unordered_multimap<hicc::Pod<i32>, hicc::Pod<i32>>;

    #[member(class = UnorderedMultiMapStringString, method = new)]
    #[cpp(func = "std::unordered_multimap<std::string, std::string> hicc::make_constructor<std::unordered_multimap<std::string, std::string>>()")]
    pub fn unordered_multimap_string_string_new() -> UnorderedMultiMapStringString;
    #[member(class = UnorderedMultiMapStringInt, method = new)]
    #[cpp(func = "std::unordered_multimap<std::string, int> hicc::make_constructor<std::unordered_multimap<std::string, int>>()")]
    pub fn unordered_multimap_string_int_new() -> UnorderedMultiMapStringInt;
    #[member(class = UnorderedMultiMapIntString, method = new)]
    #[cpp(func = "std::unordered_multimap<int, std::string> hicc::make_constructor<std::unordered_multimap<int, std::string>>()")]
    pub fn unordered_multimap_int_string_new() -> UnorderedMultiMapIntString;
    #[member(class = UnorderedMultiMapIntInt, method = new)]
    #[cpp(func = "std::unordered_multimap<int, int> hicc::make_constructor<std::unordered_multimap<int, int>>()")]
    pub fn unordered_multimap_int_int_new() -> UnorderedMultiMapIntInt;
}