use class_constructor::*;

fn show(s: &hicc_std::string) -> String {
    let cs = unsafe { std::ffi::CStr::from_ptr(s.c_str()) };
    cs.to_str().unwrap().to_string()
}

#[test]
fn default_ctor() {
    let w = Widget::new();
    assert_eq!(w.value(), 0);
    assert_eq!(show(&w.name()), "default");
}

#[test]
fn int_ctor() {
    let w = Widget::from_int(99);
    assert_eq!(w.value(), 99);
    assert_eq!(show(&w.name()), "int");
}

#[test]
fn named_ctor() {
    let n = hicc_std::string::from(c"foo");
    let w = Widget::from_named(n, 42);
    assert_eq!(w.value(), 42);
    assert_eq!(show(&w.name()), "foo");
}
