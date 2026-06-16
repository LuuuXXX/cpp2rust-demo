#pragma once

#include <string>
#include <cstdio>
#include <cstring>

namespace template_specialization_ns {

// 通用类模板：ValueHolder<T> —— 默认实现。
template <typename T>
class ValueHolder {
    T value_;
public:
    explicit ValueHolder(T value) : value_(value) {}
    T get() const { return value_; }
    const char* describe() const {
        static char buf[64];
        std::snprintf(buf, sizeof(buf), "ValueHolder<T>(generic)");
        return buf;
    }
};

// 全特化：ValueHolder<std::string> —— 针对字符串的不同实现。
// 构造函数设为私有并仅对具体包装类 StringHolder 开放（friend），避免 hicc 直出
// 将这个「模板特化」当作可独立实例化的普通类来绑定（其类名带模板实参，不可裸用）。
template <>
class ValueHolder<std::string> {
    std::string value_;
    explicit ValueHolder(std::string value) : value_(std::move(value)) {}
    friend class StringHolder;
public:
    const char* get() const { return value_.c_str(); }
    const char* describe() const {
        static char buf[256];
        std::snprintf(buf, sizeof(buf),
                      "ValueHolder<std::string>(value=\"%s\", length=%zu)",
                      value_.c_str(), value_.size());
        return buf;
    }
};

// 具体实例化：每个具体类型暴露一个 idiomatic 命名空间类。
class IntHolder {
    ValueHolder<int> impl_;
public:
    explicit IntHolder(int value) : impl_(value) {}
    int get() const { return impl_.get(); }
    const char* describe() const { return impl_.describe(); }
};

class DoubleHolder {
    ValueHolder<double> impl_;
public:
    explicit DoubleHolder(double value) : impl_(value) {}
    double get() const { return impl_.get(); }
    const char* describe() const { return impl_.describe(); }
};

// StringHolder 走特化版本 ValueHolder<std::string>。
class StringHolder {
    ValueHolder<std::string> impl_;
public:
    explicit StringHolder(const char* value) : impl_(std::string(value)) {}
    const char* get() const { return impl_.get(); }
    const char* describe() const { return impl_.describe(); }
};

// 锚点：本单元可链接的非模板符号。
int template_specialization_anchor();

} // namespace template_specialization_ns
