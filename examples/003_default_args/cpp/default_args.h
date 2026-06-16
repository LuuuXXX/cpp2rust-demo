#pragma once

namespace default_args_ns {

// 命名空间内自由函数 + C++ 默认参数（times 默认 1）：无需 extern "C"，
// 由 hicc import_lib! 以 ns::fn() 直出绑定（FFI 层需显式传全部实参）。
int greet(const char* name, int times = 1);

} // namespace default_args_ns
