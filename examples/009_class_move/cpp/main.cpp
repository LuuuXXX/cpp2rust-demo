#include "class_move.h"

int main() {
    {
        class_move_ns::UniqueVector src(3);
        for (int i = 0; i < 3; ++i) {
            src.set(i, (i + 1) * 100);
        }

        class_move_ns::UniqueVector dest;
        dest.move_from(src); // 资源从 src 移动到 dest

        std::cout << "dest size=" << dest.size()
                  << " dest[0]=" << dest.get(0)
                  << " src size=" << src.size() << std::endl;
    }
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
