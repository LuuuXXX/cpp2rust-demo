use crate::{string, forward_list};

hicc::import_lib! {
    #![link_name = "hicc_std_forward_list"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/forward_list.hpp>
    }
    class string;

    /// 对应`std::forward_list<int>`
    pub class ForwardListInt = forward_list<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::forward_list<int>> hicc::make_unique<std::forward_list<int>>()")]
    #[member(class = ForwardListInt, method = new)]
    pub fn forward_list_int_new() -> ForwardListInt;

    /// 对应`std::forward_list<std::string>`
    pub class ForwardListString = forward_list<string>;
    #[cpp(func = "std::unique_ptr<std::forward_list<std::string>> hicc::make_unique<std::forward_list<std::string>>()")]
    #[member(class = ForwardListString, method = new)]
    pub fn forward_list_string_new() -> ForwardListString;
}