//! 043_namespace_nested: 嵌套命名空间类直接绑定。
//!
//! `ConfigManager` / `DataProcessor` 保持 C++ 嵌套命名空间 `foo::bar::config`
//! 与 `foo::baz`，hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "namespace_nested.h"
}

hicc::import_class! {
    #[cpp(class = "foo::bar::config::ConfigManager")]
    pub class ConfigManager {
        #[cpp(method = "void set_value(const char* key, int value)")]
        pub fn set_value(&mut self, key: *const i8, value: i32);

        #[cpp(method = "int get_value(const char* key) const")]
        pub fn get_value(&self, key: *const i8) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        pub fn new() -> Self { config_manager_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "foo::baz::DataProcessor")]
    pub class DataProcessor {
        #[cpp(method = "int process(int input) const")]
        pub fn process(&self, input: i32) -> i32;

        pub fn new() -> Self { data_processor_new() }
    }
}

hicc::import_lib! {
    #![link_name = "namespace_nested"]

    #[cpp(func = "std::unique_ptr<foo::bar::config::ConfigManager> hicc::make_unique<foo::bar::config::ConfigManager>()")]
    pub fn config_manager_new() -> ConfigManager;

    #[cpp(func = "std::unique_ptr<foo::baz::DataProcessor> hicc::make_unique<foo::baz::DataProcessor>()")]
    pub fn data_processor_new() -> DataProcessor;

    #[cpp(func = "const char* foo::get_version()")]
    pub unsafe fn get_version() -> *const i8;

    #[cpp(func = "int foo::get_build_number()")]
    pub fn get_build_number() -> i32;
}
