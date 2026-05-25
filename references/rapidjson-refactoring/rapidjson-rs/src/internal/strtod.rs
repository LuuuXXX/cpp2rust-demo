//! String to floating point conversion utilities.
//!
//! This module implements a port of RapidJSON's `internal::strtod`
//! logic in safe Rust. The high-level strategy is:
//!
//! - Parse the decimal string into a significand `d` and decimal
//!   exponent `p`.
//! - Use a fast-path (`StrtodFast`) when the value fits into the
//!   normal precision range.
//! - For more complex cases, use a combination of DiyFp and
//!   BigInteger to achieve high precision and correct rounding
//!   behaviour (within 0.5 ULP, round-to-even).

use crate::error::Error;
use crate::internal::biginteger::BigInteger;
use crate::internal::diyfp::{get_cached_power10, DiyFp};
use crate::internal::ieee754::Double;
use crate::internal::pow10::pow10;

/// Parses a string slice into an `f64` using a port of RapidJSON's
/// `StrtodFullPrecision` pipeline.
pub fn parse_f64(input: &str) -> Result<f64, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::Parse {
            message: "invalid float",
            offset: Some(0),
        });
    }

    // Basic grammar: optional sign, digits, optional '.', digits,
    // optional exponent.
    let (sign, digits, decimal_pos, exp) =
        match parse_decimal_components(trimmed) {
            Some(parts) => parts,
            None => {
                return Err(Error::Parse {
                    message: "invalid float",
                    offset: Some(0),
                })
            }
        };

    // Convert leading portion to a double, keeping count of exponent
    // adjustment.
    let (d, p) = parse_leading_digits(&digits, decimal_pos);

    let value = strtod_full_precision(d, p, &digits, digits.len(), decimal_pos, exp);
    Ok(if sign { -value } else { value })
}

#[allow(dead_code)]
fn parse_simple_decimal_int(input: &str) -> Option<i64> {
    if input.is_empty() {
        return None;
    }

    let mut chars = input.chars();
    let mut negative = false;
    let first = chars.next()?;

    let mut start_iter = chars;
    let mut first_digit = first;

    if first == '-' || first == '+' {
        negative = first == '-';
        first_digit = start_iter.next()?;
    }

    if !first_digit.is_ascii_digit() {
        return None;
    }

    let mut value: i64 = (first_digit as i64) - ('0' as i64);

    for ch in start_iter {
        if !ch.is_ascii_digit() {
            return None;
        }

        let digit = (ch as i64) - ('0' as i64);
        let (min, max) = if negative {
            (i64::MIN / 10, i64::MAX / 10)
        } else {
            (0, i64::MAX / 10)
        };

        if value < min || value > max {
            return None;
        }

        value = value * 10 + digit;
    }

    if negative {
        value = -value;
    }

    Some(value)
}

/// Fast path for parsing a subset of decimal floating point numbers.
///
/// Supported grammar (no surrounding whitespace):
///
/// ```text
/// [+-]? digits ('.' digits)? ([eE] [+-]? digits)?
/// ```
///
/// When the mantissa or exponent grows too large this function
/// returns `None` and the caller is expected to fall back to
/// `str::parse`.
#[allow(dead_code)]
fn parse_decimal_fast(input: &str) -> Option<f64> {
    let bytes = input.as_bytes();
    let len = bytes.len();

    if len == 0 {
        return None;
    }

    let mut i = 0;
    let mut negative = false;

    // Optional sign.
    if bytes[i] == b'+' || bytes[i] == b'-' {
        negative = bytes[i] == b'-';
        i += 1;
        if i == len {
            return None;
        }
    }

    // Integer part.
    let mut int: u64 = 0;
    let mut int_digits = 0u32;
    while i < len && bytes[i].is_ascii_digit() {
        let digit = (bytes[i] - b'0') as u64;
        // Guard against overflow in the fast path; fall back to
        // `str::parse` for extremely large integers.
        if int > (u64::MAX - digit) / 10 {
            return None;
        }
        int = int * 10 + digit;
        int_digits += 1;
        i += 1;
    }

    // Fractional part.
    let mut frac: u64 = 0;
    let mut frac_digits = 0i32;
    if i < len && bytes[i] == b'.' {
        i += 1;
        while i < len && bytes[i].is_ascii_digit() {
            let digit = (bytes[i] - b'0') as u64;
            if frac > (u64::MAX - digit) / 10 {
                return None;
            }
            frac = frac * 10 + digit;
            frac_digits += 1;
            i += 1;
        }
    }

    // Exponent part.
    let mut exp10: i32 = 0;
    if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i == len {
            return None;
        }

        let mut exp_negative = false;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            exp_negative = bytes[i] == b'-';
            i += 1;
            if i == len {
                return None;
            }
        }

        if !bytes[i].is_ascii_digit() {
            return None;
        }

        let mut exp: i32 = 0;
        while i < len && bytes[i].is_ascii_digit() {
            let digit = (bytes[i] - b'0') as i32;
            if exp > (i32::MAX - digit) / 10 {
                // Exponent too large for the fast path; delegate to
                // the fallback implementation.
                return None;
            }
            exp = exp * 10 + digit;
            i += 1;
        }

        exp10 = if exp_negative { -exp } else { exp };
    }

    // Reject trailing characters that are not part of the grammar.
    if i != len {
        return None;
    }

    // At least one digit must be present in either the integer or
    // fractional part.
    if int_digits == 0 && frac_digits == 0 {
        return None;
    }

    let mut value = int as f64;
    if frac_digits > 0 {
        // Scale the fractional digits by 10^-frac_digits.
        value += (frac as f64) * 10f64.powi(-frac_digits);
    }

    if exp10 != 0 {
        value *= 10f64.powi(exp10);
    }

    if negative {
        value = -value;
    }

    Some(value)
}

fn fast_path(significand: f64, exp: i32) -> f64 {
    if exp < -308 {
        0.0
    } else if exp >= 0 {
        significand * pow10(exp)
    } else {
        significand / pow10(-exp)
    }
}

#[allow(dead_code)]
fn strtod_normal_precision(mut d: f64, p: i32) -> f64 {
    if p < -308 {
        d = fast_path(d, -308);
        d = fast_path(d, p + 308);
    } else {
        d = fast_path(d, p);
    }
    d
}

fn strtod_fast(d: f64, p: i32) -> Option<f64> {
    // Fast path for decimal to double conversion when possible.
    if p > 22 && p < 22 + 16 {
        // Fast Path Cases In Disguise
        let scale = pow10(p - 22);
        return Some(d * scale);
    }

    if (-22..=22).contains(&p) && d <= 9_007_199_254_740_991.0 {
        Some(fast_path(d, p))
    } else {
        None
    }
}

fn min3<T: Ord + Copy>(a: T, b: T, c: T) -> T {
    let mut m = a;
    if m > b {
        m = b;
    }
    if m > c {
        m = c;
    }
    m
}

fn check_within_half_ulp(b: f64, d: &BigInteger, d_exp: i32) -> i32 {
    let db = Double::from_f64(b);
    let b_int = db.significand();
    let b_exp = db.exponent();
    let h_exp = b_exp - 1;

    let mut d_s_exp2 = 0;
    let mut d_s_exp5 = 0;
    let mut b_s_exp2 = 0;
    let mut b_s_exp5 = 0;
    let mut h_s_exp2 = 0;
    let mut h_s_exp5 = 0;

    // Adjust for decimal exponent
    if d_exp >= 0 {
        d_s_exp2 += d_exp;
        d_s_exp5 += d_exp;
    } else {
        b_s_exp2 -= d_exp;
        b_s_exp5 -= d_exp;
        h_s_exp2 -= d_exp;
        h_s_exp5 -= d_exp;
    }

    // Adjust for binary exponent
    if b_exp >= 0 {
        b_s_exp2 += b_exp;
    } else {
        d_s_exp2 -= b_exp;
        h_s_exp2 -= b_exp;
    }

    // Adjust for half ulp exponent
    if h_exp >= 0 {
        h_s_exp2 += h_exp;
    } else {
        d_s_exp2 -= h_exp;
        b_s_exp2 -= h_exp;
    }

    // Remove common power of two factor from all three scaled values
    let common_exp2 = min3(d_s_exp2, b_s_exp2, h_s_exp2);
    d_s_exp2 -= common_exp2;
    b_s_exp2 -= common_exp2;
    h_s_exp2 -= common_exp2;

    let mut d_s = d.clone();
    d_s = d_s.multiply_pow5(d_s_exp5 as u32).shl_bits(d_s_exp2 as u32);

    let mut b_s = BigInteger::from_u64(b_int);
    b_s = b_s.multiply_pow5(b_s_exp5 as u32).shl_bits(b_s_exp2 as u32);

    let mut h_s = BigInteger::from_u64(1);
    h_s = h_s.multiply_pow5(h_s_exp5 as u32).shl_bits(h_s_exp2 as u32);

    let (delta, _) = d_s.difference(&b_s);
    delta.cmp(&h_s) as i32
}

fn strtod_diyfp(decimals: &[u8], d_len: i32, mut d_exp: i32) -> Option<f64> {
    // Compute an approximation and see if it is within 1/2 ULP
    #[allow(unused_mut)]
    let mut d_len = d_len;
    let mut significand: u64 = 0;
    let mut i = 0;
    const LIMIT: u64 = 0x1999_9999_9999_9999; // 1844674407370955161

    while i < d_len {
        let ch = decimals[i as usize];
        let digit = (ch - b'0') as u64;
        if significand > LIMIT || (significand == LIMIT && digit >= 5) {
            break;
        }
        significand = significand * 10 + digit;
        i += 1;
    }

    if i < d_len && decimals[i as usize] >= b'5' {
        significand += 1;
    }

    let remaining = d_len - i;
    const K_ULP_SHIFT: i32 = 3;
    const K_ULP: i32 = 1 << K_ULP_SHIFT;
    let mut error: i64 = if remaining == 0 { 0 } else { (K_ULP / 2) as i64 };

    let mut v = DiyFp::new(significand, 0).normalize();
    error <<= -v.e;

    d_exp += remaining;

    let mut actual_exp = 0;
    let cached_power = get_cached_power10(d_exp, &mut actual_exp);
    if actual_exp != d_exp {
        // Precomputed powers of ten used to adjust the DiyFp when the
        // cached power exponent does not exactly match the requested one.
        const K_POW10_BITS: [(u64, i32); 7] = [
            (0xa000_0000_0000_0000, -60),
            (0xc800_0000_0000_0000, -57),
            (0xfa00_0000_0000_0000, -54),
            (0x9c40_0000_0000_0000, -50),
            (0xc350_0000_0000_0000, -47),
            (0xf424_0000_0000_0000, -44),
            (0x9896_8000_0000_0000, -40),
        ];
        let adjustment = d_exp - actual_exp;
        debug_assert!((1..8).contains(&adjustment));
        let (f_bits, e_bits) = K_POW10_BITS[(adjustment - 1) as usize];
        v = v.mul_internal(DiyFp::new(f_bits, e_bits));
        if d_len + adjustment > 19 {
            error += (K_ULP / 2) as i64;
        }
    }

    v = v.mul_internal(cached_power);

    error += K_ULP as i64 + if error == 0 { 0 } else { 1 };

    let old_exp = v.e;
    v = v.normalize();
    error <<= old_exp - v.e;

    let effective_size = Double::effective_significand_size(64 + v.e);
    let mut precision_size = 64 - effective_size;
    if precision_size + K_ULP_SHIFT >= 64 {
        let scale_exp = (precision_size + K_ULP_SHIFT) - 63;
        v.f >>= scale_exp;
        v.e += scale_exp;
        error = (error >> scale_exp) + 1 + K_ULP as i64;
        precision_size -= scale_exp;
    }

    let mut rounded = DiyFp::new(v.f >> precision_size, v.e + precision_size);
    let precision_bits = (v.f & ((1u64 << precision_size) - 1)) * K_ULP as u64;
    let half_way = (1u64 << (precision_size - 1)) * K_ULP as u64;
    if precision_bits >= half_way + error as u64 {
        rounded.f += 1;
        if rounded.f & (DiyFp::K_DP_HIDDEN_BIT << 1) != 0 {
            rounded.f >>= 1;
            rounded.e += 1;
        }
    }

    let result = rounded.to_f64();
    Some(result)
}

fn strtod_big_integer(approx: f64, decimals: &[u8], _d_len: i32, d_exp: i32) -> f64 {
    let d_int = BigInteger::from_decimal_str(core::str::from_utf8(decimals).unwrap())
        .unwrap_or_else(BigInteger::zero);
    let a = Double::from_f64(approx);
    let cmp = check_within_half_ulp(a.value(), &d_int, d_exp);
    if cmp < 0 {
        a.value()
    } else if cmp == 0 {
        if a.significand() & 1 != 0 {
            a.next_positive_double()
        } else {
            a.value()
        }
    } else {
        a.next_positive_double()
    }
}

fn strtod_full_precision(
    d: f64,
    p: i32,
    decimals: &[u8],
    length: usize,
    decimal_position: usize,
    exp: i32,
) -> f64 {
    debug_assert!(d >= 0.0);
    debug_assert!(length >= 1);

    if let Some(result) = strtod_fast(d, p) {
        return result;
    }

    let mut d_len = length as i32;
    let mut decimals = decimals;

    let d_exp_adjust = (length - decimal_position) as i32;
    let mut d_exp = exp - d_exp_adjust;

    // Trim leading zeros
    while d_len > 0 && decimals.first() == Some(&b'0') {
        d_len -= 1;
        decimals = &decimals[1..];
    }

    // Trim trailing zeros
    while d_len > 0 && decimals[(d_len - 1) as usize] == b'0' {
        d_len -= 1;
        d_exp += 1;
    }

    if d_len == 0 {
        return 0.0;
    }

    const K_MAX_DECIMAL_DIGIT: i32 = 767 + 1;
    if d_len > K_MAX_DECIMAL_DIGIT {
        d_exp += d_len - K_MAX_DECIMAL_DIGIT;
        d_len = K_MAX_DECIMAL_DIGIT;
    }

    if d_len + d_exp <= -324 {
        return 0.0;
    }

    if d_len + d_exp > 309 {
        return f64::INFINITY;
    }

    if let Some(result) = strtod_diyfp(&decimals[..d_len as usize], d_len, d_exp) {
        return result;
    }

    strtod_big_integer(d, &decimals[..d_len as usize], d_len, d_exp)
}

fn parse_decimal_components(
    input: &str,
) -> Option<(bool, Vec<u8>, usize, i32)> {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut sign = false;

    if bytes[i] == b'+' || bytes[i] == b'-' {
        sign = bytes[i] == b'-';
        i += 1;
    }

    let mut digits = Vec::with_capacity(bytes.len());
    let mut decimal_pos = None;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            if decimal_pos.is_some() {
                return None;
            }
            decimal_pos = Some(digits.len());
        } else if b == b'e' || b == b'E' {
            i += 1;
            break;
        } else if b.is_ascii_digit() {
            digits.push(b);
        } else {
            return None;
        }
        i += 1;
    }

    if digits.is_empty() {
        return None;
    }

    let decimal_pos = decimal_pos.unwrap_or(digits.len());

    let mut exp: i32 = 0;
    if i < bytes.len() {
        let mut exp_sign = 1i32;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            exp_sign = if bytes[i] == b'-' { -1 } else { 1 };
            i += 1;
        }
        if i == bytes.len() || !bytes[i].is_ascii_digit() {
            return None;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp = exp
                .saturating_mul(10)
                .saturating_add((bytes[i] - b'0') as i32);
            i += 1;
        }
        exp *= exp_sign;
    }

    Some((sign, digits, decimal_pos, exp))
}

fn parse_leading_digits(digits: &[u8], decimal_pos: usize) -> (f64, i32) {
    let mut d = 0.0f64;
    let mut i = 0;
    let limit = 16.min(decimal_pos);
    while i < limit {
        d = d * 10.0 + (digits[i] - b'0') as f64;
        i += 1;
    }

    let mut p = 0i32;
    if decimal_pos > limit {
        p = (decimal_pos - limit) as i32;
    }
    (d, p)
}

#[cfg(test)]
mod tests {
    use super::parse_f64;

    #[test]
    fn should_parse_valid_float_when_parse_f64() {
        let value = parse_f64("1.5").expect("parse should succeed");
        assert!(value > 1.4 && value < 1.6);
    }

    #[test]
    fn should_parse_simple_integer_fast_path() {
        let value = parse_f64("42").expect("parse should succeed");
        assert_eq!(value, 42.0);

        let value = parse_f64("-7").expect("parse should succeed");
        assert_eq!(value, -7.0);
    }

    #[test]
    fn should_parse_decimal_fraction_when_parse_f64() {
        let value = parse_f64("0.125").expect("parse should succeed");
        assert!((value - 0.125).abs() < 1e-12);
    }

    #[test]
    fn should_parse_scientific_notation_when_parse_f64() {
        let value = parse_f64("1e3").expect("parse should succeed");
        assert_eq!(value, 1000.0);

        let value = parse_f64("-2.5e2").expect("parse should succeed");
        assert!((value + 250.0).abs() < 1e-12);
    }

    #[test]
    fn should_fail_when_input_is_not_a_number() {
        let err = parse_f64("not-a-number").expect_err("parse should fail");
        let msg = err.to_string();
        assert!(msg.contains("parse error"));
    }

    #[test]
    fn should_fail_when_input_is_empty() {
        let err = parse_f64("").expect_err("parse should fail");
        let msg = err.to_string();
        assert!(msg.contains("parse error"));
    }
}
