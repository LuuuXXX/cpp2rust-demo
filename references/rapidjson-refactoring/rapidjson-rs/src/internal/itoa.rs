//! Integer to ASCII conversion utilities.

/// Two-digit lookup table, identical in contents to the C++
/// `GetDigitsLut()` used by RapidJSON. Each pair encodes the ASCII
/// bytes for a value in [0, 100).
const DIGITS_LUT: [u8; 200] = [
    b'0', b'0', b'0', b'1', b'0', b'2', b'0', b'3', b'0', b'4', b'0', b'5', b'0', b'6', b'0', b'7',
    b'0', b'8', b'0', b'9', b'1', b'0', b'1', b'1', b'1', b'2', b'1', b'3', b'1', b'4', b'1', b'5',
    b'1', b'6', b'1', b'7', b'1', b'8', b'1', b'9', b'2', b'0', b'2', b'1', b'2', b'2', b'2', b'3',
    b'2', b'4', b'2', b'5', b'2', b'6', b'2', b'7', b'2', b'8', b'2', b'9', b'3', b'0', b'3', b'1',
    b'3', b'2', b'3', b'3', b'3', b'4', b'3', b'5', b'3', b'6', b'3', b'7', b'3', b'8', b'3', b'9',
    b'4', b'0', b'4', b'1', b'4', b'2', b'4', b'3', b'4', b'4', b'4', b'5', b'4', b'6', b'4', b'7',
    b'4', b'8', b'4', b'9', b'5', b'0', b'5', b'1', b'5', b'2', b'5', b'3', b'5', b'4', b'5', b'5',
    b'5', b'6', b'5', b'7', b'5', b'8', b'5', b'9', b'6', b'0', b'6', b'1', b'6', b'2', b'6', b'3',
    b'6', b'4', b'6', b'5', b'6', b'6', b'6', b'7', b'6', b'8', b'6', b'9', b'7', b'0', b'7', b'1',
    b'7', b'2', b'7', b'3', b'7', b'4', b'7', b'5', b'7', b'6', b'7', b'7', b'7', b'8', b'7', b'9',
    b'8', b'0', b'8', b'1', b'8', b'2', b'8', b'3', b'8', b'4', b'8', b'5', b'8', b'6', b'8', b'7',
    b'8', b'8', b'8', b'9', b'9', b'0', b'9', b'1', b'9', b'2', b'9', b'3', b'9', b'4', b'9', b'5',
    b'9', b'6', b'9', b'7', b'9', b'8', b'9', b'9',
];

/// Converts an unsigned 64-bit integer to its decimal string
/// representation. This is a convenience wrapper around `write_u64`
/// for use in tests and debugging.
#[must_use]
pub fn u64_to_string(value: u64) -> String {
    let mut buf = [0u8; 32];
    let len = write_u64(value, &mut buf);
    String::from_utf8(buf[..len].to_vec()).expect("itoa produced valid UTF-8")
}

/// Writes the decimal representation of an unsigned 64-bit integer
/// into the provided buffer and returns the number of bytes written.
///
/// The buffer must be large enough to hold the full representation
/// (最多 20 位十进制数字)。实现采用与 RapidJSON `u64toa` 类似的
/// LUT 驱动算法，无堆分配，不使用格式化宏。
#[must_use]
pub fn write_u64(mut value: u64, buf: &mut [u8]) -> usize {
    if buf.is_empty() {
        return 0;
    }

    let lut = &DIGITS_LUT;
    let mut pos = 0usize;

    const TEN8: u64 = 100_000_000;
    const TEN9: u64 = TEN8 * 10;
    const TEN10: u64 = TEN8 * 100;
    const TEN11: u64 = TEN8 * 1000;
    const TEN12: u64 = TEN8 * 10_000;
    const TEN13: u64 = TEN8 * 100_000;
    const TEN14: u64 = TEN8 * 1_000_000;
    const TEN15: u64 = TEN8 * 10_000_000;
    const TEN16: u64 = TEN8 * TEN8;

    if value < TEN8 {
        let v = value as u32;
        if v < 10_000 {
            let d1 = (v / 100) << 1;
            let d2 = (v % 100) << 1;

            if v >= 1000 {
                buf[pos] = lut[d1 as usize];
                pos += 1;
            }
            if v >= 100 {
                buf[pos] = lut[d1 as usize + 1];
                pos += 1;
            }
            if v >= 10 {
                buf[pos] = lut[d2 as usize];
                pos += 1;
            }
            buf[pos] = lut[d2 as usize + 1];
            pos += 1;
        } else {
            // value = bbbbcccc
            let b = v / 10_000;
            let c = v % 10_000;

            let d1 = (b / 100) << 1;
            let d2 = (b % 100) << 1;
            let d3 = (c / 100) << 1;
            let d4 = (c % 100) << 1;

            if value >= 10_000_000 {
                buf[pos] = lut[d1 as usize];
                pos += 1;
            }
            if value >= 1_000_000 {
                buf[pos] = lut[d1 as usize + 1];
                pos += 1;
            }
            if value >= 100_000 {
                buf[pos] = lut[d2 as usize];
                pos += 1;
            }
            buf[pos] = lut[d2 as usize + 1];
            pos += 1;

            buf[pos] = lut[d3 as usize];
            buf[pos + 1] = lut[d3 as usize + 1];
            buf[pos + 2] = lut[d4 as usize];
            buf[pos + 3] = lut[d4 as usize + 1];
            pos += 4;
        }
    } else if value < TEN16 {
        let v0 = (value / TEN8) as u32;
        let v1 = (value % TEN8) as u32;

        let b0 = v0 / 10_000;
        let c0 = v0 % 10_000;
        let d1 = (b0 / 100) << 1;
        let d2 = (b0 % 100) << 1;
        let d3 = (c0 / 100) << 1;
        let d4 = (c0 % 100) << 1;

        let b1 = v1 / 10_000;
        let c1 = v1 % 10_000;
        let d5 = (b1 / 100) << 1;
        let d6 = (b1 % 100) << 1;
        let d7 = (c1 / 100) << 1;
        let d8 = (c1 % 100) << 1;

        if value >= TEN15 {
            buf[pos] = lut[d1 as usize];
            pos += 1;
        }
        if value >= TEN14 {
            buf[pos] = lut[d1 as usize + 1];
            pos += 1;
        }
        if value >= TEN13 {
            buf[pos] = lut[d2 as usize];
            pos += 1;
        }
        if value >= TEN12 {
            buf[pos] = lut[d2 as usize + 1];
            pos += 1;
        }
        if value >= TEN11 {
            buf[pos] = lut[d3 as usize];
            pos += 1;
        }
        if value >= TEN10 {
            buf[pos] = lut[d3 as usize + 1];
            pos += 1;
        }
        if value >= TEN9 {
            buf[pos] = lut[d4 as usize];
            pos += 1;
        }

        buf[pos] = lut[d4 as usize + 1];
        buf[pos + 1] = lut[d5 as usize];
        buf[pos + 2] = lut[d5 as usize + 1];
        buf[pos + 3] = lut[d6 as usize];
        buf[pos + 4] = lut[d6 as usize + 1];
        buf[pos + 5] = lut[d7 as usize];
        buf[pos + 6] = lut[d7 as usize + 1];
        buf[pos + 7] = lut[d8 as usize];
        buf[pos + 8] = lut[d8 as usize + 1];
        pos += 9;
    } else {
        let a = (value / TEN16) as u32; // 1 to 1844
        value %= TEN16;

        if a < 10 {
            buf[pos] = b'0' + (a as u8);
            pos += 1;
        } else if a < 100 {
            let i = (a << 1) as usize;
            buf[pos] = lut[i];
            buf[pos + 1] = lut[i + 1];
            pos += 2;
        } else if a < 1000 {
            buf[pos] = b'0' + (a / 100) as u8;
            pos += 1;
            let i = ((a % 100) << 1) as usize;
            buf[pos] = lut[i];
            buf[pos + 1] = lut[i + 1];
            pos += 2;
        } else {
            let i = (a / 100) << 1;
            let j = (a % 100) << 1;
            buf[pos] = lut[i as usize];
            buf[pos + 1] = lut[i as usize + 1];
            buf[pos + 2] = lut[j as usize];
            buf[pos + 3] = lut[j as usize + 1];
            pos += 4;
        }

        let v0 = (value / TEN8) as u32;
        let v1 = (value % TEN8) as u32;

        let b0 = v0 / 10_000;
        let c0 = v0 % 10_000;
        let d1 = (b0 / 100) << 1;
        let d2 = (b0 % 100) << 1;
        let d3 = (c0 / 100) << 1;
        let d4 = (c0 % 100) << 1;

        let b1 = v1 / 10_000;
        let c1 = v1 % 10_000;
        let d5 = (b1 / 100) << 1;
        let d6 = (b1 % 100) << 1;
        let d7 = (c1 / 100) << 1;
        let d8 = (c1 % 100) << 1;

        buf[pos] = lut[d1 as usize];
        buf[pos + 1] = lut[d1 as usize + 1];
        buf[pos + 2] = lut[d2 as usize];
        buf[pos + 3] = lut[d2 as usize + 1];
        buf[pos + 4] = lut[d3 as usize];
        buf[pos + 5] = lut[d3 as usize + 1];
        buf[pos + 6] = lut[d4 as usize];
        buf[pos + 7] = lut[d4 as usize + 1];
        buf[pos + 8] = lut[d5 as usize];
        buf[pos + 9] = lut[d5 as usize + 1];
        buf[pos + 10] = lut[d6 as usize];
        buf[pos + 11] = lut[d6 as usize + 1];
        buf[pos + 12] = lut[d7 as usize];
        buf[pos + 13] = lut[d7 as usize + 1];
        buf[pos + 14] = lut[d8 as usize];
        buf[pos + 15] = lut[d8 as usize + 1];
        pos += 16;
    }

    pos
}

/// Writes the decimal representation of an unsigned 32-bit integer
/// into the provided buffer and returns the number of bytes written.
#[must_use]
pub fn write_u32(value: u32, buf: &mut [u8]) -> usize {
    write_u64(value as u64, buf)
}

/// Writes the decimal representation of a signed 64-bit integer into
/// the provided buffer and returns the number of bytes written.
#[must_use]
    pub fn write_i64(value: i64, buf: &mut [u8]) -> usize {
    if buf.is_empty() {
        return 0;
    }

        let n = value;
        let mut pos = 0;

    if n < 0 {
        buf[pos] = b'-';
        pos += 1;
        let abs = n as i128;
        let abs_u = (-abs) as u64;
        return pos + write_u64(abs_u, &mut buf[pos..]);
    }

    pos + write_u64(n as u64, &mut buf[pos..])
}

/// Writes the decimal representation of a signed 32-bit integer into
/// the provided buffer and returns the number of bytes written.
#[must_use]
pub fn write_i32(value: i32, buf: &mut [u8]) -> usize {
    write_i64(value as i64, buf)
}

#[cfg(test)]
mod tests {
    use super::{u64_to_string, write_i32, write_i64, write_u32, write_u64};

    #[test]
    fn should_convert_small_integer_when_u64_to_string() {
        assert_eq!(u64_to_string(42), "42");
    }

    #[test]
    fn should_write_decimal_representation_when_write_u64() {
        let mut buf = [0u8; 32];
        let len = write_u64(42, &mut buf);
        assert_eq!(&buf[..len], b"42");

        let len = write_u64(0, &mut buf);
        assert_eq!(&buf[..len], b"0");

        let len = write_u64(9_223_372_036_854_775_807, &mut buf);
        assert_eq!(&buf[..len], b"9223372036854775807");
    }

    #[test]
    fn should_write_u32_when_write_u32() {
        let mut buf = [0u8; 16];
        let len = write_u32(12345, &mut buf);
        assert_eq!(&buf[..len], b"12345");
    }

    #[test]
    fn should_write_i64_with_sign_when_write_i64() {
        let mut buf = [0u8; 32];
        let len = write_i64(-42, &mut buf);
        assert_eq!(&buf[..len], b"-42");

        let len = write_i64(42, &mut buf);
        assert_eq!(&buf[..len], b"42");
    }

    #[test]
    fn should_write_i32_with_sign_when_write_i32() {
        let mut buf = [0u8; 16];
        let len = write_i32(-123, &mut buf);
        assert_eq!(&buf[..len], b"-123");

        let len = write_i32(123, &mut buf);
        assert_eq!(&buf[..len], b"123");
    }
}
