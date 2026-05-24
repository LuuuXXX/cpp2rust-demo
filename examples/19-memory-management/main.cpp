// Example 19: Memory Management
// C++ features: new/delete, placement new, RAII, custom allocators

#include <cstdint>
#include <cstring>
#include <new>
#include <cstdlib>

class TrackedObject {
private:
    int id;
    static int next_id;

public:
    TrackedObject() : id(next_id++) {
    }

    ~TrackedObject() {
    }

    int get_id() const { return id; }

    static int get_created_count() { return next_id; }
};

int TrackedObject::next_id = 1;

// Operator new overload
void* operator new(size_t size) {
    return malloc(size);
}

void operator delete(void* ptr) noexcept {
    free(ptr);
}

// Placement new
class PlacementBuffer {
private:
    char data[1024];
    size_t used;

public:
    PlacementBuffer() : used(0) {}

    void* allocate(size_t size) {
        if (used + size > sizeof(data)) {
            return nullptr;
        }
        void* ptr = data + used;
        used += size;
        return ptr;
    }

    size_t get_used() const { return used; }
};

// Aligned allocation
void* aligned_alloc_example(size_t alignment, size_t size) {
    void* ptr = nullptr;
    if (posix_memalign(&ptr, alignment, size) == 0) {
        return ptr;
    }
    return nullptr;
}

// Array new/delete
class ArrayClass {
private:
    int* data;
    size_t size;

public:
    ArrayClass(size_t s) : size(s) {
        data = new int[size];
        for (size_t i = 0; i < size; i++) {
            data[i] = static_cast<int>(i);
        }
    }

    ~ArrayClass() {
        delete[] data;
    }

    int get(size_t idx) const {
        return data[idx];
    }
};
