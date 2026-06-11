use lambda_basic::*;

fn main() {
    println!("=== 039_lambda_basic - Lambda 表达式 ===\n");

    println!("--- Direct function calls ---");
    println!("add_impl(3, 4) = {}", add_impl(3, 4));
    println!("multiply_impl(3, 4) = {}", multiply_impl(3, 4));
    println!("max_impl(3, 4) = {}", max_impl(3, 4));

    println!("\n--- LambdaWrapper Demo ---");
    let mut add_wrapper = make_add_lambda();
    println!("add invoke(5, 6) = {}", add_wrapper.invoke(5, 6));

    let mut mul_wrapper = make_multiply_lambda();
    println!("multiply invoke(5, 6) = {}", mul_wrapper.invoke(5, 6));

    println!("\n--- StateLambda Demo ---");
    let mut state = state_lambda_new(10);
    println!("initial value = {}", state.get_value());
    println!("add(5) = {}", state.add(5));
    println!("add(3) = {}", state.add(3));

    println!("\n--- Comparator Demo ---");
    let mut cmp = comparator_new_add();
    println!("compare(2, 3) = {}", cmp.compare(2, 3));

    println!("\nRust FFI: Lambda 表达式映射");
    println!("1. 函数指针可以通过 FFI 传递");
    println!("2. 捕获状态的 lambda 需要包装在类中");
    println!("3. 此示例展示基本的类封装模式");
}
