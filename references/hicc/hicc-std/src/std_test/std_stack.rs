use crate::{string, stack};

hicc::import_lib! {
    #![link_name = "hicc_std_stack"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/stack.hpp>
    }
    class string;

    /// 对应`std::stack<int>`
    pub class StackInt = stack<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::stack<int>> hicc::make_unique<std::stack<int>>()")]
    #[member(class = StackInt, method = new)]
    pub fn stack_int_new() -> StackInt;
    /// 对应`std::stack<std::string>`
    pub class StackString = stack<string>;
    #[cpp(func = "std::unique_ptr<std::stack<std::string>> hicc::make_unique<std::stack<std::string>>()")]
    #[member(class = StackString, method = new)]
    pub fn stack_string_new() -> StackString;
}