#include "default_args.h"
#include <iostream>

namespace default_args_ns {

int greet(const char* name, int times) {
    for (int i = 0; i < times; ++i) {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
    return times;
}

} // namespace default_args_ns
