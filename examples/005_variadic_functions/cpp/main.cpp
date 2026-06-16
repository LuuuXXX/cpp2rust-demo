#include "variadic_functions.h"
#include <cstdio>

int main() {
    printf("sum_3 = %d\n", variadic_functions_ns::sum_3(1, 2, 3));
    printf("sum_5 = %d\n", variadic_functions_ns::sum_5(1, 2, 3, 4, 5));
    return 0;
}
