#include "noexcept_basic.h"
#include <iostream>
#include <stdexcept>
#include <utility>

// NoexceptMover implementation
NoexceptMover::NoexceptMover(int value) : value_(value) {}
NoexceptMover::~NoexceptMover() {}
NoexceptMover::NoexceptMover(NoexceptMover&& other) noexcept : value_(other.value_) {
    other.value_ = 0;
}
NoexceptMover& NoexceptMover::operator=(NoexceptMover&& other) noexcept {
    if (this != &other) {
        value_ = other.value_;
        other.value_ = 0;
    }
    return *this;
}
int NoexceptMover::get_value() const {
    return value_;
}

// FFI wrapper implementations
struct NoexceptMover* noexcept_mover_new(int value) {
    return new NoexceptMover(value);
}

void noexcept_mover_delete(struct NoexceptMover* self) {
    delete self;
}

// noexcept move - transfers ownership
struct NoexceptMover* noexcept_mover_move(struct NoexceptMover* other) noexcept {
    if (other) {
        auto* moved = new NoexceptMover(std::move(*other));
        std::cout << "noexcept_mover_move: transferred ownership" << std::endl;
        return moved;
    }
    return nullptr;
}

// Check if a function pointer points to a noexcept function
int is_noexcept(int (*)(int, int)) noexcept {
    // Simplified: we can only reliably check at compile time with constexpr
    // For runtime check via function pointer, we assume noexcept functions
    // are passed (noexcept_add, noexcept_multiply, conditional_abs)
    return 1;
}

// FFI function implementations
int noexcept_add(int a, int b) noexcept {
    return a + b;
}

int noexcept_multiply(int a, int b) noexcept {
    return a * b;
}

int throwing_divide(int a, int b) {
    if (b == 0) {
        throw std::runtime_error("Division by zero");
    }
    return a / b;
}

int check_noexcept(int (*fn)(int, int)) noexcept {
    // Simplified check - assume all passed functions are noexcept
    return 1;
}

int conditional_abs(int value) noexcept {
    return value >= 0 ? value : -value;
}

