// L1/L2 mirror tests for test/unittest/bigintegertest.cpp
//
// This file name is intentionally aligned with the legacy gtest source:
// - C++: test/unittest/bigintegertest.cpp
// - Rust: rapidjson-rs/tests/bigintegertest.rs
//
// L1: C++ backend via FFI (CxxBigIntegerAdapter)
// L2: Rust backend via native implementation (RustBigIntegerAdapter)

use rapidjson_sys::bigintegertest_ffi::CxxBigInteger;
use rapidjson_rs::internal::biginteger::BigInteger as RustBigInteger;

// Domain trait for BigInteger behavior used in tests.
// The methods reflect the operations exercised in bigintegertest.cpp.
trait TestBigInteger {
    fn zero() -> Self
    where
        Self: Sized;

    fn from_u64(value: u64) -> Self
    where
        Self: Sized;

    fn from_decimal_literal(lit: &str) -> Self
    where
        Self: Sized;

    fn add_u64(&mut self, value: u64);
    fn mul_u64(&mut self, value: u64);
    fn mul_u32(&mut self, value: u32);
    fn shl_bits(&mut self, bits: u32);
    fn compare(&self, other: &Self) -> i32;
    fn is_zero(&self) -> bool;
}

struct CxxBigIntegerAdapter(CxxBigInteger);

impl CxxBigIntegerAdapter {
    fn new_zero() -> Self {
        Self(CxxBigInteger::from_decimal_literal("0"))
    }
}

impl TestBigInteger for CxxBigIntegerAdapter {
    fn zero() -> Self {
        Self::new_zero()
    }

    fn from_u64(value: u64) -> Self {
        let mut v = CxxBigInteger::from_decimal_literal("0");
        v.add_u64(value);
        Self(v)
    }

    fn from_decimal_literal(lit: &str) -> Self {
        Self(CxxBigInteger::from_decimal_literal(lit))
    }

    fn add_u64(&mut self, value: u64) {
        self.0.add_u64(value);
    }

    fn mul_u64(&mut self, value: u64) {
        self.0.mul_u64(value);
    }

    fn mul_u32(&mut self, value: u32) {
        self.0.mul_u32(value);
    }

    fn shl_bits(&mut self, bits: u32) {
        self.0.shl_bits(bits);
    }

    fn compare(&self, other: &Self) -> i32 {
        self.0.compare(&other.0)
    }

    fn is_zero(&self) -> bool {
        let zero = Self::zero();
        self.compare(&zero) == 0
    }
}

struct RustBigIntegerAdapter(RustBigInteger);

impl RustBigIntegerAdapter {
    fn new_zero() -> Self {
        Self(RustBigInteger::zero())
    }
}

impl TestBigInteger for RustBigIntegerAdapter {
    fn zero() -> Self {
        Self::new_zero()
    }

    fn from_u64(value: u64) -> Self {
        Self(RustBigInteger::from_u64(value))
    }

    fn from_decimal_literal(lit: &str) -> Self {
        let v = RustBigInteger::from_decimal_str(lit)
            .unwrap_or_else(|| panic!("invalid decimal literal: {}", lit));
        Self(v)
    }

    fn add_u64(&mut self, value: u64) {
        self.0 = self.0.add_u64(value);
    }

    fn mul_u64(&mut self, value: u64) {
        self.0 = self.0.mul_u64(value);
    }

    fn mul_u32(&mut self, value: u32) {
        self.0 = self.0.mul_u32(value);
    }

    fn shl_bits(&mut self, bits: u32) {
        self.0 = self.0.shl_bits(bits);
    }

    fn compare(&self, other: &Self) -> i32 {
        use core::cmp::Ordering::*;
        match self.0.cmp(&other.0) {
            Less => -1,
            Equal => 0,
            Greater => 1,
        }
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

// Mirror TEST(BigInteger, Constructor)
fn run_biginteger_constructor<M>()
where
    M: TestBigInteger,
{
    // Constants from bigintegertest.cpp
    let zero = M::from_decimal_literal("0");
    let one = M::from_decimal_literal("1");
    let two64 = M::from_decimal_literal("18446744073709551616");

    // EXPECT_TRUE(kZero.IsZero());
    assert!(zero.is_zero());

    // EXPECT_TRUE(kZero == kZero);
    assert_eq!(zero.compare(&zero), 0);

    // EXPECT_TRUE(kZero == BIGINTEGER_LITERAL("0"));
    let zero2 = M::from_decimal_literal("0");
    assert_eq!(zero.compare(&zero2), 0);

    // EXPECT_TRUE(kZero == BIGINTEGER_LITERAL("00"));
    let zero3 = M::from_decimal_literal("00");
    assert_eq!(zero.compare(&zero3), 0);

    // const BigInteger a(123);
    let a = M::from_u64(123);
    // EXPECT_TRUE(a == a);
    assert_eq!(a.compare(&a), 0);

    // EXPECT_TRUE(a == BIGINTEGER_LITERAL("123"));
    let a2 = M::from_decimal_literal("123");
    assert_eq!(a.compare(&a2), 0);

    // EXPECT_TRUE(a == BIGINTEGER_LITERAL("0123"));
    let a3 = M::from_decimal_literal("0123");
    assert_eq!(a.compare(&a3), 0);

    // Representation-specific checks (GetCount/GetDigit) are omitted
    // here on purpose to keep the test focused on observable
    // behaviour rather than internal storage layout.
    let _ = one;
    let _ = two64;
}

// Mirror TEST(BigInteger, AddUint64)
fn run_biginteger_add_uint64<M>()
where
    M: TestBigInteger,
{
    let zero = M::from_decimal_literal("0");
    let one = M::from_decimal_literal("1");
    let two = M::from_decimal_literal("2");
    let uint64_max = M::from_decimal_literal("18446744073709551615");
    let two64 = M::from_decimal_literal("18446744073709551616");

    // BigInteger a = kZero;
    let mut a = M::from_decimal_literal("0");
    // a += 0u; EXPECT_TRUE(kZero == a);
    a.add_u64(0);
    assert_eq!(a.compare(&zero), 0);

    // a += 1u; EXPECT_TRUE(kOne == a);
    a.add_u64(1);
    assert_eq!(a.compare(&one), 0);

    // a += 1u; EXPECT_TRUE(BigInteger(2) == a);
    a.add_u64(1);
    assert_eq!(a.compare(&two), 0);

    // EXPECT_TRUE(BigInteger(RAPIDJSON_UINT64_C2(0xFFFFFFFF, 0xFFFFFFFF)) == kUint64Max);
    let via_u64_max = M::from_u64(u64::MAX);
    assert_eq!(via_u64_max.compare(&uint64_max), 0);

    // BigInteger b = kUint64Max;
    let mut b = M::from_decimal_literal("18446744073709551615");
    // b += 1u; EXPECT_TRUE(kTwo64 == b);
    b.add_u64(1);
    assert_eq!(b.compare(&two64), 0);

    // b += RAPIDJSON_UINT64_C2(0xFFFFFFFF, 0xFFFFFFFF);
    b.add_u64(u64::MAX);
    // EXPECT_TRUE(BIGINTEGER_LITERAL("36893488147419103231") == b);
    let expected = M::from_decimal_literal("36893488147419103231");
    assert_eq!(b.compare(&expected), 0);
}

// Mirror TEST(BigInteger, MultiplyUint64)
fn run_biginteger_multiply_uint64<M>()
where
    M: TestBigInteger,
{
    let zero = M::from_decimal_literal("0");
    let one = M::from_decimal_literal("1");

    // BigInteger a = kZero;
    let mut a = M::from_decimal_literal("0");
    // a *= static_cast<uint64_t>(0); EXPECT_TRUE(kZero == a);
    a.mul_u64(0);
    assert_eq!(a.compare(&zero), 0);
    // a *= static_cast<uint64_t>(123); EXPECT_TRUE(kZero == a);
    a.mul_u64(123);
    assert_eq!(a.compare(&zero), 0);

    // BigInteger b = kOne;
    let mut b = M::from_decimal_literal("1");
    // b *= 1; EXPECT_TRUE(kOne == b);
    b.mul_u64(1);
    assert_eq!(b.compare(&one), 0);
    // b *= 0; EXPECT_TRUE(kZero == b);
    b.mul_u64(0);
    assert_eq!(b.compare(&zero), 0);

    // BigInteger c(123);
    let mut c = M::from_u64(123);
    // c *= 456u; EXPECT_TRUE(BigInteger(123u * 456u) == c);
    c.mul_u64(456);
    let expected = M::from_u64(123u64 * 456u64);
    assert_eq!(c.compare(&expected), 0);

    // c *= UINT64_MAX; EXPECT_TRUE(BIGINTEGER_LITERAL("1034640981606221330982120") == c);
    c.mul_u64(u64::MAX);
    let expected2 = M::from_decimal_literal("1034640981606221330982120");
    assert_eq!(c.compare(&expected2), 0);

    // c *= UINT64_MAX; EXPECT_TRUE(BIGINTEGER_LITERAL("19085757395861596536664473018420572782123800") == c);
    c.mul_u64(u64::MAX);
    let expected3 = M::from_decimal_literal(
        "19085757395861596536664473018420572782123800",
    );
    assert_eq!(c.compare(&expected3), 0);
}

// Mirror TEST(BigInteger, MultiplyUint32)
fn run_biginteger_multiply_uint32<M>()
where
    M: TestBigInteger,
{
    let zero = M::from_decimal_literal("0");
    let one = M::from_decimal_literal("1");

    // BigInteger a = kZero;
    let mut a = M::from_decimal_literal("0");
    // a *= 0u; EXPECT_TRUE(kZero == a);
    a.mul_u32(0);
    assert_eq!(a.compare(&zero), 0);
    // a *= 123u; EXPECT_TRUE(kZero == a);
    a.mul_u32(123);
    assert_eq!(a.compare(&zero), 0);

    // BigInteger b = kOne;
    let mut b = M::from_decimal_literal("1");
    // b *= 1u; EXPECT_TRUE(kOne == b);
    b.mul_u32(1);
    assert_eq!(b.compare(&one), 0);
    // b *= 0u; EXPECT_TRUE(kZero == b);
    b.mul_u32(0);
    assert_eq!(b.compare(&zero), 0);

    // BigInteger c(123);
    let mut c = M::from_u64(123);
    // c *= 456u; EXPECT_TRUE(BigInteger(123u * 456u) == c);
    c.mul_u32(456);
    let expected = M::from_u64(123u64 * 456u64);
    assert_eq!(c.compare(&expected), 0);

    // c *= 0xFFFFFFFFu; EXPECT_TRUE(BIGINTEGER_LITERAL("240896125641960") == c);
    c.mul_u32(0xFFFF_FFFFu32);
    let expected2 = M::from_decimal_literal("240896125641960");
    assert_eq!(c.compare(&expected2), 0);

    // c *= 0xFFFFFFFFu; EXPECT_TRUE(BIGINTEGER_LITERAL("1034640981124429079698200") == c);
    c.mul_u32(0xFFFF_FFFFu32);
    let expected3 = M::from_decimal_literal("1034640981124429079698200");
    assert_eq!(c.compare(&expected3), 0);
}

// Mirror TEST(BigInteger, LeftShift)
fn run_biginteger_left_shift<M>()
where
    M: TestBigInteger,
{
    let zero = M::from_decimal_literal("0");

    // BigInteger a = kZero;
    let mut a = M::from_decimal_literal("0");
    // a <<= 1; EXPECT_TRUE(kZero == a);
    a.shl_bits(1);
    assert_eq!(a.compare(&zero), 0);
    // a <<= 64; EXPECT_TRUE(kZero == a);
    a.shl_bits(64);
    assert_eq!(a.compare(&zero), 0);

    // a = BigInteger(123);
    let mut a = M::from_u64(123);
    // a <<= 0; EXPECT_TRUE(BigInteger(123) == a);
    a.shl_bits(0);
    let expected123 = M::from_u64(123);
    assert_eq!(a.compare(&expected123), 0);

    // a <<= 1; EXPECT_TRUE(BigInteger(246) == a);
    a.shl_bits(1);
    let expected246 = M::from_u64(246);
    assert_eq!(a.compare(&expected246), 0);

    // a <<= 64; EXPECT_TRUE(BIGINTEGER_LITERAL("4537899042132549697536") == a);
    a.shl_bits(64);
    let expected_big = M::from_decimal_literal("4537899042132549697536");
    assert_eq!(a.compare(&expected_big), 0);

    // a <<= 99; EXPECT_TRUE(BIGINTEGER_LITERAL("2876235222267216943024851750785644982682875244576768") == a);
    a.shl_bits(99);
    let expected_bigger = M::from_decimal_literal(
        "2876235222267216943024851750785644982682875244576768",
    );
    assert_eq!(a.compare(&expected_bigger), 0);

    // a = 1; a <<= 64; a <<= 256; huge literal check
    let mut a = M::from_u64(1);
    a.shl_bits(64);
    a.shl_bits(256);
    let expected_huge = M::from_decimal_literal(
        "2135987035920910082395021706169552114602704522356652769947041607822219725780640550022962086936576",
    );
    assert_eq!(a.compare(&expected_huge), 0);
}

// Mirror TEST(BigInteger, Compare)
fn run_biginteger_compare<M>()
where
    M: TestBigInteger,
{
    let zero = M::from_decimal_literal("0");
    let one = M::from_decimal_literal("1");
    let uint64_max = M::from_decimal_literal("18446744073709551615");
    let two64 = M::from_decimal_literal("18446744073709551616");

    // EXPECT_EQ(0, kZero.Compare(kZero));
    assert_eq!(zero.compare(&zero), 0);

    // EXPECT_EQ(1, kOne.Compare(kZero));
    assert_eq!(one.compare(&zero), 1);

    // EXPECT_EQ(-1, kZero.Compare(kOne));
    assert_eq!(zero.compare(&one), -1);

    // EXPECT_EQ(0, kUint64Max.Compare(kUint64Max));
    assert_eq!(uint64_max.compare(&uint64_max), 0);

    // EXPECT_EQ(0, kTwo64.Compare(kTwo64));
    assert_eq!(two64.compare(&two64), 0);

    // EXPECT_EQ(-1, kUint64Max.Compare(kTwo64));
    assert_eq!(uint64_max.compare(&two64), -1);

    // EXPECT_EQ(1, kTwo64.Compare(kUint64Max));
    assert_eq!(two64.compare(&uint64_max), 1);
}

// L1: C++ backend entrypoints

#[test]
fn l1_cxx_biginteger_constructor() {
    run_biginteger_constructor::<CxxBigIntegerAdapter>();
}

#[test]
fn l1_cxx_biginteger_add_uint64() {
    run_biginteger_add_uint64::<CxxBigIntegerAdapter>();
}

#[test]
fn l1_cxx_biginteger_multiply_uint64() {
    run_biginteger_multiply_uint64::<CxxBigIntegerAdapter>();
}

#[test]
fn l1_cxx_biginteger_multiply_uint32() {
    run_biginteger_multiply_uint32::<CxxBigIntegerAdapter>();
}

#[test]
fn l1_cxx_biginteger_left_shift() {
    run_biginteger_left_shift::<CxxBigIntegerAdapter>();
}

#[test]
fn l1_cxx_biginteger_compare() {
    run_biginteger_compare::<CxxBigIntegerAdapter>();
}

// L2: Rust backend entrypoints

#[test]
fn l2_rust_biginteger_constructor() {
    run_biginteger_constructor::<RustBigIntegerAdapter>();
}

#[test]
fn l2_rust_biginteger_add_uint64() {
    run_biginteger_add_uint64::<RustBigIntegerAdapter>();
}

#[test]
fn l2_rust_biginteger_multiply_uint64() {
    run_biginteger_multiply_uint64::<RustBigIntegerAdapter>();
}

#[test]
fn l2_rust_biginteger_multiply_uint32() {
    run_biginteger_multiply_uint32::<RustBigIntegerAdapter>();
}

#[test]
fn l2_rust_biginteger_left_shift() {
    run_biginteger_left_shift::<RustBigIntegerAdapter>();
}

#[test]
fn l2_rust_biginteger_compare() {
    run_biginteger_compare::<RustBigIntegerAdapter>();
}
