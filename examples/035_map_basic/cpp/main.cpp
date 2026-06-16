#include "map_basic.h"
#include <iostream>

int main() {
    using namespace map_basic_ns;

    StringIntMap m;
    m.insert("apple", 3);
    m.insert("banana", 5);
    m.insert("apple", 7);
    std::cout << "size=" << m.size()
              << " apple=" << m.get("apple")
              << " banana?=" << m.contains("banana") << "\n";
    std::cout << "missing=" << m.get("missing")
              << " first_key=" << m.first_key() << "\n";
    std::cout << "erase banana=" << m.erase("banana")
              << " size=" << m.size() << "\n";
    m.clear();
    std::cout << "after clear size=" << m.size() << "\n";

    Counter c;
    c.add("rust");
    c.add("cpp");
    c.add("rust");
    std::cout << "counter rust=" << c.count("rust")
              << " cpp=" << c.count("cpp")
              << " unique=" << c.unique_words()
              << " last=" << c.last_word() << "\n";
    return 0;
}
