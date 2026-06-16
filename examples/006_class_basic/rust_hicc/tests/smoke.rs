use class_basic::*;

#[test]
fn counter_basic() {
    let mut c = Counter::new();
    c.inc();
    c.inc();
    c.inc_by(10);
    assert_eq!(c.count(), 12);
}

#[test]
fn counter_named() {
    let n = hicc_std::string::from(c"named");
    let mut c = Counter::with_name(&n);
    c.inc();
    assert_eq!(c.count(), 1);
    let nm = c.name();
    let cs = unsafe { std::ffi::CStr::from_ptr(nm.c_str()) };
    assert_eq!(cs.to_str().unwrap(), "named");
}

#[test]
fn counter_reset() {
    let mut c = Counter::new();
    for _ in 0..5 { c.inc(); }
    c.reset();
    assert_eq!(c.count(), 0);
}
