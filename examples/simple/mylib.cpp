#include "mylib.hpp"
#include <cstring>
#include <cstdio>

namespace mylib {

int add(int a, int b) { return a + b; }

double scale(double x, double factor) { return x * factor; }

int string_length(const char* str) { return str ? (int)std::strlen(str) : 0; }

int log_message(const char* level, const char* msg) {
    std::printf("[%s] %s\n", level, msg);
    return 0;
}

void process(int value)        { std::printf("process(int=%d)\n", value); }
void process(double value)     { std::printf("process(double=%f)\n", value); }
void process(const char* value) { std::printf("process(str=%s)\n", value); }

} // namespace mylib
