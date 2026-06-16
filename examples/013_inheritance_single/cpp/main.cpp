#include "inheritance_single.h"
#include <iostream>

int main() {
    using namespace inheritance_single_ns;
    Animal animal("Generic");
    Dog dog("Buddy");

    std::cout << animal.name() << ": " << animal.speak() << std::endl;
    std::cout << dog.name() << ": " << dog.speak() << std::endl;
    std::cout << dog.bark() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
