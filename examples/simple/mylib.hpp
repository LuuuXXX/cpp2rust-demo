#pragma once
// Simple C++ library header for the cpp2rust-demo simple example.

namespace mylib {
    /// Add two integers.
    int add(int a, int b);

    /// Scale a value by a factor.
    double scale(double x, double factor);

    /// Return the length of a C string.
    int string_length(const char* str);

    /// Log a message (returns 0 on success).
    int log_message(const char* level, const char* msg);

    // Function overloads – cpp2rust-demo handles these with numeric suffixes.
    void process(int value);
    void process(double value);
    void process(const char* value);
}
