use enum_class::*;

fn main() {
    println!("=== 044_enum_class - enum class（hicc 直出）===\n");

    let mut result = OperationResult::new();
    result.set_error(3);
    result.set_state(1);
    result.set_flags(7);

    println!(
        "error={} state={} flags={}",
        result.get_error(),
        result.get_state(),
        result.get_flags()
    );
    println!(
        "combine_flags(1,2)={} has_execute={} has_execute_in_read={}",
        combine_flags(1, 2),
        has_flag(result.get_flags(), 4),
        has_flag(1, 4)
    );

    println!("\nRust FFI: hicc 直接绑定持有 enum class 的类，析构由 Rust Drop 自动完成");
}
