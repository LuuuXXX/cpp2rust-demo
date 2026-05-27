hicc::cpp! {
    #include <iostream>

    class MyClass {
        int secret_value;
    public:
        MyClass(int v) : secret_value(v) {}
        ~MyClass() {}
        int getValue() const { return secret_value; }
        void setValue(int v) { secret_value = v; }
    };

    MyClass* myclass_new(int secret_value) {
        return new MyClass(secret_value);
    }

    void myclass_delete(MyClass* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "MyClass")]
    class MyClass {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn set_value(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    class MyClass;

    #[cpp(func = "MyClass* myclass_new(int)")]
    fn myclass_new(secret_value: i32) -> *mut MyClass;

    #[cpp(func = "void myclass_delete(MyClass* self)")]
    unsafe fn myclass_delete(self_: *mut MyClass);
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


