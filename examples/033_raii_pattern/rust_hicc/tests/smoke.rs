//! 033_raii_pattern 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 RAII 语义正确。
//!
//! 注：`active_count` 仅由本文件中的 Resource 测试触达、`rollback_count` 仅由 Transaction
//! 测试触达，两者互不影响，因此在 cargo 默认并行测试下断言仍然确定。

use raii_pattern::*;

#[test]
fn smoke_resource_raii() {
    let name = std::ffi::CString::new("db").expect("CString::new failed");
    let r = Resource::new(name.as_ptr());
    let nm = unsafe { std::ffi::CStr::from_ptr(r.name()).to_string_lossy().into_owned() };
    assert_eq!(nm, "db");

    // r 此刻存活，以当前计数为基线观察 RAII 增减。
    let base = active_count();
    {
        let n2 = std::ffi::CString::new("file").expect("CString::new failed");
        let _r2 = Resource::new(n2.as_ptr());
        assert_eq!(active_count(), base + 1, "构造应使活跃计数 +1");
    } // Drop → 析构释放
    assert_eq!(active_count(), base, "析构应使活跃计数恢复");
}

#[test]
fn smoke_transaction_raii() {
    let mut t = Transaction::new();
    assert_eq!(t.committed(), 0);
    t.commit();
    assert_eq!(t.committed(), 1);

    let base = rollback_count();
    {
        let _t2 = Transaction::new(); // 未提交 → Drop 时回滚
    }
    assert_eq!(rollback_count(), base + 1, "未提交事务析构应回滚一次");

    {
        let mut t3 = Transaction::new();
        t3.commit();
    } // 已提交 → 不回滚
    assert_eq!(rollback_count(), base + 1, "已提交事务析构不应回滚");
}
