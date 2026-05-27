hicc::cpp! {
    #include <iostream>
    #include <stack>

    class IntStack {
    public:
        Stack<int> impl;
    public:
        IntStack() = default;
        int size() const { return impl.size(); }
        bool empty() const { return impl.empty(); }
        void push(int value) { impl.push(value); }
        int top() const { return impl.top(); }
        void pop() { impl.pop(); }
    };

    class DoubleStack {
    public:
        Stack<double> impl;
    public:
        DoubleStack() = default;
        int size() const { return impl.size(); }
        bool empty() const { return impl.empty(); }
        void push(double value) { impl.push(value); }
        double top() const { return impl.top(); }
        void pop() { impl.pop(); }
    };

    IntStack* intstack_new() {
        return new IntStack();
    }

    void intstack_delete(IntStack* self) {
        delete self;
    }

    DoubleStack* doublestack_new() {
        return new DoubleStack();
    }

    void doublestack_delete(DoubleStack* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "IntStack")]
    class IntStack {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void push(int value)")]
        fn push(&mut self, value: i32);

        #[cpp(method = "int top() const")]
        fn top(&self) -> i32;

        #[cpp(method = "void pop()")]
        fn pop(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleStack")]
    class DoubleStack {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void push(double value)")]
        fn push(&mut self, value: f64);

        #[cpp(method = "double top() const")]
        fn top(&self) -> f64;

        #[cpp(method = "void pop()")]
        fn pop(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "template_class"]

    class IntStack;
    class DoubleStack;

    #[cpp(func = "IntStack* intstack_new()")]
    fn intstack_new() -> *mut IntStack;

    #[cpp(func = "void intstack_delete(IntStack* self)")]
    unsafe fn intstack_delete(self_: *mut IntStack);

    #[cpp(func = "DoubleStack* doublestack_new()")]
    fn doublestack_new() -> *mut DoubleStack;

    #[cpp(func = "void doublestack_delete(DoubleStack* self)")]
    unsafe fn doublestack_delete(self_: *mut DoubleStack);
}

fn main() {
    println!("=== 025_template_class - 类模板 ===\n");

    // IntStack
    let int_stack = intstack_new();
    println!("IntStack empty: {}", intstack_empty(&int_stack) == 1);

    intstack_push(&int_stack, 10);
    intstack_push(&int_stack, 20);
    intstack_push(&int_stack, 30);

    println!("IntStack size: {}", intstack_size(&int_stack));
    println!("IntStack top: {}", intstack_top(&int_stack));
    intstack_pop(&int_stack);
    println!("After pop, top: {}", intstack_top(&int_stack));

    unsafe { intstack_delete(&int_stack) };

    println!();

    // DoubleStack
    let double_stack = doublestack_new();
    println!("DoubleStack empty: {}", doublestack_empty(&double_stack) == 1);

    doublestack_push(&double_stack, 1.1);
    doublestack_push(&double_stack, 2.2);
    doublestack_push(&double_stack, 3.3);

    println!("DoubleStack size: {}", doublestack_size(&double_stack));
    println!("DoubleStack top: {}", doublestack_top(&double_stack));
    doublestack_pop(&double_stack);
    println!("After pop, top: {}", doublestack_top(&double_stack));

    unsafe { doublestack_delete(&double_stack) };

    println!("\nRust FFI: 类模板 = 为每种类型实例化独立结构");
    println!("Stack<int> -> IntStack");
    println!("Stack<double> -> DoubleStack");
}



