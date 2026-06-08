// Abseil string_view extern-C 包装层（string_view_ffi.cpp）
//
// 封装 absl::string_view 为 extern "C" opaque-handle 接口，
// 验证工具在 namespace_class_mode（命名空间类 + void* opaque 指针）路径下
// 能正确生成 import_lib! FFI 绑定（不生成 import_class!）。
//
// 使用系统安装的 libabsl-dev（Ubuntu: apt-get install libabsl-dev）。
// 头文件路径：/usr/include/absl/strings/string_view.h

#include <absl/strings/string_view.h>
#include <cstring>
#include <cstdlib>

extern "C" {

/// 创建 absl::string_view（不拥有 data 所指内存）
void* absl_string_view_create(const char* data, size_t len) {
    return new absl::string_view(data, len);
}

/// 释放 absl::string_view
void absl_string_view_destroy(void* sv) {
    delete static_cast<absl::string_view*>(sv);
}

/// 返回字符串长度
size_t absl_string_view_size(void* sv) {
    return static_cast<absl::string_view*>(sv)->size();
}

/// 返回指向数据的指针（不拥有所有权）
const char* absl_string_view_data(void* sv) {
    return static_cast<absl::string_view*>(sv)->data();
}

/// 判断是否为空
bool absl_string_view_empty(void* sv) {
    return static_cast<absl::string_view*>(sv)->empty();
}

/// 比较两个 string_view：相等返回 0，小于返回负数，大于返回正数
int absl_string_view_compare(void* a, void* b) {
    return static_cast<absl::string_view*>(a)->compare(
        *static_cast<absl::string_view*>(b));
}

} // extern "C"
