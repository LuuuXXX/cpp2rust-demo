#pragma once

#include <cstdint>

namespace enum_class_ns {

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
    ErrorCode error_;
    State state_;
    Flags flags_;
public:
    OperationResult();

    void set_error(int code);
    int get_error() const;
    void set_state(unsigned char s);
    unsigned char get_state() const;
    void set_flags(unsigned int f);
    unsigned int get_flags() const;
};

unsigned int combine_flags(unsigned int f1, unsigned int f2);
int has_flag(unsigned int flags, unsigned int flag);

// 锚点：本单元可链接的非模板符号。
int enum_class_anchor();

} // namespace enum_class_ns
