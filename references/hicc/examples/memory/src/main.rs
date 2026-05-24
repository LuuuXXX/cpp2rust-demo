hicc::cpp! {

#include <hicc/std/memory.hpp>
#include <hicc/std/string.hpp>

struct my_deleter {
    template<class T>
    void operator()(T* p) {
        std::cout << "my_delete: " << p << std::endl;
        delete p;
    }
};

std::shared_ptr<int> make_shared_int(int v) {
        return std::make_shared<int>(v + 100);
}

std::shared_ptr<std::string> make_shared_str(const std::string& hello) {
        return std::make_shared<std::string>(hello + " world");
}

std::unique_ptr<int, my_deleter> make_unique_int(int v) {
        return std::unique_ptr<int, my_deleter>(new int(v + 100));
}

std::unique_ptr<std::string, my_deleter> make_unique_str(const std::string& hello) {
        return std::unique_ptr<std::string, my_deleter>(new std::string(hello + " world"));
}

std::unique_ptr<int> make_unique_int_def(int v) {
        return std::make_unique<int>(v + 100);
}

std::unique_ptr<std::string> make_unique_str_def(const std::string& hello) {
        return std::make_unique<std::string>(hello + " world");
}

}

hicc::import_lib! {
    #![link_name = "memory"]

    class string = hicc_std::string;

    #[cpp(func = "std::shared_ptr<int> make_shared_int(int)")]
    fn int_sptr(v: i32) -> hicc::shared_ptr<hicc::Pod<i32>>;

    #[cpp(func = "std::shared_ptr<std::string> make_shared_str(const std::string&)")]
    fn str_sptr(s: &hicc_std::string) -> hicc::shared_ptr<string>;

    #[cpp(func = "std::unique_ptr<int, my_deleter> make_unique_int(int)")]
    fn int_uptr(v: i32) -> hicc::unique_ptr<hicc::Pod<i32>>;

    #[cpp(func = "std::unique_ptr<std::string, my_deleter> make_unique_str(const std::string&)")]
    fn str_uptr(s: &hicc_std::string) -> hicc::unique_ptr<string>;

    #[cpp(func = "std::unique_ptr<int> make_unique_int_def(int)")]
    fn int_uptr_def(v: i32) -> hicc::unique_ptr<hicc::Pod<i32>>;

    #[cpp(func = "std::unique_ptr<std::string> make_unique_str_def(const std::string&)")]
    fn str_uptr_def(s: &hicc_std::string) -> string;
}

fn main() {
    let weak;
    {
    let iptr = int_sptr(100);
    println!("int_sptr = {}", unsafe { *iptr.get() });
    weak = iptr.weak();
    println!("weak_ptr.expired = {}", weak.expired());
    let iptr = weak.lock();
    println!("int_sptr from weak = {}", unsafe { *iptr.get() });
    }
    println!("weak_ptr.expired should be true = {}", weak.expired());
    println!("weak_ptr.lock.is_empty should be true = {}", weak.lock().is_empty());

    let sptr = str_sptr(&hicc_std::string::from(c"hi"));
    let sptr = sptr.get();

    println!("str_sptr = {:?}", sptr.as_cstr());

    let iptr = int_uptr(100);
    println!("int_uptr = {}, is_empty = {}", unsafe { *iptr.get() }, iptr.is_empty());

    let sptr = str_uptr(&hicc_std::string::from(c"hi"));
    let sptr = sptr.get();

    println!("str_uptr = {:?}", sptr.as_cstr());

    let iptr = int_uptr_def(100);
    println!("int_uptr_def = {}", unsafe { *iptr.get() });

    let sptr = str_uptr_def(&hicc_std::string::from(c"hi"));
    println!("str_uptr_def = {:?}", sptr.as_cstr());
}
