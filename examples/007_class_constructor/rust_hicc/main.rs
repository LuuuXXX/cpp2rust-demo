use class_constructor::*;

fn main() {
    let w1 = Widget::new();
    let w2 = Widget::from_int(99);
    let name = hicc_std::string::from(c"named");
    let w3 = Widget::from_named(name, 7);

    println!("w1: {}/{}", show(&w1.name()), w1.value());
    println!("w2: {}/{}", show(&w2.name()), w2.value());
    println!("w3: {}/{}", show(&w3.name()), w3.value());
}

fn show(s: &hicc_std::string) -> String {
    let cs = unsafe { std::ffi::CStr::from_ptr(s.c_str()) };
    cs.to_str().unwrap().to_string()
}
