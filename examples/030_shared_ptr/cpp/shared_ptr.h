#pragma once

#include <string>
#include <memory>
#include <vector>

namespace shared_ptr_ns {

// SharedData：内部用 std::shared_ptr 持有字符串负载，演示共享所有权与引用计数。
// hicc 直出无需 extern-C shim，析构由 Rust Drop 自动完成。
class SharedData {
    std::shared_ptr<std::string> data_;
public:
    explicit SharedData(const char* name)
        : data_(std::make_shared<std::string>(name ? name : "")) {}

    const char* name() const { return data_->c_str(); }

    // 当前控制块的引用计数（每个独立 SharedData 默认为 1）。
    int use_count() const { return static_cast<int>(data_.use_count()); }

    // 释放对负载的持有，模拟 shared_ptr::reset。
    void reset() { data_.reset(); }

    // 负载是否已释放（类似 weak_ptr::expired）。
    int expired() const { return data_ ? 0 : 1; }
};

// Cache：用 vector<shared_ptr> 缓存负载，演示同一资源被多处共享时引用计数增长。
class Cache {
    std::vector<std::shared_ptr<std::string>> entries_;
public:
    Cache() = default;

    // 缓存一个名字，返回缓存后该资源的引用计数（本地 sp + 缓存副本 = 2）。
    int store(const char* name) {
        auto sp = std::make_shared<std::string>(name ? name : "");
        entries_.push_back(sp);
        return static_cast<int>(sp.use_count());
    }

    int size() const { return static_cast<int>(entries_.size()); }
    void clear() { entries_.clear(); }
};

// 锚点：本单元可链接的非模板符号。
int shared_ptr_anchor();

} // namespace shared_ptr_ns
