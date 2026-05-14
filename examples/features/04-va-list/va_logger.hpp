#pragma once
// Demonstrates how cpp2rust-demo handles functions with va_list parameters.
//
// When the *last* parameter of a C++ function is `va_list`, the tool:
//  1. Drops the `va_list` parameter from the Rust signature
//  2. Appends a trailing `...` to the parameter list
//  3. Wraps the binding in `unsafe fn` (variadic calls are inherently unsafe)
//
// The generated Rust binding matches the C-level ABI for variadic functions.
//
// Note: Only the last-parameter `va_list` pattern is supported.
// Full C-style variadic (`...`) without va_list is skipped (HiccLimitation).

#include <stdarg.h>

namespace logger {

/// Log a message with a va_list (last parameter is va_list → unsafe variadic binding).
void log_message(int level, const char* fmt, va_list args);

/// Printf-style formatting helper (last parameter is va_list).
int format_string(char* buf, int buf_size, const char* fmt, va_list args);

/// Normal (non-variadic) function for comparison.
void flush();

}  // namespace logger
