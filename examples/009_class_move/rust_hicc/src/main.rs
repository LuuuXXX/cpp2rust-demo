hicc::cpp! {
    #include <iostream>
    #include <cstring>

    class UniqueVector {
        int* data;
        int size;
    public:
        UniqueVector() : data(nullptr), size(0) {}
        UniqueVector(int* data, int size) : size(size) {
    this->data = new int[size];
    std::memcpy(this->data, data, size * sizeof(int));
}
        ~UniqueVector() {
    delete[] data;
}
        UniqueVector(UniqueVector&& other) noexcept : data(other.data), size(other.size) {
    other.data = nullptr;
    other.size = 0;
}
        UniqueVector& operator=(UniqueVector&& other) noexcept {
    if (this != &other) {
        delete[] data;
        data = other.data;
        size = other.size;
        other.data = nullptr;
        other.size = 0;
    }
    return *this;
}
        int get(int index) const {
    if (index >= 0 && index < size) {
        return data[index];
    }
    return 0;
}
        void set(int index, int value) {
    if (index >= 0 && index < size) {
        data[index] = value;
    }
}
        int getSize() const {
    return size;
}
        void moveFrom(UniqueVector& src) {
    delete[] data;
    data = src.data;
    size = src.size;
    src.data = nullptr;
    src.size = 0;
}
    };

    UniqueVector* unique_vector_new() {
        return new UniqueVector();
    }

    UniqueVector* unique_vector_newWithData(int* data, int size) {
        return new UniqueVector(data, size);
    }

    void unique_vector_delete(UniqueVector* self) {
        delete self;
    }

    void unique_vector_move(UniqueVector* dest, UniqueVector* src) {
        std::cout << "Moving UniqueVector: " << src->getSize() << " -> " << dest->getSize() << std::endl;
        dest->moveFrom(*src);
    }
}

hicc::import_class! {
    #[cpp(class = "UniqueVector")]
    class UniqueVector {
        #[cpp(method = "int get(int index) const")]
        fn get(&self, index: i32) -> i32;

        #[cpp(method = "void set(int index, int value)")]
        fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int getSize() const")]
        fn get_size(&self) -> i32;

        #[cpp(method = "void moveFrom(UniqueVector & src)")]
        fn move_from(&mut self, src: &mut UniqueVector);
    }
}

hicc::import_lib! {
    #![link_name = "class_move"]

    class UniqueVector;

    #[cpp(func = "UniqueVector* unique_vector_new()")]
    fn unique_vector_new() -> *mut UniqueVector;

    #[cpp(func = "UniqueVector* unique_vector_newWithData(int*, int)")]
    unsafe fn unique_vector_new_with_data(data: *mut i32, size: i32) -> *mut UniqueVector;

    #[cpp(func = "void unique_vector_delete(UniqueVector* self)")]
    unsafe fn unique_vector_delete(self_: *mut UniqueVector);

    #[cpp(func = "void unique_vector_move(UniqueVector* dest, UniqueVector* src)")]
    unsafe fn unique_vector_move(dest: *mut UniqueVector, src: *mut UniqueVector);
}

fn main() {
    unsafe {
        // Create source vector with data
        let mut data = vec![10, 20, 30, 40, 50];
        let mut src_with_data = unique_vector_new_with_data(data.as_mut_ptr(), 5);

        println!("src_with_data size: {}", src_with_data.get_size());
        println!("src_with_data[0]: {}", src_with_data.get(0));

        // Create destination vector
        let mut dest = unique_vector_new();
        println!("dest size before move: {}", dest.get_size());

        // Move: transfer resources from src to dest
        unique_vector_move(&dest, &src_with_data);

        println!("dest size after move: {}", dest.get_size());
        println!("dest[0]: {}", dest.get(0));

        // src should now be empty
        println!("src_with_data size after move: {}", src_with_data.get_size());

        // Cleanup
        unique_vector_delete(&dest);
        unique_vector_delete(&src_with_data);
    }

    println!("\nRust FFI: Move semantics work!");
}



