#pragma once

namespace hello_world_ns {

// 命名空间内自由函数：无需 extern "C"，由 hicc import_lib! 以 ns::fn() 直出绑定。
void hello_world();

} // namespace hello_world_ns
