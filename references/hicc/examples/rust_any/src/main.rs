
hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/unordered_set.hpp>
    #include <hicc/std/vector.hpp>
    #include <hicc/std/array.hpp>
    #include <hicc/std/string.hpp>
    #include <hicc/rust_any.hpp>
    // 按需定义容器类型.
    typedef std::map<RustAny, std::string> CppMap;
    typedef std::unordered_set<RustAny> CppSet;
    typedef std::vector<RustAny> CppVec;
    typedef std::array<RustAny, 5> CppArray;
}

hicc::import_lib! {
    #![link_name = "example"]

    // 对应`c++`的容器类型
    class RustMap = hicc_std::map<hicc::RustKey<Key>, hicc_std::string>;
    class RustSet = hicc_std::set<hicc::RustHashKey<Key>>;
    class RustVec = hicc_std::vector<hicc::RustAny<Key>>;
    class RustArray = hicc_std::array<hicc::RustAny<Key>>;
    // 具体容器类型的创建接口.
    #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
    fn rustmap_new() -> RustMap;
    #[cpp(func = "std::unique_ptr<CppSet> hicc::make_unique<CppSet>()")]
    fn rustset_new() -> RustSet;
    #[cpp(func = "std::unique_ptr<CppVec> hicc::make_unique<CppVec>()")]
    fn rustvec_new() -> RustVec;
    #[cpp(func = "std::unique_ptr<CppArray> hicc::make_unique<CppArray>()")]
    fn rustarray_new() -> RustArray;
}

#[derive(Clone, PartialEq, PartialOrd, Hash)]
struct Key {
    val: i32,
    key: i32,
}

impl Key {
    fn new(val: i32, key: i32) -> Self {
        Self{ val, key }
    }
}

fn main() {
    let mut map = rustmap_new();
    let mut set = rustset_new();
    let mut vec = rustvec_new();
    let mut array = rustarray_new();

    let key = hicc::RustAny::new_clone(Key::new(1, 1));

    vec.push_back(&key);
    let back = vec.back().unwrap();
    println!("back.key = {}", back.key);

    // 不能直接访问缺省构造的`RustAny`.
    array.fill(&key);
    let val1 = array.get(1).unwrap();
    println!("array[1].key = {}", val1.key);

    let k1 = hicc::RustHashKey::new_clone(Key::new(1, 1));
    set.insert(&k1);
    println!("set.count = {}", set.count(&k1));

    let k2 = hicc::RustKey::new_clone(Key::new(1, 1));
    map.insert(&k2, &hicc_std::string::from(c"hello"));
    println!("map.count = {}", map.count(&k2));
}
