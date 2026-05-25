hicc::cpp! {
    #include <iostream>

    class MyClass {
        int secret_value;
        friend int friend_function_getSum(const MyClass* a, const MyClass* b);
        friend int friend_function_getProduct(const MyClass* a, const MyClass* b);
        friend int friend_function_compare(const MyClass* a, const MyClass* b);
    public:
        MyClass(int v);
        ~MyClass();
        int getValue() const;
        void setValue(int v);
    };

    MyClass* myclass_new(int secret_value) {
        return new MyClass(secret_value);
    }

    void myclass_delete(MyClass* self) {
        delete self;
    }

    int myclass_getValue(MyClass* self) {
        return self->getValue();
    }

    void myclass_setValue(MyClass* self, int value) {
        self->setValue(value);
    }

    int friend_function_getSum(const MyClass* a, const MyClass* b) {
        int sum = a->secret_value + b->secret_value;
        std::cout << "Friend function getSum: " << a->secret_value
                  << " + " << b->secret_value << " = " << sum << std::endl;
        return sum;
    }

    int friend_function_getProduct(const MyClass* a, const MyClass* b) {
        int product = a->secret_value * b->secret_value;
        std::cout << "Friend function getProduct: " << a->secret_value
                  << " * " << b->secret_value << " = " << product << std::endl;
        return product;
    }

    int friend_function_compare(const MyClass* a, const MyClass* b) {
        if (a->secret_value < b->secret_value) {
            std::cout << "Friend function compare: a < b" << std::endl;
            return -1;
        } else if (a->secret_value > b->secret_value) {
            std::cout << "Friend function compare: a > b" << std::endl;
            return 1;
        } else {
            std::cout << "Friend function compare: a == b" << std::endl;
            return 0;
        }
    }

    MyClass::MyClass(int v) : secret_value(v) {}
    MyClass::~MyClass() {}
    int MyClass::getValue() const { return secret_value; }
    void MyClass::setValue(int v) { secret_value = v; }
}

hicc::import_class! {
    #[cpp(class = "MyClass")]
    class MyClass {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn setValue(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    class MyClass;

    #[cpp(func = "MyClass* myclass_new(int secret_value)")]
    fn myclass_new(secret_value: i32) -> *mut MyClass;

    #[cpp(func = "void myclass_delete(MyClass* self)")]
    unsafe fn myclass_delete(self_: *mut MyClass);

    #[cpp(func = "int myclass_getValue(MyClass* self)")]
    fn myclass_getValue(self_: *mut MyClass) -> i32;

    #[cpp(func = "void myclass_setValue(MyClass* self, int value)")]
    fn myclass_setValue(self_: *mut MyClass, value: i32);

    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    fn friend_function_getSum(a: *mut MyClass, b: *mut MyClass) -> i32;

    #[cpp(func = "int friend_function_getProduct(const MyClass* a, const MyClass* b)")]
    fn friend_function_getProduct(a: *mut MyClass, b: *mut MyClass) -> i32;

    #[cpp(func = "int friend_function_compare(const MyClass* a, const MyClass* b)")]
    fn friend_function_compare(a: *mut MyClass, b: *mut MyClass) -> i32;
}

fn main() {
    println!("=== Friend Function FFI ===\n");
    println!("Friend functions in C++ can access private members of a class\n");

    let a = myclass_new(10);
    let b = myclass_new(20);

    println!("Created MyClass objects:");
    println!("  a.value = {}", myclass_getValue(&a));
    println!("  b.value = {}", myclass_getValue(&b));
    println!();

    // Friend functions: can access private members
    println!("Friend function operations:");
    let sum = friend_function_getSum(&a, &b);
    println!("  Sum: {}", sum);

    let product = friend_function_getProduct(&a, &b);
    println!("  Product: {}", product);

    let cmp = friend_function_compare(&a, &b);
    println!("  Compare: {}", cmp);

    println!();
    println!("Rust FFI: Friend functions are just regular functions");
    println!("In C FFI, we can access struct members directly");
    println!("The 'friend' relationship is a C++ access control concept");

    unsafe {
        myclass_delete(&a);
        myclass_delete(&b);
    }
}
