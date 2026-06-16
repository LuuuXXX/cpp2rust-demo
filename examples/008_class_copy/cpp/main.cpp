#include "class_copy.h"

int main() {
    {
        class_copy_ns::Buffer b1(5);
        for (int i = 0; i < 5; ++i) {
            b1.set(i, (i + 1) * 10);
        }
        class_copy_ns::Buffer b2(b1); // 深拷贝

        b1.set(0, 999); // 修改原对象不应影响拷贝
        std::cout << "b1[0]=" << b1.get(0) << " b2[0]=" << b2.get(0) << std::endl;
        std::cout << "b2 size=" << b2.size() << std::endl;
    }
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
