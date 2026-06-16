use class_basic::*;

fn main() {
    let name = hicc_std::string::from(c"demo");
    let mut c = Counter::with_name(&name);

    c.inc();
    c.inc();
    c.inc_by(10);

    let nm = c.name();
    let cs = unsafe { std::ffi::CStr::from_ptr(nm.c_str()) };
    println!("[rust] name={} count={}", cs.to_str().unwrap(), c.count());

    c.reset();
    println!("[rust] after reset, count={}", c.count());
}
