use crate::{string, vector, VecBool};

hicc::import_lib! {
    #![link_name = "hicc_std_vector"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/vector.hpp>
    }
    class string;

    /// 对应`std::vector<int>`
    pub class VecInt = vector<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::vector<int>> hicc::make_unique<std::vector<int>>()")]
    #[member(class = VecInt, method = new)]
    pub fn vec_int_new() -> VecInt;

    /// 对应`std::vector<std::string>`
    pub class VecString = vector<string>;
    #[cpp(func = "std::unique_ptr<std::vector<std::string>> hicc::make_unique<std::vector<std::string>>()")]
    #[member(class = VecString, method = new)]
    pub fn vec_string_new() -> VecString;

    /// 对应`std::vector<std::vector<std::string>>`
    pub class VecVecString = vector<vector<string>>;
    #[cpp(func = "std::unique_ptr<std::vector<std::vector<std::string>>> hicc::make_unique<std::vector<std::vector<std::string>>>()")]
    #[member(class = VecVecString, method = new)]
    pub fn vec_vec_string_new() -> VecVecString;

    #[cpp(func = "std::unique_ptr<std::vector<bool>> hicc::make_unique<std::vector<bool>>()")]
    #[member(class = VecBool, method = new)]
    pub fn vec_bool_new() -> VecBool;
}