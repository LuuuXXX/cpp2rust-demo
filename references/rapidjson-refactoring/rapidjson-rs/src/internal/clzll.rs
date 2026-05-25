//! Count leading zeros utilities.

/// Counts the number of leading zero bits in a 64-bit value.
///
/// In debug builds, passing `0` will trigger a debug assertion, as
/// the original C++ implementation treats this as undefined
/// behaviour. In release builds, `clzll(0)` returns `64`.
#[must_use]
pub fn clzll(value: u64) -> u32 {
    debug_assert!(value != 0, "clzll is undefined for 0");
    if value == 0 {
        64
    } else {
        value.leading_zeros()
    }
}

#[cfg(test)]
mod tests {
    use super::clzll;

    #[test]
    fn should_count_leading_zeros_when_clzll_for_basic_values() {
        assert_eq!(clzll(1), 63);
        assert_eq!(clzll(2), 62);
        assert_eq!(clzll(0x8000_0000_0000_0000), 0);
    }

    #[test]
    fn should_respect_contract_when_value_is_zero() {
        // In debug builds this should trigger the debug assertion,
        // while in release builds clzll(0) is defined as 64. We
        // assert these behaviours separately.
        if cfg!(debug_assertions) {
            let result = std::panic::catch_unwind(|| {
                let _ = clzll(0);
            });
            assert!(result.is_err());
        } else {
            assert_eq!(clzll(0), 64);
        }
    }
}
