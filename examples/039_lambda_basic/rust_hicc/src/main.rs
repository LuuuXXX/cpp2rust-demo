hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <algorithm>

    typedef int (*IntBinaryOp)(int, int);

    class LambdaWrapperImpl {
    public:
        std::function<int(int, int)> fn;
    public:
        LambdaWrapperImpl(int (*fn_ptr)(int, int)) : fn(fn_ptr) {}
        ~LambdaWrapperImpl() {}
    };

    class StateLambdaImpl {
    public:
        int value;
        std::function<int(int)> adder;
    public:
        StateLambdaImpl(int initial) : value(initial), adder([this](int delta) { return value += delta; }) {}
        ~StateLambdaImpl() {}
    };

    class ComparatorImpl {
    public:
        std::function<int(int, int)> cmp;
    public:
        ComparatorImpl(int (*cmp_fn)(int, int)) : cmp(cmp_fn) {}
        ~ComparatorImpl() {}
    };

    struct LambdaWrapper {
    public:
        LambdaWrapperImpl* impl;
        LambdaWrapper(int (*fn)(int, int)) : impl(new LambdaWrapperImpl(fn)) {}
        ~LambdaWrapper() { delete impl; }
        int invoke(int a, int b) { return impl->fn(a, b); }
    };

    struct StateLambda {
    public:
        StateLambdaImpl* impl;
        StateLambda(int initial_value) : impl(new StateLambdaImpl(initial_value)) {}
        ~StateLambda() { delete impl; }
        int get_value() const { return impl->value; }
        int add(int delta) { return impl->adder(delta); }
    };

    struct Comparator {
    public:
        ComparatorImpl* impl;
        Comparator(int (*cmp)(int, int)) : impl(new ComparatorImpl(cmp)) {}
        ~Comparator() { delete impl; }
        int compare(int a, int b) const { return impl->cmp(a, b); }
    };

    int add_impl(int a, int b) {
        std::cout << "add lambda called: " << a << " + " << b << std::endl;
        return a + b;
    }

    int multiply_impl(int a, int b) {
        std::cout << "multiply lambda called: " << a << " * " << b << std::endl;
        return a * b;
    }

    int max_impl(int a, int b) {
        std::cout << "max lambda called: " << a << " vs " << b << std::endl;
        return std::max(a, b);
    }

    int apply_operation(int a, int b, IntBinaryOp op) {
        if (op) return op(a, b);
        return 0;
    }

    int apply_twice(int x, IntBinaryOp op) {
        if (op) return op(op(x, x), x);
        return x;
    }

    LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int)) {
        return new LambdaWrapper(fn);
    }

    void lambda_wrapper_delete(LambdaWrapper* self) {
        delete self;
    }

    LambdaWrapper* make_add_lambda() {
        return new LambdaWrapper(add_impl);
    }

    LambdaWrapper* make_multiply_lambda() {
        return new LambdaWrapper(multiply_impl);
    }

    LambdaWrapper* make_max_lambda() {
        return new LambdaWrapper(max_impl);
    }

    StateLambda* state_lambda_new(int initial_value) {
        return new StateLambda(initial_value);
    }

    void state_lambda_delete(StateLambda* self) {
        delete self;
    }

    Comparator* comparator_new(int (*cmp)(int, int)) {
        return new Comparator(cmp);
    }

    Comparator* comparator_new_add() {
        return new Comparator(add_impl);
    }

    void comparator_delete(Comparator* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "LambdaWrapper")]
    class LambdaWrapper {
        #[cpp(method = "int invoke(int, int)")]
        fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda")]
    class StateLambda {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int add(int)")]
        fn add(&mut self, delta: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Comparator")]
    class Comparator {
        #[cpp(method = "int compare(int, int) const")]
        fn compare(&self, a: i32, b: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;
    class Comparator;

    #[cpp(func = "void lambda_wrapper_delete(LambdaWrapper* self)")]
    unsafe fn lambda_wrapper_delete(self_: *mut LambdaWrapper);

    #[cpp(func = "LambdaWrapper* make_add_lambda()")]
    fn make_add_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_multiply_lambda()")]
    fn make_multiply_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_max_lambda()")]
    fn make_max_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "StateLambda* state_lambda_new(int)")]
    fn state_lambda_new(initial_value: i32) -> *mut StateLambda;

    #[cpp(func = "void state_lambda_delete(StateLambda* self)")]
    unsafe fn state_lambda_delete(self_: *mut StateLambda);

    #[cpp(func = "Comparator* comparator_new_add()")]
    fn comparator_new_add() -> *mut Comparator;

    #[cpp(func = "void comparator_delete(Comparator* self)")]
    unsafe fn comparator_delete(self_: *mut Comparator);

    #[cpp(func = "int add_impl(int, int)")]
    fn add_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int multiply_impl(int, int)")]
    fn multiply_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_impl(int, int)")]
    fn max_impl(a: i32, b: i32) -> i32;
}

fn main() {
    println!("=== 039_lambda_basic - Lambda 表达式 ===\n");

    println!("--- Direct function calls ---");
    println!("add_impl(3, 4) = {}", add_impl(3, 4));
    println!("multiply_impl(3, 4) = {}", multiply_impl(3, 4));
    println!("max_impl(3, 4) = {}", max_impl(3, 4));

    println!("\n--- LambdaWrapper Demo ---");
    let mut add_wrapper = make_add_lambda();
    println!("add invoke(5, 6) = {}", add_wrapper.invoke(5, 6));
    unsafe { lambda_wrapper_delete(&add_wrapper); }

    let mut mul_wrapper = make_multiply_lambda();
    println!("multiply invoke(5, 6) = {}", mul_wrapper.invoke(5, 6));
    unsafe { lambda_wrapper_delete(&mul_wrapper); }

    println!("\n--- StateLambda Demo ---");
    let mut state = state_lambda_new(10);
    println!("initial value = {}", state.get_value());
    println!("add(5) = {}", state.add(5));
    println!("add(3) = {}", state.add(3));
    unsafe { state_lambda_delete(&state); }

    println!("\n--- Comparator Demo ---");
    let mut cmp = comparator_new_add();
    println!("compare(2, 3) = {}", cmp.compare(2, 3));
    unsafe { comparator_delete(&cmp); }

    println!("\nRust FFI: Lambda 表达式映射");
    println!("1. 函数指针可以通过 FFI 传递");
    println!("2. 捕获状态的 lambda 需要包装在类中");
    println!("3. 此示例展示基本的类封装模式");
}



