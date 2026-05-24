use crate::{string, unordered_set, unordered_multiset};

hicc::import_lib! {
    #![link_name = "hicc_std_unordered_set"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/unordered_set.hpp>
    }
    class string;

    /// 对应`std::unordered_set<int>`
    pub class UnorderedSetInt = unordered_set<hicc::Pod<i32>>;
    /// 对应`std::unordered_set<std::string>`
    pub class UnorderedSetString = unordered_set<string>;
    /// 对应`std::unordered_multiset<int>`
    pub class UnorderedMultiSetInt = unordered_multiset<hicc::Pod<i32>>;
    /// 对应`std::unordered_multiset<std::string>`
    pub class UnorderedMultiSetString = unordered_multiset<string>;

    #[member(class = UnorderedSetInt, method = new)]
    #[cpp(func = "std::unordered_set<int> hicc::make_constructor<std::unordered_set<int>>()")]
    pub fn unordered_set_int_new() -> UnorderedSetInt;
    #[member(class = UnorderedSetString, method = new)]
    #[cpp(func = "std::unordered_set<std::string> hicc::make_constructor<std::unordered_set<std::string>>()")]
    pub fn unordered_set_string_new() -> UnorderedSetString;

    #[member(class = UnorderedMultiSetInt, method = new)]
    #[cpp(func = "std::unordered_multiset<int> hicc::make_constructor<std::unordered_multiset<int>>()")]
    pub fn unordered_multiset_int_new() -> UnorderedMultiSetInt;
    #[member(class = UnorderedMultiSetString, method = new)]
    #[cpp(func = "std::unordered_multiset<std::string> hicc::make_constructor<std::unordered_multiset<std::string>>()")]
    pub fn unordered_multiset_string_new() -> UnorderedMultiSetString;
}