// Example 02: Pointers and References
// C++ features: pointers, references, function pointers, smart pointer concepts

#include <cstdint>

// Counter class for demonstrating object lifetime
class Counter {
private:
    int value;
public:
    Counter() : value(0) {}
    Counter(int v) : value(v) {}
    int get() const { return value; }
    void set(int v) { value = v; }
    void increment() { value++; }
    void decrement() { value--; }
};

// Pointer passing and modification
void increment(int* p) {
    if (p) (*p)++;
}

void decrement(int* p) {
    if (p) (*p)--;
}

// Reference passing
void increment_ref(int& r) {
    r++;
}

void decrement_ref(int& r) {
    r--;
}

// Double reference (pointer to reference is not allowed, but reference to pointer)
void ptr_to_ptr(int** pp) {
    if (pp && *pp) (**pp)++;
}

// Create new int on heap
int* create_int(int v) {
    int* p = new int(v);
    return p;
}

// Delete int from heap
void destroy_int(int* p) {
    delete p;
}

// Swap using pointers
void swap_ptr(int* a, int* b) {
    if (a && b) {
        int temp = *a;
        *a = *b;
        *b = temp;
    }
}

// Swap using references
void swap_ref(int& a, int& b) {
    int temp = a;
    a = b;
    b = temp;
}

// Null pointer check
bool is_null_ptr(const int* p) {
    return p == nullptr;
}

// Pointer arithmetic
int* advance_ptr(int* p, int n) {
    return p + n;
}

// Function pointer type
typedef int (*BinaryOp)(int, int);

int apply_op(int a, int b, BinaryOp op) {
    return op(a, b);
}

int add(int a, int b) { return a + b; }
int multiply(int a, int b) { return a * b; }
int subtract(int a, int b) { return a - b; }

// Object pointer methods
Counter* create_counter(int initial) {
    return new Counter(initial);
}

void destroy_counter(Counter* c) {
    delete c;
}

int get_counter(Counter* c) {
    return c ? c->get() : -1;
}
