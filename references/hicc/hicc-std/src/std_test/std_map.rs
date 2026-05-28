use crate::{string, map, multimap};

hicc::import_lib! {
    #![link_name = "hicc_std_map"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/map.hpp>
    }
    class string;

    /// 对应`std::map<std::string, std::string>`
    pub class MapStringString = map<string, string>;
    /// 对应`std::map<std::string, int>`
    pub class MapStringInt = map<string, hicc::Pod<i32>>;
    /// 对应`std::map<int, std::string>`
    pub class MapIntString = map<hicc::Pod<i32>, string>;
    /// 对应`std::map<int, int>`
    pub class MapIntInt = map<hicc::Pod<i32>, hicc::Pod<i32>>;

    #[member(class = MapStringString, method = new)]
    #[cpp(func = "std::map<std::string, std::string> hicc::make_constructor<std::map<std::string, std::string>>()")]
    pub fn map_string_string_new() -> MapStringString;
    #[member(class = MapStringInt, method = new)]
    #[cpp(func = "std::map<std::string, int> hicc::make_constructor<std::map<std::string, int>>()")]
    pub fn map_string_int_new() -> MapStringInt;
    #[member(class = MapIntString, method = new)]
    #[cpp(func = "std::map<int, std::string> hicc::make_constructor<std::map<int, std::string>>()")]
    pub fn map_int_string_new() -> MapIntString;
    #[member(class = MapIntInt, method = new)]
    #[cpp(func = "std::map<int, int> hicc::make_constructor<std::map<int, int>>()")]
    pub fn map_int_int_new() -> MapIntInt;

    /// 对应`std::multimap<std::string, std::string>`
    pub class MultiMapStringString = multimap<string, string>;
    /// 对应`std::multimap<std::string, int>`
    pub class MultiMapStringInt = multimap<string, hicc::Pod<i32>>;
    /// 对应`std::multimap<int, std::string>`
    pub class MultiMapIntString = multimap<hicc::Pod<i32>, string>;
    /// 对应`std::multimap<int, int>`
    pub class MultiMapIntInt = multimap<hicc::Pod<i32>, hicc::Pod<i32>>;

    #[member(class = MultiMapStringString, method = new)]
    #[cpp(func = "std::multimap<std::string, std::string> hicc::make_constructor<std::multimap<std::string, std::string>>()")]
    pub fn multimap_string_string_new() -> MultiMapStringString;
    #[member(class = MultiMapStringInt, method = new)]
    #[cpp(func = "std::multimap<std::string, int> hicc::make_constructor<std::multimap<std::string, int>>()")]
    pub fn multimap_string_int_new() -> MultiMapStringInt;
    #[member(class = MultiMapIntString, method = new)]
    #[cpp(func = "std::multimap<int, std::string> hicc::make_constructor<std::multimap<int, std::string>>()")]
    pub fn multimap_int_string_new() -> MultiMapIntString;
    #[member(class = MultiMapIntInt, method = new)]
    #[cpp(func = "std::multimap<int, int> hicc::make_constructor<std::multimap<int, int>>()")]
    pub fn multimap_int_int_new() -> MultiMapIntInt;
}