#pragma once

#include <string>

namespace unique_ptr_ns {

// UniqueBuffer：独占持有一段缓冲区。hicc 直出用 unique_ptr 管理其所有权，
// 析构自动完成，无需手写 *_delete。
class UniqueBuffer {
    std::string data_;
public:
    explicit UniqueBuffer(int sz) : data_(sz, '\0') {}

    int size() const { return static_cast<int>(data_.size()); }
    char* data() { return &data_[0]; }
    void fill(char c) { for (auto& ch : data_) ch = c; }
    char at(int i) const { return data_[i]; }

    // 独占所有权：use_count 恒为 1。
    int use_count() const { return 1; }
};

// Processor：内部缓冲区，处理输入字符串。
class Processor {
    std::string buffer_;
public:
    Processor() = default;

    const char* process(const char* input) {
        if (input) {
            buffer_ = std::string(input) + " [processed]";
        }
        return buffer_.c_str();
    }
};

// 锚点：本单元可链接的非模板符号。
int unique_ptr_anchor();

} // namespace unique_ptr_ns
