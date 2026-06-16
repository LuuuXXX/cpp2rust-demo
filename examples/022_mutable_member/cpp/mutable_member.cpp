#include "mutable_member.h"

namespace mutable_member_ns {

DataFetcher::DataFetcher(int seed) : seed_(seed), access_count_(0) {}
DataFetcher::~DataFetcher() = default;

// const 方法修改 mutable 成员：合法，因为 access_count_ 被 mutable 修饰。
int DataFetcher::fetch() const {
    ++access_count_;
    return seed_ + access_count_;
}

int DataFetcher::accessCount() const { return access_count_; }

int mutable_member_anchor() { return 0; }

} // namespace mutable_member_ns
