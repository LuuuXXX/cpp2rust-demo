hicc::cpp! {
    #include "rapidjson_direct.h"
    using ParseResult = rapidjson::ParseResult;
}

hicc::import_class! {
    #[cpp(class = "ParseResult")]
    pub class ParseResult {
        #[cpp(method = "size_t Offset() const")]
        pub fn offset(&self) -> u64;

        #[cpp(method = "bool IsError() const")]
        pub fn is_error(&self) -> bool;

        #[cpp(method = "void Clear()")]
        pub fn clear(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "rapidjson_direct"]

    class ParseResult;

    #[cpp(func = "std::unique_ptr<ParseResult> hicc::make_unique<ParseResult>()")]
    pub fn parse_result_new() -> ParseResult;
}
