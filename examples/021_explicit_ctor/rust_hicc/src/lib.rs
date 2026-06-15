hicc::cpp! {
    #include "explicit_ctor.h"
    std::unique_ptr<Widget> widget_new_int(int v) {
        return std::make_unique<Widget>(v);
    }
    std::unique_ptr<Widget> widget_new_double(double v) {
        return std::make_unique<Widget>(v);
    }
}

hicc::import_class! {
    #[cpp(class = "Widget")]
    pub class Widget {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    class Widget;

    #[cpp(func = "std::unique_ptr<Widget> widget_new_int(int)")]
    pub fn widget_new_with_v_i32(v: i32) -> Widget;

    #[cpp(func = "std::unique_ptr<Widget> widget_new_double(double)")]
    pub fn widget_new_with_v_f64(v: f64) -> Widget;
}
