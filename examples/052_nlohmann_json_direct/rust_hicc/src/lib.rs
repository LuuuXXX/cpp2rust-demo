hicc::cpp! {
    #include "nlohmann_json_direct.h"
}

hicc::import_lib! {
    #![link_name = "nlohmann_json_direct"]

    #[cpp(func = "const char* nlohmann_json_parse_and_dump(const char*)")]
    pub unsafe fn nlohmann_json_parse_and_dump(json_str: *const i8) -> *const i8;
}
