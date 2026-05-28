use crate::{string, set, multiset};

hicc::import_lib! {
    #![link_name = "hicc_std_set"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/set.hpp>
    }
    class string;

    /// 对应`std::set<int>`
    pub class SetInt = set<hicc::Pod<i32>>;
    /// 对应`std::set<std::string>`
    pub class SetString = set<string>;
    /// 对应`std::multiset<int>`
    pub class MultiSetInt = multiset<hicc::Pod<i32>>;
    /// 对应`std::multiset<std::string>`
    pub class MultiSetString = multiset<string>;

    #[cpp(func = "std::unique_ptr<std::set<int>> hicc::make_unique<std::set<int>>()")]
    #[member(class = SetInt, method = new)]
    pub fn set_int_new() -> SetInt;
    #[cpp(func = "std::unique_ptr<std::set<std::string>> hicc::make_unique<std::set<std::string>>()")]
    #[member(class = SetString, method = new)]
    pub fn set_string_new() -> SetString;

    #[cpp(func = "std::unique_ptr<std::multiset<int>> hicc::make_unique<std::multiset<int>>()")]
    #[member(class = MultiSetInt, method = new)]
    pub fn multiset_int_new() -> MultiSetInt;
    #[cpp(func = "std::unique_ptr<std::multiset<std::string>> hicc::make_unique<std::multiset<std::string>>()")]
    #[member(class = MultiSetString, method = new)]
    pub fn multiset_string_new() -> MultiSetString;
}