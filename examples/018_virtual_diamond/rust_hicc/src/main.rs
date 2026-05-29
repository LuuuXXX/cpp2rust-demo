hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    int d_get_a_value(D* self) {
        return self->getAValue();
    }

}

hicc::import_class! {
    #[cpp(class = "D", destroy = "d_delete")]
    class D {
        #[cpp(method = "int getBValue() const")]
        fn get_b_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        fn get_c_value(&self) -> i32;

        #[cpp(method = "int getDValue() const")]
        fn get_d_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    class D;

    #[cpp(func = "D* d_new(int, int, int, int)")]
    fn d_new(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    fn d_get_a_value(self_: *mut D) -> i32;
}

fn main() {
    println!("=== Diamond Inheritance FFI with hicc ===\n");
    println!("Diamond inheritance structure:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ /");
    println!("       D");
    println!();
    println!("Virtual inheritance ensures only ONE A subobject in D\n");

    let mut d = unsafe { d_new(1, 2, 3, 4) };
    use hicc::AbiClass;

    println!("Values:");
    println!("  A value (via B): {}", d_get_a_value(&d.as_mut_ptr()));
    println!("  B value: {}", d.get_b_value());
    println!("  C value: {}", d.get_c_value());
    println!("  D value: {}", d.get_d_value());

    println!();
    d.compute();

    println!("\nRust FFI: Diamond inheritance works correctly with hicc!");
}

