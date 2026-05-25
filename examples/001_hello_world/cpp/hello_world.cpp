#include "hello_world.h"
#include <iostream>

extern "C" {

void hello_world(void) {
    std::cout << "Hello, World!" << std::endl;
}

}
