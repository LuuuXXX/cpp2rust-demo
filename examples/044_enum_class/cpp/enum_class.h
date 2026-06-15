#pragma once

#include <cstddef>
#include <cstdint>

namespace example {

enum class ErrorCode : int {
    None = 0,
    InvalidInput = 1,
    OutOfMemory = 2,
    NotFound = 3,
    PermissionDenied = 4,
    Unknown = 99
};

enum class State : unsigned char {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3
};

enum class Flags : unsigned int {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    All = 7
};

class OperationResult {
private:
    ErrorCode error_;
    State state_;
    Flags flags_;
public:
    OperationResult();
    ~OperationResult();
    void set_error(int code);
    [[nodiscard]] int get_error() const;
    void set_state(unsigned char s);
    [[nodiscard]] unsigned char get_state() const;
    void set_flags(unsigned int f);
    [[nodiscard]] unsigned int get_flags() const;
};

}  // namespace example
