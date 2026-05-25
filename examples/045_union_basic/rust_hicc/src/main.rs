hicc::cpp! {
    #include <cstring>
    #include <cstdint>

    // Type constants for Variant
    const int VALUE_TYPE_INT = 0;
    const int VALUE_TYPE_FLOAT = 1;
    const int VALUE_TYPE_STRING = 2;

    // Variant class with anonymous union
    class Variant {
        int type_;
        union {
            int int_value_;
            float float_value_;
            char string_buffer_[64];
        } data_;
    public:
        Variant();
        ~Variant();
        int get_type() const { return type_; }
        void set_int(int value);
        void set_float(float value);
        void set_string(const char* value);
        int get_int() const;
        float get_float() const;
        const char* get_string() const;
    };

    // Anonymous union struct for direct memory overlay demo
    struct IntFloatUnion {
        union {
            int int_value;
            float float_value;
        } data;
    };

    // Factory functions for Variant
    Variant* variant_new_int(int value) {
        auto* v = new Variant();
        v->set_int(value);
        return v;
    }

    Variant* variant_new_float(float value) {
        auto* v = new Variant();
        v->set_float(value);
        return v;
    }

    Variant* variant_new_string(const char* value) {
        auto* v = new Variant();
        v->set_string(value);
        return v;
    }

    void variant_delete(Variant* self) {
        delete self;
    }

    int variant_get_type(const Variant* self) {
        if (self) return self->get_type();
        return VALUE_TYPE_INT;
    }

    int variant_get_int(const Variant* self) {
        if (self) return self->get_int();
        return 0;
    }

    float variant_get_float(const Variant* self) {
        if (self) return self->get_float();
        return 0.0f;
    }

    const char* variant_get_string(const Variant* self) {
        if (self) return self->get_string();
        return "";
    }

    void variant_set_int(Variant* self, int value) {
        if (self) self->set_int(value);
    }

    void variant_set_float(Variant* self, float value) {
        if (self) self->set_float(value);
    }

    void variant_set_string(Variant* self, const char* value) {
        if (self) self->set_string(value);
    }

    // IntFloatUnion accessors
    int union_get_int(const IntFloatUnion* u) {
        if (u) return u->data.int_value;
        return 0;
    }

    float union_get_float(const IntFloatUnion* u) {
        if (u) return u->data.float_value;
        return 0.0f;
    }

    void union_set_int(IntFloatUnion* u, int value) {
        if (u) u->data.int_value = value;
    }

    void union_set_float(IntFloatUnion* u, float value) {
        if (u) u->data.float_value = value;
    }
}

hicc::import_class! {
    #[cpp(class = "Variant")]
    class Variant {
        #[cpp(method = "int get_type() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "void set_int(int value)")]
        fn set_int(&mut self, value: i32);

        #[cpp(method = "void set_float(float value)")]
        fn set_float(&mut self, value: f32);

        #[cpp(method = "void set_string(const char* value)")]
        fn set_string(&mut self, value: *const i8);

        #[cpp(method = "int get_int() const")]
        fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        fn get_string(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    class Variant;
    struct IntFloatUnion;

    #[cpp(func = "Variant* variant_new_int(int value)")]
    fn variant_new_int(value: i32) -> *mut Variant;

    #[cpp(func = "Variant* variant_new_float(float value)")]
    fn variant_new_float(value: f32) -> *mut Variant;

    #[cpp(func = "Variant* variant_new_string(const char* value)")]
    fn variant_new_string(value: *const i8) -> *mut Variant;

    #[cpp(func = "void variant_delete(Variant* self)")]
    unsafe fn variant_delete(self_: *mut Variant);

    #[cpp(func = "int variant_get_type(const Variant* self)")]
    fn variant_get_type(self_: *mut Variant) -> i32;

    #[cpp(func = "int variant_get_int(const Variant* self)")]
    fn variant_get_int(self_: *mut Variant) -> i32;

    #[cpp(func = "float variant_get_float(const Variant* self)")]
    fn variant_get_float(self_: *mut Variant) -> f32;

    #[cpp(func = "const char* variant_get_string(const Variant* self)")]
    fn variant_get_string(self_: *mut Variant) -> *const i8;

    #[cpp(func = "void variant_set_int(Variant* self, int value)")]
    fn variant_set_int(self_: *mut Variant, value: i32);

    #[cpp(func = "void variant_set_float(Variant* self, float value)")]
    fn variant_set_float(self_: *mut Variant, value: f32);

    #[cpp(func = "void variant_set_string(Variant* self, const char* value)")]
    fn variant_set_string(self_: *mut Variant, value: *const i8);

    #[cpp(func = "int union_get_int(const IntFloatUnion* u)")]
    fn union_get_int(u: *const IntFloatUnion) -> i32;

    #[cpp(func = "float union_get_float(const IntFloatUnion* u)")]
    fn union_get_float(u: *const IntFloatUnion) -> f32;

    #[cpp(func = "void union_set_int(IntFloatUnion* u, int value)")]
    fn union_set_int(u: *mut IntFloatUnion, value: i32);

    #[cpp(func = "void union_set_float(IntFloatUnion* u, float value)")]
    fn union_set_float(u: *mut IntFloatUnion, value: f32);
}

// Type constants
pub const VALUE_TYPE_INT: i32 = 0;
pub const VALUE_TYPE_FLOAT: i32 = 1;
pub const VALUE_TYPE_STRING: i32 = 2;

fn variant_type_name(t: i32) -> &'static str {
    match t {
        0 => "INT",
        1 => "FLOAT",
        2 => "STRING",
        _ => "UNKNOWN",
    }
}

fn main() {
    println!("=== 045_union_basic - Unions ===\n");

    // Variant example
    println!("--- Variant Demo ---");

    let v_int = variant_new_int(42);
    println!("Type: {}, Value: {}", variant_type_name(variant_get_type(&v_int)), variant_get_int(&v_int));
    unsafe { variant_delete(&v_int); }

    let v_float = variant_new_float(3.14);
    println!("Type: {}, Value: {}", variant_type_name(variant_get_type(&v_float)), variant_get_float(&v_float));
    unsafe { variant_delete(&v_float); }

    let v_string = variant_new_string("Hello, Union!\0".as_ptr() as *const i8);
    let s = unsafe { std::ffi::CStr::from_ptr(variant_get_string(&v_string)) };
    println!("Type: {}, Value: {}", variant_type_name(variant_get_type(&v_string)), s.to_str().unwrap());
    unsafe { variant_delete(&v_string); }

    // Memory overlay demo
    println!("\n--- Memory Overlay Demo ---");
    println!("sizeof(int) = {}, sizeof(float) = {}", std::mem::size_of::<i32>(), std::mem::size_of::<f32>());

    let layout = std::alloc::Layout::from_size_align(8, 4).unwrap();
    let union_int = unsafe { std::alloc::alloc(layout) as *mut IntFloatUnion };

    // Set int value
    unsafe { union_set_int(union_int, 0x41414141) };  // 'AAAA' in ASCII
    println!("Set as int: {} (0x{:08x})", unsafe { union_get_int(union_int) }, unsafe { union_get_int(union_int) as u32 });

    // Read same memory as float
    let float_bits = unsafe { union_get_float(union_int) };
    println!("Read as float: {} (bits: 0x{:08x})", float_bits, float_bits.to_bits());

    unsafe { std::alloc::dealloc(union_int as *mut u8, layout) };

    println!("\n--- Summary ---");
    println!("1. union all members share the same memory");
    println!("2. Modifying one member affects other members");
    println!("3. union size equals the largest member size");
    println!("4. Often used to save memory or for type punning");
    println!("5. FFI passes union via variant wrapper");
}
