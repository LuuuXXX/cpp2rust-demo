#include "summary.h"
#include <cstdint>

Counter* counter_new() {
    return new Counter();
}

void counter_delete(struct Counter* self) {
    delete self;
}

int safe_add(int a, int b) {
    return a + b;
}

int get_max_size() {
    const int MAX_SIZE = 100;
    return MAX_SIZE;
}
