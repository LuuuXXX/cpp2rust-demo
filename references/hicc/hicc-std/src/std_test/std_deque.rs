use crate::{string, deque};

hicc::import_lib! {
    #![link_name = "hicc_std_deque"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/deque.hpp>
    }
    class string;

    /// 对应`std::deque<int>`
    pub class DequeInt = deque<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::deque<int>> hicc::make_unique<std::deque<int>>()")]
    #[member(class = DequeInt, method = new)]
    pub fn deque_int_new() -> DequeInt;
    /// 对应`std::deque<std::string>`
    pub class DequeString = deque<string>;
    #[cpp(func = "std::unique_ptr<std::deque<std::string>> hicc::make_unique<std::deque<std::string>>()")]
    #[member(class = DequeString, method = new)]
    pub fn deque_string_new() -> DequeString;
}
