hicc::cpp! {
    #include <utility>
    #include <cstdint>

    // noexcept functions - guaranteed not to throw
    inline int noexcept_add(int a, int b) noexcept {
        return a + b;
    }

    inline int noexcept_multiply(int a, int b) noexcept {
        return a * b;
    }

    inline int conditional_abs(int value) noexcept {
        return value >= 0 ? value : -value;
    }

    // Move-only type with noexcept move operations
    class NoexceptMover {
        int value_;
    public:
        NoexceptMover(int value) : value_(value) {}
        ~NoexceptMover() {}
        NoexceptMover(NoexceptMover&& other) noexcept : value_(other.value_) {
            other.value_ = 0;
        }
        NoexceptMover& operator=(NoexceptMover&& other) noexcept {
            if (this != &other) {
                value_ = other.value_;
                other.value_ = 0;
            }
            return *this;
        }
        int get_value() const { return value_; }
        NoexceptMover(const NoexceptMover&) = delete;
        NoexceptMover& operator=(const NoexceptMover&) = delete;
    };

    // Factory functions
    NoexceptMover* noexcept_mover_new(int value) {
        return new NoexceptMover(value);
    }

    void noexcept_mover_delete(NoexceptMover* self) {
        delete self;
    }

    // noexcept move - transfers ownership
    NoexceptMover* noexcept_mover_move(NoexceptMover* other) noexcept {
        if (other) {
            return new NoexceptMover(std::move(*other));
        }
        return nullptr;
    }
}

hicc::import_class! {
    #[cpp(class = "NoexceptMover")]
    class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    class NoexceptMover;

    #[cpp(func = "int noexcept_add(int a, int b) noexcept")]
    fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_multiply(int a, int b) noexcept")]
    fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int conditional_abs(int value) noexcept")]
    fn conditional_abs(value: i32) -> i32;

    #[cpp(func = "NoexceptMover* noexcept_mover_new(int value)")]
    fn noexcept_mover_new(value: i32) -> *mut NoexceptMover;

    #[cpp(func = "void noexcept_mover_delete(NoexceptMover* self)")]
    unsafe fn noexcept_mover_delete(self_: *mut NoexceptMover);

    #[cpp(func = "NoexceptMover* noexcept_mover_move(NoexceptMover* other) noexcept")]
    fn noexcept_mover_move(other: *mut NoexceptMover) -> *mut NoexceptMover;
}

fn main() {
    println!("=== 047_noexcept_basic - noexcept ===\n");

    // noexcept functions
    println!("--- noexcept Functions ---");
    println!("noexcept_add(10, 20) = {}", noexcept_add(10, 20));
    println!("noexcept_multiply(6, 7) = {}", noexcept_multiply(6, 7));
    println!("conditional_abs(-42) = {}", conditional_abs(-42));

    // noexcept move semantics
    println!("\n--- noexcept Move Semantics ---");
    let mover1 = noexcept_mover_new(100);
    println!("Original mover created, value = {}", mover1.get_value());

    let mover2 = noexcept_mover_move(&mover1);
    println!("Mover moved (noexcept), new value = {}", mover2.get_value());

    unsafe { noexcept_mover_delete(&mover2); }

    println!("\n--- Summary ---");
    println!("1. noexcept declares function won't throw");
    println!("2. Move constructors and move assignment operators often use noexcept");
    println!("3. noexcept move operations have better performance in STL containers");
    println!("4. noexcept functions cannot call potentially throwing functions");
    println!("5. In FFI, noexcept is part of function signature");
}
