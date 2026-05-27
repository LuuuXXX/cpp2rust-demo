hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <algorithm>

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
    };

    struct StateLambda {
    public:
        StateLambdaImpl* impl;
        StateLambda(int initial_value) : impl(new StateLambdaImpl(initial_value)) {}
        ~StateLambda() { delete impl; }
    };

    struct Comparator {
    public:
        ComparatorImpl* impl;
        Comparator(int (*cmp)(int, int)) : impl(new ComparatorImpl(cmp)) {}
        ~Comparator() { delete impl; }
    };

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

    void comparator_delete(Comparator* self) {
        delete self;
    }

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
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;
    class Comparator;

    #[cpp(func = "int apply_operation(int, int, IntBinaryOp)")]
    fn apply_operation(a: i32, b: i32, op: IntBinaryOp) -> i32;

    #[cpp(func = "int apply_twice(int, IntBinaryOp)")]
    fn apply_twice(x: i32, op: IntBinaryOp) -> i32;

    #[cpp(func = "LambdaWrapper* lambda_wrapper_new(int (*)(int, int))")]
    fn lambda_wrapper_new(fn_: int (*)(int, int)) -> *mut LambdaWrapper;

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

    #[cpp(func = "Comparator* comparator_new(int (*)(int, int))")]
    fn comparator_new(cmp: int (*)(int, int)) -> *mut Comparator;

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

    // Simple wrapper demo
    println!("--- BinaryOp Demo ---");
    let mut op = binary_op_new();

    // Store some operations
    op.store(10, 20, 30);
    println!("Stored: a={}, b={}, result={}", op.get_a(), op.get_b(), op.get_result());

    op.store(5, 3, 8);
    println!("Stored: a={}, b={}, result={}", op.get_a(), op.get_b(), op.get_result());

    unsafe {
        binary_op_delete(&op);
    }

    println!("\nRust FFI: Lambda 表达式映射");
    println!("1. 函数指针可以通过 FFI 传递");
    println!("2. 捕获状态的 lambda 需要包装在类中");
    println!("3. 此示例展示基本的类封装模式");
}


