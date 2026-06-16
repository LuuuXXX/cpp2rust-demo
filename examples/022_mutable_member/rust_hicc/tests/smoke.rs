//! 022_mutable_member 冒烟测试：const 方法经 mutable 成员更新访问计数。

use mutable_member::*;

#[test]
fn const_method_updates_mutable() {
    let f = DataFetcher::new(100);
    assert_eq!(f.access_count(), 0);
    assert_eq!(f.fetch(), 101);
    assert_eq!(f.fetch(), 102);
    assert_eq!(f.access_count(), 2);
}
