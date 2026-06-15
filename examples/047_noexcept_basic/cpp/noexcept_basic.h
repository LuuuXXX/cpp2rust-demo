#pragma once

#include <cstddef>

#ifdef __cplusplus

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
