#include "tuple_basic.h"
#include <iostream>

int main() {
    using namespace tuple_basic_ns;

    Record record(42, 98.5, "alice");
    std::cout << "id=" << record.id()
              << " score=" << record.score()
              << " name=" << record.name() << "\n";
    record.set_id(100);
    record.set_score(88.25);
    std::cout << "after set id=" << record.id()
              << " score=" << record.score()
              << " name=" << record.name() << "\n";
    return 0;
}
