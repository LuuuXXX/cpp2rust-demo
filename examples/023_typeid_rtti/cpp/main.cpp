#include "typeid_rtti.h"
#include <iostream>
#include <typeinfo>

int main() {
    using namespace typeid_rtti_ns;
    Circle c(2.0);
    Rectangle r(3.0, 4.0);
    Triangle t(6.0, 2.0);

    // 经基类引用用 typeid 取回动态类型（RTTI）。
    const Shape& sc = c;
    const Shape& sr = r;
    std::cout << "circle area=" << c.area()
              << " typeid=" << typeid(sc).name() << std::endl;
    std::cout << "rect area=" << r.area()
              << " typeid=" << typeid(sr).name() << std::endl;
    std::cout << "triangle area=" << t.area() << std::endl;
    std::cout << "same type (c,c)=" << (typeid(sc) == typeid((const Shape&)c)) << std::endl;
    std::cout << "same type (c,r)=" << (typeid(sc) == typeid(sr)) << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
