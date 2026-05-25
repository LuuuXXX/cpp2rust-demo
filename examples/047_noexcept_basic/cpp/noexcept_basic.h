#pragma once

#include <cstddef>

#ifdef __cplusplus
extern "C" {
#endif

// noexcept functions - guaranteed not to throw
int noexcept_add(int a, int b) noexcept;
int noexcept_multiply(int a, int b) noexcept;

// Function that may throw
int throwing_divide(int a, int b);

// noexcept operator check
int check_noexcept(int (*fn)(int, int)) noexcept;

// Conditional noexcept
int conditional_abs(int value) noexcept;

// Move-only type with noexcept move operations
struct NoexceptMover;

struct NoexceptMover* noexcept_mover_new(int value);
void noexcept_mover_delete(struct NoexceptMover* self);
struct NoexceptMover* noexcept_mover_move(struct NoexceptMover* other) noexcept;

// Check if a function pointer is noexcept
int is_noexcept(int (*fn)(int, int)) noexcept;

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class NoexceptMover {
private:
    int value_;
public:
    NoexceptMover(int value);
    ~NoexceptMover();
    NoexceptMover(NoexceptMover&& other) noexcept;
    NoexceptMover& operator=(NoexceptMover&& other) noexcept;
    [[nodiscard]] int get_value() const;
    NoexceptMover(const NoexceptMover&) = delete;
    NoexceptMover& operator=(const NoexceptMover&) = delete;
};

#endif
