hicc::cpp! {
    #include <iostream>

    #include "explicit_ctor.h"
}

hicc::import_class! {
    #[cpp(class = "Widget", destroy = "widget_delete")]
    pub class Widget {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    class Widget;

    #[cpp(func = "Widget* widget_new(int)")]
    pub fn widget_new(value: i32) -> Widget;

    #[cpp(func = "Widget* widget_fromInt(int)")]
    pub fn widget_from_int(value: i32) -> Widget;

    #[cpp(func = "Widget* widget_fromDouble(double)")]
    pub fn widget_from_double(value: f64) -> Widget;
}
