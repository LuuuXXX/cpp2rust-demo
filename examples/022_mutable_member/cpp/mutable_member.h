#pragma once

namespace mutable_member_ns {

// mutable 成员：被 mutable 修饰的成员即使在 const 方法中也可被修改。
class DataFetcher {
public:
    explicit DataFetcher(int seed);
    ~DataFetcher();

    // const 方法：逻辑上不改变对象「可见状态」，但更新 mutable 的访问计数缓存。
    int fetch() const;
    int accessCount() const;

private:
    int seed_;
    mutable int access_count_;
};

// 锚点：触发 detect_idiomatic_mode 走直出路径。
int mutable_member_anchor();

} // namespace mutable_member_ns
