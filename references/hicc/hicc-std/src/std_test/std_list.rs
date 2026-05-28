use crate::{string, list};

hicc::import_lib! {
    #![link_name = "hicc_std_list"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/list.hpp>
    }
    class string;

    /// 对应`std::list<int>`
    pub class ListInt = list<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::list<int>> hicc::make_unique<std::list<int>>()")]
    #[member(class = ListInt, method = new)]
    pub fn list_int_new() -> ListInt;
    /// 对应`std::list<std::string>`
    pub class ListString = list<string>;
    #[cpp(func = "std::unique_ptr<std::list<std::string>> hicc::make_unique<std::list<std::string>>()")]
    #[member(class = ListString, method = new)]
    pub fn list_string_new() -> ListString;
}