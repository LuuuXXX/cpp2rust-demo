//! v6 Phase C：`@make_proxy` 代理工厂骨架生成测试。
//!
//! 验证：
//! 1. 默认（未设置 `CPP2RUST_GEN_PROXY`）时，生成器不输出任何代理工厂骨架；
//! 2. 开启开关后，「继承 C++ 抽象接口的具体类」生成结合 `#[interface(name = ...)]`
//!    的 `@make_proxy` 工厂骨架。
//!
//! 因为开关通过进程级环境变量控制，所有断言集中在单个 `#[test]` 中串行执行，
//! 避免与其他测试并发设置/读取环境变量产生竞态。

mod common;

const PROXY_SRC: &str = r#"
struct Foo {
    virtual ~Foo() {}
    virtual void foo() const = 0;
};

struct Bar : public Foo {
    virtual void bar() const = 0;
};

// 继承接口 Bar（其本身继承 Foo）的具体类，含默认构造函数。
struct Baz : public Bar {
    Baz() {}
    virtual void foo() const override {}
    virtual void bar() const override {}
};

// 不继承任何接口的普通类，不应派生代理工厂。
struct Plain {
    Plain() {}
    void run() {}
};
"#;

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn proxy_skeleton_gated_by_env() {
    // ── 默认关闭：不应输出任何代理工厂骨架 ──
    std::env::remove_var("CPP2RUST_GEN_PROXY");
    let off = match common::generate_from_source("proxy_off", PROXY_SRC) {
        Some(s) => s,
        None => {
            eprintln!("跳过：当前环境缺少 C++ 预处理器或 libclang");
            return;
        }
    };
    assert!(
        !off.contains("@make_proxy"),
        "默认关闭时不应生成 @make_proxy 工厂骨架，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("cpp2rust-todo[PROXY]"),
        "默认关闭时不应出现 PROXY 占位，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("new_rust_baz"),
        "默认关闭时不应生成代理工厂函数，实际输出：\n{}",
        off
    );

    // ── 开启开关：应输出代理工厂骨架 ──
    std::env::set_var("CPP2RUST_GEN_PROXY", "1");
    let on =
        common::generate_from_source("proxy_on", PROXY_SRC).expect("已确认环境可用，生成不应失败");
    std::env::remove_var("CPP2RUST_GEN_PROXY");

    // @make_proxy 工厂 #[cpp(func = ...)] 声明
    assert!(
        on.contains("#[cpp(func = \"Baz @make_proxy<Baz>()\")]"),
        "应生成 Baz 的 @make_proxy 工厂声明，实际输出：\n{}",
        on
    );
    // 结合 #[interface(name = "Bar")]（直接接口基类）
    assert!(
        on.contains("#[interface(name = \"Bar\")]"),
        "应生成结合直接接口基类的 #[interface(name = ...)]，实际输出：\n{}",
        on
    );
    // Rust 工厂函数：第一个参数为 hicc::Interface<Baz>
    assert!(
        on.contains("fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;"),
        "应生成以 hicc::Interface<Baz> 为首参的代理工厂函数，实际输出：\n{}",
        on
    );
    // PROXY 占位注释
    assert!(
        on.contains("cpp2rust-todo[PROXY]"),
        "代理工厂骨架应附带 PROXY 占位注释，实际输出：\n{}",
        on
    );
    // 不继承接口的 Plain 不应派生代理工厂
    assert!(
        !on.contains("new_rust_plain"),
        "不继承接口的 Plain 不应派生代理工厂，实际输出：\n{}",
        on
    );
}
