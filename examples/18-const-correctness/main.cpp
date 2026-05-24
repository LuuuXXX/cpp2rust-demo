// Example 18: Const Correctness
// C++ features: const methods, const references, const pointers, mutable members

#include <cstring>

class Buffer {
private:
    char* data;
    mutable size_t access_count;  // mutable can be modified in const method

public:
    Buffer(const char* str) {
        size_t len = strlen(str);
        data = new char[len + 1];
        strcpy(data, str);
        access_count = 0;
    }

    ~Buffer() {
        delete[] data;
    }

    // Const method - promises not to modify state
    size_t length() const {
        access_count++;  // OK: mutable member
        return strlen(data);
    }

    // Const method
    const char* c_str() const {
        access_count++;
        return data;
    }

    // Non-const method - can modify state
    void clear() {
        data[0] = '\0';
        access_count++;
    }

    // Const method returning const reference
    const char& front() const {
        return data[0];
    }

    // Non-const method returning non-const reference
    char& front() {
        return data[0];
    }

    size_t get_access_count() const {
        return access_count;
    }
};

// Function taking const reference
size_t get_length(const Buffer& buf) {
    return buf.length();  // Can call const methods
}

// Function taking const pointer
size_t get_length_ptr(const Buffer* buf) {
    if (buf) {
        return buf->length();
    }
    return 0;
}

// Function taking const pointer to const
size_t get_length_const_ptr(const Buffer* const buf) {
    if (buf) {
        return buf->length();
    }
    return 0;
}

// Overload based on const
class Overloaded {
public:
    void process() {
        // Non-const version
    }

    void process() const {
        // Const version
    }
};
