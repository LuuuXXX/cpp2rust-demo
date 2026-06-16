#include "raii_pattern.h"
#include <iostream>

int main() {
    using namespace raii_pattern_ns;

    std::cout << "active_count(start)=" << active_count() << "\n";
    {
        Resource a("db");
        std::cout << "after acquire a: name=" << a.name()
                  << " active=" << active_count() << "\n";
        {
            Resource b("file");
            std::cout << "after acquire b: active=" << active_count() << "\n";
        }
        std::cout << "after b released: active=" << active_count() << "\n";
    }
    std::cout << "after all released: active=" << active_count() << "\n";

    int rb = rollback_count();
    {
        Transaction t1;
        t1.commit();
        std::cout << "t1 committed=" << t1.committed() << "\n";
    }
    {
        Transaction t2; // 未提交 → 析构回滚
        std::cout << "t2 committed=" << t2.committed() << "\n";
    }
    std::cout << "rollback delta=" << (rollback_count() - rb) << "\n";
    return 0;
}
