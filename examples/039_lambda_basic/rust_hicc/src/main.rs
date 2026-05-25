hicc::cpp! {
    #include <iostream>

    // Wrapper that stores a simple binary operation result
    class BinaryOp {
    public:
        int last_a;
        int last_b;
        int last_result;
        BinaryOp() : last_a(0), last_b(0), last_result(0) {}
        void store(int a, int b, int result) { last_a = a; last_b = b; last_result = result; }
        int get_a() const { return last_a; }
        int get_b() const { return last_b; }
        int get_result() const { return last_result; }
    };

    BinaryOp* binary_op_new() { return new BinaryOp(); }
    void binary_op_delete(BinaryOp* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "BinaryOp")]
    class BinaryOp {
        #[cpp(method = "int get_a() const")]
        fn get_a(&self) -> i32;

        #[cpp(method = "int get_b() const")]
        fn get_b(&self) -> i32;

        #[cpp(method = "int get_result() const")]
        fn get_result(&self) -> i32;

        #[cpp(method = "void store(int, int, int)")]
        fn store(&mut self, a: i32, b: i32, result: i32);
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class BinaryOp;

    #[cpp(func = "BinaryOp* binary_op_new()")]
    fn binary_op_new() -> *mut BinaryOp;

    #[cpp(func = "void binary_op_delete(BinaryOp* self)")]
    unsafe fn binary_op_delete(self_: *mut BinaryOp);
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
