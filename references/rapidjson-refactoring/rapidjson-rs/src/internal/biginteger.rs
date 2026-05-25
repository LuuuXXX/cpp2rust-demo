//! Internal big integer utilities.

/// Non-negative big integer representation used by internal
/// algorithms. The type is intentionally minimal and focuses on
/// addition, subtraction and small multiplications needed by numeric
/// conversions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInteger {
    // Least-significant limb first, base 2^32.
    limbs: Vec<u32>,
}

impl BigInteger {
    /// Creates a zero value big integer.
    #[must_use]
    pub fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    /// Creates a big integer from an unsigned 64-bit value.
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        if value == 0 {
            return Self::zero();
        }

        let lo = value as u32;
        let hi = (value >> 32) as u32;
        let mut limbs = Vec::with_capacity(2);
        limbs.push(lo);
        if hi != 0 {
            limbs.push(hi);
        }
        Self { limbs }
    }

    // Internal helpers reserved for future big integer operations.
    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.limbs.len()
    }

    #[allow(dead_code)]
    fn limb(&self, index: usize) -> Option<u32> {
        self.limbs.get(index).copied()
    }

    /// Returns true if the value is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    /// Adds two non-negative big integers.
    #[must_use]
    pub fn add(&self, rhs: &Self) -> Self {
        let max_len = core::cmp::max(self.limbs.len(), rhs.limbs.len());
        let mut result = Vec::with_capacity(max_len + 1);
        let mut carry: u64 = 0;

        for i in 0..max_len {
            let a = self.limbs.get(i).copied().unwrap_or(0) as u64;
            let b = rhs.limbs.get(i).copied().unwrap_or(0) as u64;
            let sum = a + b + carry;
            result.push(sum as u32);
            carry = sum >> 32;
        }

        if carry != 0 {
            result.push(carry as u32);
        }

        Self { limbs: result }
    }

    /// Subtracts `rhs` from `self`. Returns `None` when `rhs` is
    /// larger than `self`, since negative values are not represented.
    #[must_use]
    pub fn sub(&self, rhs: &Self) -> Option<Self> {
        if self < rhs {
            return None;
        }

        let mut result = Vec::with_capacity(self.limbs.len());
        let mut borrow: u64 = 0;

        for i in 0..self.limbs.len() {
            let a = self.limbs[i] as u64;
            let b = rhs.limbs.get(i).copied().unwrap_or(0) as u64;
            let sub = a.wrapping_sub(b + borrow);
            borrow = if a < b + borrow { 1 } else { 0 };
            result.push(sub as u32);
        }

        debug_assert_eq!(borrow, 0);

        // Remove leading zero limbs.
        while result.last().copied() == Some(0) {
            result.pop();
        }

        Some(Self { limbs: result })
    }

    /// Multiplies a big integer by a small 32-bit value.
    #[must_use]
    pub fn mul_u32(&self, rhs: u32) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::zero();
        }

        let mut result = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry: u64 = 0;

        for limb in &self.limbs {
            let prod = (*limb as u64) * (rhs as u64) + carry;
            result.push(prod as u32);
            carry = prod >> 32;
        }

        if carry != 0 {
            result.push(carry as u32);
        }

        Self { limbs: result }
    }

    /// Adds a 64-bit unsigned integer to this big integer and returns
    /// the result.
    #[must_use]
    pub fn add_u64(&self, rhs: u64) -> Self {
        // Reuse the existing addition implementation by converting the
        // small integer into a BigInteger first. This avoids
        // duplicating carry-handling logic and keeps the API simple.
        let other = BigInteger::from_u64(rhs);
        self.add(&other)
    }

    /// Multiplies this big integer by a 64-bit unsigned integer and
    /// returns the result.
    #[must_use]
    pub fn mul_u64(&self, rhs: u64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::zero();
        }

        if rhs <= u32::MAX as u64 {
            return self.mul_u32(rhs as u32);
        }

        // General case: treat the multiplier as a full 64-bit value
        // and perform limb-wise multiplication in base 2^32 using a
        // 128-bit accumulator to avoid overflow.
        let mut result = Vec::with_capacity(self.limbs.len() + 2);
        let mut carry: u128 = 0;

        for &limb in &self.limbs {
            let prod = (limb as u128) * (rhs as u128) + carry;
            result.push(prod as u32);
            carry = prod >> 32;
        }

        // Flush remaining carry limbs.
        while carry != 0 {
            result.push((carry & 0xFFFF_FFFF) as u32);
            carry >>= 32;
        }

        Self { limbs: result }
    }

    /// Returns a new big integer equal to `self << shift` (left shift
    /// by the given number of bits).
    #[must_use]
    pub fn shl_bits(&self, shift: u32) -> Self {
        if self.is_zero() || shift == 0 {
            return self.clone();
        }

        let word_shift = (shift / 32) as usize;
        let bit_shift = shift % 32;

        let mut result = vec![0u32; self.limbs.len() + word_shift + 1];
        let mut carry: u64 = 0;

        for (i, &limb) in self.limbs.iter().enumerate() {
            let value = ((limb as u64) << bit_shift) | carry;
            result[i + word_shift] = value as u32;
            carry = value >> 32;
        }

        if carry != 0 {
            result[self.limbs.len() + word_shift] = carry as u32;
        }

        // Trim potential leading zero limbs.
        while result.last().copied() == Some(0) {
            result.pop();
        }

        Self { limbs: result }
    }

    /// Multiplies this big integer by 5^exp and returns the result.
    ///
    /// This is a straightforward implementation that repeatedly
    /// multiplies by 5. It prioritizes correctness and simplicity; if
    /// hotspots are identified we can adopt a more sophisticated
    /// exponentiation-by-squaring strategy later.
    #[must_use]
    pub fn multiply_pow5(&self, exp: u32) -> Self {
        if exp == 0 || self.is_zero() {
            return self.clone();
        }

        let mut result = self.clone();
        for _ in 0..exp {
            result = result.mul_u32(5);
        }
        result
    }

    /// Creates a big integer from a decimal ASCII string.
    ///
    /// Returns `None` if the input is empty or contains non-digit
    /// characters.
    #[must_use]
    pub fn from_decimal_str(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        let mut value = BigInteger::zero();

        for &b in s.as_bytes() {
            if !b.is_ascii_digit() {
                return None;
            }
            let digit = (b - b'0') as u64;
            // value = value * 10 + digit
            value = value.mul_u32(10).add_u64(digit);
        }

        Some(value)
    }

    /// Computes the absolute difference between `self` and `rhs`.
    ///
    /// Returns a tuple `(result, is_negative)` where `result` is the
    /// non-negative absolute difference and `is_negative` is `true`
    /// when `self < rhs`.
    #[must_use]
    pub fn difference(&self, rhs: &Self) -> (Self, bool) {
        use core::cmp::Ordering;

        match self.cmp(rhs) {
            Ordering::Equal => (BigInteger::zero(), false),
            Ordering::Greater => {
                let diff = self
                    .sub(rhs)
                    .expect("difference: self >= rhs ensures subtraction succeeds");
                (diff, false)
            }
            Ordering::Less => {
                let diff = rhs
                    .sub(self)
                    .expect("difference: rhs >= self ensures subtraction succeeds");
                (diff, true)
            }
        }
    }
}

impl Ord for BigInteger {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use core::cmp::Ordering;

        let len_a = self.limbs.len();
        let len_b = other.limbs.len();
        if len_a != len_b {
            return len_a.cmp(&len_b);
        }

        for i in (0..len_a).rev() {
            let a = self.limbs[i];
            let b = other.limbs[i];
            if a != b {
                return a.cmp(&b);
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for BigInteger {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
impl BigInteger {
    // Test-only helper to convert the internal representation back to
    // a primitive value for assertions.
    fn to_u128(&self) -> u128 {
        let mut value: u128 = 0;
        for (i, limb) in self.limbs.iter().enumerate() {
            let part = (*limb as u128) << (32 * i as u32);
            value |= part;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::BigInteger;

    #[test]
    fn should_create_zero_when_big_integer_zero() {
        let value = BigInteger::zero();
        let other = BigInteger::zero();
        assert_eq!(value, other);
    }

    #[test]
    fn should_roundtrip_small_values_when_from_u64() {
        let values = [0_u64, 1, 10, u32::MAX as u64, u32::MAX as u64 + 1];
        for &v in &values {
            let big = BigInteger::from_u64(v);
            assert_eq!(big.to_u128(), v as u128);
        }
    }

    #[test]
    fn should_add_when_big_integer_add() {
        let a = BigInteger::from_u64(1_000_000_000);
        let b = BigInteger::from_u64(2_000_000_000);
        let sum = a.add(&b);
        assert_eq!(sum.to_u128(), 3_000_000_000u128);
    }

    #[test]
    fn should_subtract_when_left_not_smaller() {
        let a = BigInteger::from_u64(3_000_000_000);
        let b = BigInteger::from_u64(1_000_000_000);
        let diff = a.sub(&b).expect("a should be >= b");
        assert_eq!(diff.to_u128(), 2_000_000_000u128);
    }

    #[test]
    fn should_return_none_when_subtract_larger_rhs() {
        let a = BigInteger::from_u64(1);
        let b = BigInteger::from_u64(2);
        assert!(a.sub(&b).is_none());
    }

    #[test]
    fn should_multiply_by_small_integer_when_mul_u32() {
        let a = BigInteger::from_u64(123_456_789);
        let prod = a.mul_u32(10);
        assert_eq!(prod.to_u128(), 1_234_567_890u128);
    }

    #[test]
    fn should_add_u64_when_big_integer_add_u64() {
        let a = BigInteger::from_u64(1_000_000_000);
        let sum = a.add_u64(5);
        assert_eq!(sum.to_u128(), 1_000_000_005u128);
    }

    #[test]
    fn should_multiply_by_u64_when_big_integer_mul_u64() {
        let a = BigInteger::from_u64(1_000_000_000);
        let prod = a.mul_u64(1_000_000_000);
        assert_eq!(prod.to_u128(), 1_000_000_000_000_000_000u128);
    }

    #[test]
    fn should_shift_left_when_big_integer_shl_bits() {
        let a = BigInteger::from_u64(1);
        let shifted = a.shl_bits(33);
        assert_eq!(shifted.to_u128(), 1u128 << 33);
    }

    #[test]
    fn should_multiply_by_power_of_five_when_multiply_pow5() {
        let one = BigInteger::from_u64(1);
        let v = one.multiply_pow5(1);
        assert_eq!(v.to_u128(), 5u128);

        let v = one.multiply_pow5(3);
        assert_eq!(v.to_u128(), 125u128);
    }

    #[test]
    fn should_parse_decimal_string_when_from_decimal_str() {
        let v = BigInteger::from_decimal_str("0").expect("parse 0");
        assert!(v.is_zero());

        let v = BigInteger::from_decimal_str("123456789").expect("parse number");
        assert_eq!(v.to_u128(), 123_456_789u128);

        assert!(BigInteger::from_decimal_str("").is_none());
        assert!(BigInteger::from_decimal_str("abc").is_none());
        assert!(BigInteger::from_decimal_str("12a3").is_none());
    }

    #[test]
    fn should_compute_difference_when_big_integer_difference() {
        let a = BigInteger::from_u64(1_000);
        let b = BigInteger::from_u64(600);

        let (diff, is_negative) = a.difference(&b);
        assert_eq!(diff.to_u128(), 400u128);
        assert!(!is_negative);

        let (diff, is_negative) = b.difference(&a);
        assert_eq!(diff.to_u128(), 400u128);
        assert!(is_negative);
    }
}
