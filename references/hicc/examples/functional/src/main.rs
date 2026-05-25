hicc::cpp! {
    #include <functional>

    int foo(int v, std::function<int(int)> func) {
        return func(v);
    }

    std::function<int(int)> bar(int v) {
        return [=](int val) -> int {
            std::cout << "c++ functional, input = " << val << ", bar input = " << v << std::endl;
            return val + v;
        };
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int foo(int, std::function<int(int)>)")]
    fn foo(v: i32, func: hicc::Function<fn(i32) -> i32>) -> i32;

    #[cpp(func = "std::function<int(int)> bar(int)")]
    fn bar(v: i32) -> hicc::Function<fn(i32) -> i32>;
}

fn main() {
    let val = foo(10, |v: i32| -> i32 {
        println!("rust v = {v}");
        return v + 10;
    }.into());

    println!("foo return {val}");

    let fun = bar(100).into();
    let val = fun(1);
    println!("bar::fun::return {val}");
}

