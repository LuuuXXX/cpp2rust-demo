hicc::cpp! {
    #include <iostream>
    #include <stack>

    template<typename T>
    class Stack {
    public:
        std::stack<T> data;
        Stack() = default;
        int size() const { return static_cast<int>(data.size()); }
        bool empty() const { return data.empty(); }
        void push(T value) { data.push(value); }
        T top() const { return data.top(); }
        void pop() { data.pop(); }
    };

    class IntStack {
    public:
        Stack<int> impl;
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
        DoubleStack() = default;
        int size() const { return impl.size(); }
        bool empty() const { return impl.empty(); }
        void push(double value) { impl.push(value); }
        double top() const { return impl.top(); }
        void pop() { impl.pop(); }
    };

    IntStack* intstack_new(void) {
        return new IntStack();
    }

    void intstack_delete(IntStack* self) {
        delete self;
    }

    int intstack_size(IntStack* self) {
        return self->impl.size();
    }

    int intstack_empty(IntStack* self) {
        return self->impl.empty() ? 1 : 0;
    }

    void intstack_push(IntStack* self, int value) {
        self->impl.push(value);
    }

    int intstack_top(IntStack* self) {
        return self->impl.top();
    }

    void intstack_pop(IntStack* self) {
        self->impl.pop();
    }

    DoubleStack* doublestack_new(void) {
        return new DoubleStack();
    }

    void doublestack_delete(DoubleStack* self) {
        delete self;
    }

    int doublestack_size(DoubleStack* self) {
        return self->impl.size();
    }

    int doublestack_empty(DoubleStack* self) {
        return self->impl.empty() ? 1 : 0;
    }

    void doublestack_push(DoubleStack* self, double value) {
        self->impl.push(value);
    }

    double doublestack_top(DoubleStack* self) {
        return self->impl.top();
    }

    void doublestack_pop(DoubleStack* self) {
        self->impl.pop();
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

    #[cpp(func = "int intstack_size(IntStack* self)")]
    fn intstack_size(self_: *mut IntStack) -> i32;

    #[cpp(func = "int intstack_empty(IntStack* self)")]
    fn intstack_empty(self_: *mut IntStack) -> i32;

    #[cpp(func = "void intstack_push(IntStack* self, int value)")]
    fn intstack_push(self_: *mut IntStack, value: i32);

    #[cpp(func = "int intstack_top(IntStack* self)")]
    fn intstack_top(self_: *mut IntStack) -> i32;

    #[cpp(func = "void intstack_pop(IntStack* self)")]
    fn intstack_pop(self_: *mut IntStack);

    #[cpp(func = "DoubleStack* doublestack_new()")]
    fn doublestack_new() -> *mut DoubleStack;

    #[cpp(func = "void doublestack_delete(DoubleStack* self)")]
    unsafe fn doublestack_delete(self_: *mut DoubleStack);

    #[cpp(func = "int doublestack_size(DoubleStack* self)")]
    fn doublestack_size(self_: *mut DoubleStack) -> i32;

    #[cpp(func = "int doublestack_empty(DoubleStack* self)")]
    fn doublestack_empty(self_: *mut DoubleStack) -> i32;

    #[cpp(func = "void doublestack_push(DoubleStack* self, double value)")]
    fn doublestack_push(self_: *mut DoubleStack, value: f64);

    #[cpp(func = "double doublestack_top(DoubleStack* self)")]
    fn doublestack_top(self_: *mut DoubleStack) -> f64;

    #[cpp(func = "void doublestack_pop(DoubleStack* self)")]
    fn doublestack_pop(self_: *mut DoubleStack);
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
