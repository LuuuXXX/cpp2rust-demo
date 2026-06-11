//! v6 Phase C（续）：`@dynamic_cast` 下行转换骨架生成测试。
//!
//! 验证：
//! 1. 默认（未设置 `CPP2RUST_GEN_DYNAMIC_CAST`）时，生成器不输出任何下行转换骨架；
//! 2. 开启开关后，「继承多态基类的派生类」生成 `@dynamic_cast` 下行转换骨架。
//!
//! 因为开关通过进程级环境变量控制，所有断言集中在单个 `#[test]` 中串行执行，
//! 避免与其他测试并发设置/读取环境变量产生竞态。

mod common;

const DCAST_SRC: &str = r#"
struct Foo {
    virtual ~Foo() {}
    virtual void foo() const {}
};

// 继承多态基类 Foo 的派生类，应派生 Foo -> Bar 的下行转换。
struct Bar : public Foo {
    virtual void foo() const override {}
};

// 三层继承：Baz 间接继承多态基类 Foo（经 Bar）。
// v6 Phase C（收尾）：应额外派生跨层下行转换 Foo -> Baz（以及 Bar -> Baz）。
struct Baz : public Bar {
    void extra() const {}
};

// 无虚函数的非多态类，其派生类不应派生下行转换。
struct Plain {
    void run() {}
};

struct PlainChild : public Plain {
    void go() {}
};
"#;

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn dynamic_cast_skeleton_gated_by_env() {
    // ── 默认关闭：不应输出任何下行转换骨架 ──
    std::env::remove_var("CPP2RUST_GEN_DYNAMIC_CAST");
    let off = match common::generate_from_source("dcast_off", DCAST_SRC) {
        Some(s) => s,
        None => {
            eprintln!("跳过：当前环境缺少 C++ 预处理器或 libclang");
            return;
        }
    };
    assert!(
        !off.contains("@dynamic_cast"),
        "默认关闭时不应生成 @dynamic_cast 骨架，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("cpp2rust-todo[DCAST]"),
        "默认关闭时不应出现 DCAST 占位，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("dynamic_cast_foo_to_bar"),
        "默认关闭时不应生成下行转换函数，实际输出：\n{}",
        off
    );

    // ── 开启开关：应输出下行转换骨架 ──
    std::env::set_var("CPP2RUST_GEN_DYNAMIC_CAST", "1");
    let on =
        common::generate_from_source("dcast_on", DCAST_SRC).expect("已确认环境可用，生成不应失败");
    std::env::remove_var("CPP2RUST_GEN_DYNAMIC_CAST");

    // @dynamic_cast 下行转换 #[cpp(func = ...)] 声明
    assert!(
        on.contains("#[cpp(func = \"const Bar* @dynamic_cast<const Bar*>(const Foo*)\")]"),
        "应生成 Foo -> Bar 的 @dynamic_cast 声明，实际输出：\n{}",
        on
    );
    // Rust 下行转换函数：以多态基类裸指针为入参，返回派生类裸指针
    assert!(
        on.contains("pub unsafe fn dynamic_cast_foo_to_bar(src: *const Foo) -> *const Bar;"),
        "应生成以 *const Foo 为入参的下行转换函数，实际输出：\n{}",
        on
    );
    // DCAST 占位注释
    assert!(
        on.contains("cpp2rust-todo[DCAST]"),
        "下行转换骨架应附带 DCAST 占位注释，实际输出：\n{}",
        on
    );
    // 跨层（间接）下行转换：Foo 是 Baz 的间接多态祖先，应派生 Foo -> Baz
    assert!(
        on.contains("#[cpp(func = \"const Baz* @dynamic_cast<const Baz*>(const Foo*)\")]"),
        "应生成跨层下行转换 Foo -> Baz 的 @dynamic_cast 声明，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("pub unsafe fn dynamic_cast_foo_to_baz(src: *const Foo) -> *const Baz;"),
        "应生成跨层下行转换函数 Foo -> Baz，实际输出：\n{}",
        on
    );
    // 直接基类下行转换 Bar -> Baz 也应存在
    assert!(
        on.contains("pub unsafe fn dynamic_cast_bar_to_baz(src: *const Bar) -> *const Baz;"),
        "应生成 Bar -> Baz 的下行转换函数，实际输出：\n{}",
        on
    );
    // 引用形式（&Src -> &Dst）：同一指针型 C++ 签名，Rust 侧返回 &Dst
    assert!(
        on.contains("pub unsafe fn dynamic_cast_foo_to_bar_ref(src: &Foo) -> &Bar;"),
        "应生成 Foo -> Bar 的引用形式下行转换函数，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("pub unsafe fn dynamic_cast_foo_to_baz_ref(src: &Foo) -> &Baz;"),
        "应生成跨层引用形式下行转换函数 Foo -> Baz，实际输出：\n{}",
        on
    );
    // 非多态基类 Plain 的派生类不应派生下行转换
    assert!(
        !on.contains("dynamic_cast_plain_to_plain_child"),
        "非多态基类 Plain 的派生类不应派生下行转换，实际输出：\n{}",
        on
    );
}
