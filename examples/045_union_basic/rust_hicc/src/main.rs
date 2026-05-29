hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>

    enum VariantType {
        VALUE_TYPE_INT = 0,
        VALUE_TYPE_FLOAT = 1,
        VALUE_TYPE_STRING = 2,
    };

    struct IntFloatUnion {
    public:
        union {
        int int_value;
        float float_value;
    } data;
    };

    class Variant {
        int type_;
        union {
        int int_value_;
        float float_value_;
        char string_buffer_[64];
    } data_;
    public:
        Variant() : type_(VALUE_TYPE_INT) {
    data_.int_value_ = 0;
}
        ~Variant() {}
        int get_type() const {
    return type_;
}
        void set_int(int value) {
    type_ = VALUE_TYPE_INT;
    data_.int_value_ = value;
}
        void set_float(float value) {
    type_ = VALUE_TYPE_FLOAT;
    data_.float_value_ = value;
}
        void set_string(const char* value) {
    type_ = VALUE_TYPE_STRING;
    if (value) {
        strncpy(data_.string_buffer_, value, 63);
        data_.string_buffer_[63] = '\0';
    } else {
        data_.string_buffer_[0] = '\0';
    }
}
        int get_int() const {
    if (type_ == VALUE_TYPE_INT) {
        return data_.int_value_;
    }
    return 0;
}
        float get_float() const {
    if (type_ == VALUE_TYPE_FLOAT) {
        return data_.float_value_;
    }
    return 0.0f;
}
        const char* get_string() const {
    if (type_ == VALUE_TYPE_STRING) {
        return data_.string_buffer_;
    }
    return "";
}
    };

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

    IntFloatUnion* union_new() {
        return new IntFloatUnion();
    }

    void union_delete(IntFloatUnion* u) {
        delete u;
    }

    int union_get_int(IntFloatUnion* u) {
        if (u) return u->data.int_value;
        return 0;
    }

    float union_get_float(IntFloatUnion* u) {
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
    #[cpp(class = "IntFloatUnion")]
    class IntFloatUnion {}
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

    class IntFloatUnion;
    class Variant;

    #[cpp(func = "Variant* variant_new_int(int)")]
    fn variant_new_int(value: i32) -> *mut Variant;

    #[cpp(func = "Variant* variant_new_float(float)")]
    fn variant_new_float(value: f32) -> *mut Variant;

    #[cpp(func = "Variant* variant_new_string(const char*)")]
    unsafe fn variant_new_string(value: *const i8) -> *mut Variant;

    #[cpp(func = "void variant_delete(Variant* self)")]
    unsafe fn variant_delete(self_: *mut Variant);

    #[cpp(func = "IntFloatUnion* union_new()")]
    fn union_new() -> *mut IntFloatUnion;

    #[cpp(func = "void union_delete(IntFloatUnion* u)")]
    unsafe fn union_delete(u: *mut IntFloatUnion);

    #[cpp(func = "int union_get_int(IntFloatUnion* u)")]
    fn union_get_int(u: *mut IntFloatUnion) -> i32;

    #[cpp(func = "float union_get_float(IntFloatUnion* u)")]
    fn union_get_float(u: *mut IntFloatUnion) -> f32;

    #[cpp(func = "void union_set_int(IntFloatUnion* u, int)")]
    unsafe fn union_set_int(u: *mut IntFloatUnion, value: i32);

    #[cpp(func = "void union_set_float(IntFloatUnion* u, float)")]
    unsafe fn union_set_float(u: *mut IntFloatUnion, value: f32);
}

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
    println!("Type: {}, Value: {}", variant_type_name(v_int.get_type()), v_int.get_int());
    unsafe { variant_delete(&v_int); }

    let v_float = variant_new_float(3.14);
    println!("Type: {}, Value: {}", variant_type_name(v_float.get_type()), v_float.get_float());
    unsafe { variant_delete(&v_float); }

    let v_string = unsafe { variant_new_string("Hello, Union!\0".as_ptr() as *const i8) };
    let s = unsafe { std::ffi::CStr::from_ptr(v_string.get_string()) };
    println!("Type: {}, Value: {}", variant_type_name(v_string.get_type()), s.to_str().unwrap());
    unsafe { variant_delete(&v_string); }

    // Memory overlay demo
    println!("\n--- Memory Overlay Demo ---");
    println!("sizeof(int) = {}, sizeof(float) = {}", std::mem::size_of::<i32>(), std::mem::size_of::<f32>());

    let mut union_int = union_new();

    // Set int value
    unsafe { union_set_int(&mut union_int, 0x41414141); }  // 'AAAA' in ASCII
    println!("Set as int: {} (0x{:08x})", union_get_int(&union_int), union_get_int(&union_int) as u32);

    // Read same memory as float
    let float_bits = union_get_float(&union_int);
    println!("Read as float: {} (bits: 0x{:08x})", float_bits, float_bits.to_bits());

    unsafe { union_delete(&union_int); };

    println!("\n--- Summary ---");
    println!("1. union all members share the same memory");
    println!("2. Modifying one member affects other members");
    println!("3. union size equals the largest member size");
    println!("4. Often used to save memory or for type punning");
    println!("5. FFI passes union via variant wrapper");
}

