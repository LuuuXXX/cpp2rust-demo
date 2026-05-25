#include "default_args.h"
#include <iostream>

int greet(const char* name, int times) {
    for (int i = 0; i < times; ++i) {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
    return times;
}
