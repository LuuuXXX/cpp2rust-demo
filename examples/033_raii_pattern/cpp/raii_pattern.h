#pragma once

#include <string>

namespace raii_pattern_ns {

// 当前存活的 Resource 实例数量（演示 RAII：构造 +1、析构 -1）。
int active_count();

// 未提交即析构（回滚）的 Transaction 累计数量（演示 RAII 作用域守卫）。
int rollback_count();

// Resource：构造时获取资源（计数 +1），析构时释放资源（计数 -1）。
// hicc 直出无需手写 *_delete；Rust Drop 触发析构时资源自动释放。
class Resource {
    std::string name_;
public:
    explicit Resource(const char* name);
    ~Resource();
    Resource(const Resource&) = delete;
    Resource& operator=(const Resource&) = delete;

    const char* name() const { return name_.c_str(); }
};

// Transaction：作用域守卫。若析构前未 commit()，析构时自动回滚（rollback_count +1）。
class Transaction {
    bool committed_;
public:
    Transaction();
    ~Transaction();
    Transaction(const Transaction&) = delete;
    Transaction& operator=(const Transaction&) = delete;

    void commit() { committed_ = true; }
    int committed() const { return committed_ ? 1 : 0; }
};

} // namespace raii_pattern_ns
