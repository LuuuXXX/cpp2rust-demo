#include "default_args.h"

int main() {
    default_args_ns::greet("World");       // 使用默认 times = 1
    default_args_ns::greet("World", 2);    // 显式传入 times = 2
    return 0;
}
