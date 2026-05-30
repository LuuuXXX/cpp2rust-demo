hicc::cpp! {
    #include <cstdint>

    #include "summary.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    class Counter {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(method = "void decrement()")]
        fn decrement(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "summary"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    fn counter_new() -> Counter;

    #[cpp(func = "int safe_add(int, int)")]
    fn safe_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int get_max_size()")]
    fn get_max_size() -> i32;
}

fn main() {
    println!("=== 048_summary - FFI Patterns Summary ===\n");

    // 1. Opaque Pointer Pattern
    println!("--- 1. Opaque Pointer Pattern ---");
    let mut counter = counter_new();
    println!("Initial value: {}", counter.get());
    counter.increment();
    counter.increment();
    println!("After 2 increments: {}", counter.get());
    counter.decrement();
    println!("After 1 decrement: {}", counter.get());

    // 2. Class Import Pattern
    println!("\n--- 2. Class Import Pattern ---");
    println!("See 006_class_basic for full class FFI pattern");

    // 3. Namespace Pattern
    println!("\n--- 3. Namespace Pattern ---");
    println!("Namespaces are flattened in FFI");
    println!("get_max_size() = {}", get_max_size());

    // 4. Enum Class Pattern
    println!("\n--- 4. Enum Class Pattern ---");
    println!("See 044_enum_class for enum class FFI pattern");
    println!("Enum values passed as integers across FFI");

    // 5. Union Pattern
    println!("\n--- 5. Union Pattern ---");
    println!("See 045_union_basic for union FFI pattern");
    println!("Unions share memory between members");

    // 6. Constexpr Pattern
    println!("\n--- 6. Constexpr Pattern ---");
    println!("constexpr values computed at compile time");
    println!("get_max_size() = {} (runtime call, but value is constexpr)", get_max_size());

    // 7. Noexcept Pattern
    println!("\n--- 7. Noexcept Pattern ---");
    println!("safe_add(10, 20) = {}", safe_add(10, 20));
    println!("noexcept guarantees no exceptions");

    // 8. Exception Handling Pattern
    println!("\n--- 8. Exception Handling Pattern ---");
    println!("See 042_exception_basic for exception FFI pattern");
    println!("Exceptions cannot cross FFI boundary");

    // Summary Table
    println!("\n=== Pattern Summary Table ===");
    println!("| Example | Pattern |");
    println!("|---------|---------|");
    println!("| 001-005 | extern \"C\" functions |");
    println!("| 006-012 | Class with opaque pointer |");
    println!("| 013-018 | Inheritance and virtual |");
    println!("| 019-023 | Operators and special members |");
    println!("| 024-028 | Templates |");
    println!("| 029-033 | Smart pointers and RAII |");
    println!("| 034-038 | STL containers |");
    println!("| 039-041 | Functions and lambdas |");
    println!("| 042 | Exception handling |");
    println!("| 043 | Nested namespaces |");
    println!("| 044 | enum class |");
    println!("| 045 | Union |");
    println!("| 046 | constexpr |");
    println!("| 047 | noexcept |");

    println!("\n=== Key FFI Principles ===");
    println!("1. C++ exceptions cannot propagate across FFI boundary");
    println!("2. Use opaque pointers for C++ classes");
    println!("3. extern \"C\" flattens C++ name mangling");
    println!("4. Enums passed as underlying integer type");
    println!("5. Unions share memory between members");
    println!("6. constexpr computed at compile time");
    println!("7. noexcept is part of function signature in FFI");
}

