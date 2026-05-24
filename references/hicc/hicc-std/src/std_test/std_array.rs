use crate::{string, array};

hicc::import_lib! {
    #![link_name = "hicc_std_array"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/array.hpp>
    }
    class string;

    /// 对应`std::array<int, N>`. 其中`N`由具体创建函数决定.
    pub class ArrayInt = array<hicc::Pod<i32>>;
    /// 对应`std::array<int, 10>`
    #[cpp(func = "std::unique_ptr<std::array<int, 10>> hicc::make_unique<std::array<int, 10>>()")]
    #[member(class = ArrayInt, method = new_10)]
    pub fn array_int_10_new() -> ArrayInt;
    /// 对应`std::array<std::string, N>`. 其中`N`由具体创建函数决定.
    pub class ArrayString = array<string>;
    /// 对应`std::array<std::string, 10>`
    #[cpp(func = "std::unique_ptr<std::array<std::string, 10>> hicc::make_unique<std::array<std::string, 10>>()")]
    #[member(class = ArrayString, method = new_10)]
    pub fn array_string_10_new() -> ArrayString;
}
