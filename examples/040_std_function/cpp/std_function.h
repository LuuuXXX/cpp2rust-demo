#pragma once

#include <functional>
#include <vector>

namespace std_function_ns {

// Callback：内部持有由 lambda 构造的 std::function，按 kind 选择运算。
// 演示 C++ std::function 在 C++ 侧内部持有回调，hicc 直出无需跨 FFI 传函数指针。
class Callback {
    std::function<int(int)> fn_;
public:
    // kind: 0=double, 1=triple, 2=negate
    explicit Callback(int kind);
    Callback(const Callback&) = delete;
    Callback& operator=(const Callback&) = delete;

    int invoke(int v) const { return fn_(v); }
};

// Pipeline：内部持有 std::function 序列，按 add 顺序依次处理输入。
class Pipeline {
    std::vector<std::function<int(int)>> fns_;
public:
    Pipeline() = default;
    Pipeline(const Pipeline&) = delete;
    Pipeline& operator=(const Pipeline&) = delete;

    void add(int kind);
    int run(int v) const;
    int size() const { return static_cast<int>(fns_.size()); }
};

// 锚点：本单元可链接的非模板符号。
int std_function_anchor();

} // namespace std_function_ns
