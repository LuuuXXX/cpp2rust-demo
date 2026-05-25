hicc::cpp! {
    #include <iostream>

    class Widget {
        int value;
    public:
        Widget(int v);
        explicit Widget(double v);
        ~Widget();
        int getValue() const;
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

    int widget_getValue(Widget* self) {
        return self->getValue();
    }

    Widget::Widget(int v) : value(v) {}
    Widget::Widget(double v) : value(static_cast<int>(v)) {}
    Widget::~Widget() {}
    int Widget::getValue() const { return value; }
}

hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "int getValue() const")]
        fn getValue(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    class Widget;

    #[cpp(func = "Widget* widget_new(int value)")]
    fn widget_new(value: i32) -> *mut Widget;

    #[cpp(func = "Widget* widget_fromInt(int value)")]
    fn widget_fromInt(value: i32) -> *mut Widget;

    #[cpp(func = "Widget* widget_fromDouble(double value)")]
    fn widget_fromDouble(value: f64) -> *mut Widget;

    #[cpp(func = "void widget_delete(Widget* self)")]
    unsafe fn widget_delete(self_: *mut Widget);

    #[cpp(func = "int widget_getValue(Widget* self)")]
    fn widget_getValue(self_: *mut Widget) -> i32;
}

fn main() {
    println!("=== 021_explicit_ctor - explicit 构造函数 ===\n");
    println!("C++ explicit 关键字防止隐式类型转换\n");

    // Implicit constructor
    let w1 = widget_new(42);
    println!("Created with implicit ctor: value = {}", widget_getValue(&w1));
    unsafe { widget_delete(&w1) };

    println!();

    // Explicit constructor - must be called explicitly
    let w2 = widget_fromInt(100);
    println!("Created with explicit int ctor: value = {}", widget_getValue(&w2));
    unsafe { widget_delete(&w2) };

    let w3 = widget_fromDouble(3.14);
    println!("Created with explicit double ctor: value = {}", widget_getValue(&w3));
    unsafe { widget_delete(&w3) };

    println!("\nRust FFI: explicit 不影响 FFI - 只是禁止隐式转换");
    println!("在 FFI 中，所有构造函数都是显式调用的");
}
