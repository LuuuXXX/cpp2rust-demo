use class_const::*;

fn main() {
    let mut calc = Calculator::new();
    calc.add(10);
    calc.add(5);
    calc.subtract(3);
    println!("value={} history={}", calc.value(), calc.history_count());

    calc.clear();
    println!("after clear value={} history={}", calc.value(), calc.history_count());
}
