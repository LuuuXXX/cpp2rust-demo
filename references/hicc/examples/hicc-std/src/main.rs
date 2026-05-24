use hicc::AbiClass;

hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/string.hpp>
    // 按需定义容器类型.
    typedef std::map<int, std::string> CppMap;
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的`MyMap`
    class RustMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;
    #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
    fn rustmap_new() -> RustMap;
}

fn main() {
    let mut map = rustmap_new();
    let name = hicc_std::string::from(c"hello");
    map.insert(&0, &name);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&0), Some(name.as_ref()));
}
