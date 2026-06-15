hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <cstring>

    #include "namespace_nested.h"

    using ConfigManager = foo::bar::config::ConfigManager;
    using DataProcessor = foo::baz::DataProcessor;

    const char* get_version() {
        return "1.0.0";
    }

    int get_build_number() {
        return 42;
    }

    int string_length(const char* str) {
        return static_cast<int>(strlen(str));
    }
}

hicc::import_class! {
    #[cpp(class = "ConfigManager")]
    pub class ConfigManager {
        #[cpp(method = "void set_value(const char* key, int value)")]
        pub fn set_value(&mut self, key: *const i8, value: i32);

        #[cpp(method = "int get_value(const char* key) const")]
        pub fn get_value(&self, key: *const i8) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "DataProcessor")]
    pub class DataProcessor {
        #[cpp(method = "int process(int input) const")]
        pub fn process(&self, input: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "namespace_nested"]

    class ConfigManager;
    class DataProcessor;

    #[cpp(func = "std::unique_ptr<ConfigManager> hicc::make_unique<ConfigManager>()")]
    pub fn config_manager_new() -> ConfigManager;

    #[cpp(func = "std::unique_ptr<DataProcessor> hicc::make_unique<DataProcessor>()")]
    pub fn data_processor_new() -> DataProcessor;

    #[cpp(func = "const char* get_version()")]
    pub unsafe fn get_version() -> *const i8;

    #[cpp(func = "int get_build_number()")]
    pub fn get_build_number() -> i32;

    #[cpp(func = "int string_length(const char*)")]
    pub unsafe fn string_length(str: *const i8) -> i32;
}
