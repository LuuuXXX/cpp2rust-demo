use exception_basic::*;

fn main() {
    println!("=== 042_exception_basic - 异常处理（hicc 直出）===\n");

    let mut calc = Calculator::new();
    println!("10 / 2 = {} error={}", calc.divide(10, 2), calc.last_error());
    println!(
        "1 / 0 = {} error={} has_error={}",
        calc.divide(1, 0),
        calc.last_error(),
        calc.has_error()
    );
    calc.clear_error();
    println!("after clear has_error={}", calc.has_error());

    for s in ["123", "abc", "99999999999999999999"] {
        let cs = std::ffi::CString::new(s).expect("CString::new failed");
        let value = calc.parse_int(cs.as_ptr());
        println!("parse_int({}) = {} error={}", s, value, calc.last_error());
    }

    println!("\nRust FFI: hicc 直接绑定类，C++ 异常在方法边界内部捕获并转为错误码");
}
