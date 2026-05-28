hicc::cpp! {
    #include <iostream>

    class Widget {
        int value;
    public:
        Widget(int v) : value(v) {}
        Widget(double v) : value(static_cast<int>(v)) {}
        ~Widget() {}
        int getValue() const { return value; }
    };

    Widget* widget_new(int value) {
        return new Widget(value);
    }

    Widget* widget_fromInt(int value) {
        return new Widget(value);
    }

    Widget* widget_fromDouble(double value) {
        return new Widget(value);
    }

    void widget_delete(Widget* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    class Widget;

    #[cpp(func = "Widget* widget_new(int)")]
    fn widget_new(value: i32) -> *mut Widget;

    #[cpp(func = "Widget* widget_fromInt(int)")]
    fn widget_from_int(value: i32) -> *mut Widget;

    #[cpp(func = "Widget* widget_fromDouble(double)")]
    fn widget_from_double(value: f64) -> *mut Widget;

    #[cpp(func = "void widget_delete(Widget* self)")]
    unsafe fn widget_delete(self_: *mut Widget);
}

fn main() {
    println!("=== 021_explicit_ctor - explicit 构造函数 ===\n");
    println!("C++ explicit 关键字防止隐式类型转换\n");

    // Implicit constructor
    let w1 = widget_new(42);
    println!("Created with implicit ctor: value = {}", w1.get_value());
    unsafe { widget_delete(&w1) };

    println!();

    // Explicit constructor - must be called explicitly
    let w2 = widget_from_int(100);
    println!("Created with explicit int ctor: value = {}", w2.get_value());
    unsafe { widget_delete(&w2) };

    let w3 = widget_from_double(3.14);
    println!("Created with explicit double ctor: value = {}", w3.get_value());
    unsafe { widget_delete(&w3) };

    println!("\nRust FFI: explicit 不影响 FFI - 只是禁止隐式转换");
    println!("在 FFI 中，所有构造函数都是显式调用的");
}

