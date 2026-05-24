// Example 12: Move Semantics
// C++ features: move constructor, move assignment, rvalue references, std::move, move semantics in classes

#include <cstring>
#include <utility>

class Buffer {
private:
    char* data;
    size_t size;

public:
    // Default constructor
    Buffer() : data(nullptr), size(0) {}

    // Constructor with size
    Buffer(size_t sz) : size(sz) {
        data = new char[size];
        memset(data, 0, size);
    }

    // Copy constructor
    Buffer(const Buffer& other) : size(other.size) {
        data = new char[size];
        memcpy(data, other.data, size);
    }

    // Move constructor
    Buffer(Buffer&& other) noexcept : data(other.data), size(other.size) {
        other.data = nullptr;
        other.size = 0;
    }

    // Copy assignment
    Buffer& operator=(const Buffer& other) {
        if (this != &other) {
            delete[] data;
            size = other.size;
            data = new char[size];
            memcpy(data, other.data, size);
        }
        return *this;
    }

    // Move assignment
    Buffer& operator=(Buffer&& other) noexcept {
        if (this != &other) {
            delete[] data;
            data = other.data;
            size = other.size;
            other.data = nullptr;
            other.size = 0;
        }
        return *this;
    }

    ~Buffer() {
        delete[] data;
    }

    size_t get_size() const { return size; }
    const char* get_data() const { return data; }
};

// Function that takes rvalue reference
Buffer create_buffer(size_t size) {
    return Buffer(size);
}

// Function that accepts by value (may move)
void process_buffer(Buffer b) {
    // b is moved in here
}

// Function with rvalue reference parameter
void process_buffer_rvalue(Buffer&& b) {
    // b is an rvalue reference
}
