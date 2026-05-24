// Example 08: Template Classes
// C++ features: class templates, template parameters, partial specialization

#include <cstdlib>

// Stack template
template<typename T, size_t SIZE = 100>
class Stack {
private:
    T data[SIZE];
    size_t top;

public:
    Stack() : top(0) {}

    void push(T value) {
        if (top < SIZE) {
            data[top++] = value;
        }
    }

    T pop() {
        if (top > 0) {
            return data[--top];
        }
        return T();
    }

    bool is_empty() const {
        return top == 0;
    }

    bool is_full() const {
        return top == SIZE;
    }

    size_t size() const {
        return top;
    }
};

// Pair template
template<typename K, typename V>
class Pair {
private:
    K key;
    V value;

public:
    Pair() {}
    Pair(const K& k, const V& v) : key(k), value(v) {}

    K get_key() const { return key; }
    V get_value() const { return value; }
    void set_key(const K& k) { key = k; }
    void set_value(const V& v) { value = v; }
};

// Fixed-size array template
template<typename T, size_t N>
class FixedArray {
private:
    T data[N];

public:
    FixedArray() {}

    T& operator[](size_t idx) {
        return data[idx];
    }

    const T& operator[](size_t idx) const {
        return data[idx];
    }

    size_t length() const {
        return N;
    }
};

// Template with non-type parameter
template<size_t SIZE>
class BitSet {
private:
    unsigned int data[(SIZE + 31) / 32];

public:
    BitSet() {
        for (size_t i = 0; i < (SIZE + 31) / 32; i++) {
            data[i] = 0;
        }
    }

    void set(size_t bit) {
        if (bit < SIZE) {
            data[bit / 32] |= (1u << (bit % 32));
        }
    }

    void clear(size_t bit) {
        if (bit < SIZE) {
            data[bit / 32] &= ~(1u << (bit % 32));
        }
    }

    bool test(size_t bit) const {
        if (bit < SIZE) {
            return (data[bit / 32] & (1u << (bit % 32))) != 0;
        }
        return false;
    }
};
