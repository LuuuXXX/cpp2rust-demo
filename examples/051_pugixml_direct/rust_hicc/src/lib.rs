hicc::cpp! {
    #include "pugixml_direct.h"
    using xml_parse_result = pugi::xml_parse_result;
}

hicc::import_class! {
    #[cpp(class = "xml_parse_result")]
    pub class xml_parse_result {
        #[cpp(method = "const char* description() const")]
        pub fn description(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "pugixml_direct"]

    class xml_parse_result;

    #[cpp(func = "std::unique_ptr<xml_parse_result> hicc::make_unique<xml_parse_result>()")]
    pub fn xml_parse_result_new() -> xml_parse_result;
}
