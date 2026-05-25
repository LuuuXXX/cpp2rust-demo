#pragma once

#include <cstddef>
#include <cstdint>

// C++ enum class definitions (must be outside extern "C")
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

#ifdef __cplusplus
extern "C" {
#endif

// FFI functions - use example::OperationResult directly
example::OperationResult* operation_result_new(void);
void operation_result_delete(example::OperationResult* p);
void operation_result_set_error(example::OperationResult* p, int error_code);
int operation_result_get_error(example::OperationResult* p);
void operation_result_set_state(example::OperationResult* p, unsigned char state);
unsigned char operation_result_get_state(example::OperationResult* p);
void operation_result_set_flags(example::OperationResult* p, unsigned int flags);
unsigned int operation_result_get_flags(example::OperationResult* p);
unsigned int combine_flags(unsigned int f1, unsigned int f2);
int has_flag(unsigned int flags, unsigned int flag);

#ifdef __cplusplus
}
#endif
