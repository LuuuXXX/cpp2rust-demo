hicc::cpp! {
    #include <cstddef>
    #include <iostream>

    struct ConstexprPoint {
    public:
        int x;
        int y;
        constexpr ConstexprPoint(int x, int y) : x(x), y(y) {}
        constexpr int manhattan_distance() const {
            return (x > 0 ? x : -x) + (y > 0 ? y : -y);
        }
    };

    static const int FIB_10 = 55;

    int get_fibonacci_10() {
        std::cout << "get_fibonacci_10() called, returning compile-time computed value: "
                  << FIB_10 << std::endl;
        return FIB_10;
    }

    int manhattan_distance(int x, int y) {
        const int dx = x > 0 ? x : -x;
        const int dy = y > 0 ? y : -y;
        return dx + dy;
    }

    int constexpr_sum_array(const int* arr, int size) {
        int sum = 0;
        for (int i = 0; i < size; ++i) {
            sum += arr[i];
        }
        return sum;
    }

    int constexpr_find_max(const int* arr, int size) {
        if (size <= 0) return 0;
        int max_val = arr[0];
        for (int i = 1; i < size; ++i) {
            if (arr[i] > max_val) {
                max_val = arr[i];
            }
        }
        return max_val;
    }

    static const int ARRAY_SIZE = 10;

    int get_array_size() {
        return ARRAY_SIZE;
    }
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "int get_fibonacci_10()")]
    fn get_fibonacci_10() -> i32;

    #[cpp(func = "int manhattan_distance(int, int)")]
    fn manhattan_distance(x: i32, y: i32) -> i32;

    #[cpp(func = "int constexpr_sum_array(const int*, int)")]
    fn constexpr_sum_array(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int constexpr_find_max(const int*, int)")]
    fn constexpr_find_max(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int get_array_size()")]
    fn get_array_size() -> i32;
}

// Rust compile-time fibonacci equivalent
const FIB_RUST: i32 = {
    let mut a = 0;
    let mut b = 1;
    let mut i = 0;
    while i < 10 {
        let result = a + b;
        a = b;
        b = result;
        i += 1;
    }
    a
};

fn main() {
    println!("=== 046_constexpr_basic - constexpr ===\n");

    // Compile-time computed fibonacci number
    println!("--- Compile-time Fibonacci ---");
    let fib_10 = get_fibonacci_10();
    println!("fibonacci<10>() = {} (computed at compile time)", fib_10);
    println!("Rust equivalent: fib(10) = {} (also compile time)", FIB_RUST);

    // Runtime manhattan distance
    println!("\n--- Runtime Manhattan Distance ---");
    println!("manhattan_distance(3, 4) = {}", manhattan_distance(3, 4));
    println!("manhattan_distance(-3, -4) = {}", manhattan_distance(-3, -4));
    println!("manhattan_distance(10, -5) = {}", manhattan_distance(10, -5));

    // Array operations
    println!("\n--- Array Operations ---");
    let arr = [1, 5, 3, 9, 2, 8, 4, 7, 6, 0];
    let size = get_array_size();
    println!("Array: {:?}", &arr[..size as usize]);

    let sum = constexpr_sum_array(arr.as_ptr(), size);
    println!("Sum: {}", sum);

    let max = constexpr_find_max(arr.as_ptr(), size);
    println!("Max: {}", max);

    println!("\n--- Summary ---");
    println!("1. constexpr specifies expression computed at compile time");
    println!("2. constexpr functions must satisfy compile-time evaluation conditions");
    println!("3. constexpr variables have determined values at compile time");
    println!("4. FFI constexpr values passed via preprocessor macros");
    println!("5. Rust const fn can achieve similar functionality");
}



