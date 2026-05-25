//! Meta-programming style utilities.

/// Returns whether a value is within the inclusive range [min, max].
#[must_use]
pub fn in_range<T>(value: T, min: T, max: T) -> bool
where
    T: PartialOrd,
{
    value >= min && value <= max
}

/// Clamps `value` into the inclusive range [min, max].
#[must_use]
pub fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd + Copy,
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{clamp, in_range};

    #[test]
    fn should_be_in_range_when_value_between_bounds() {
        assert!(in_range(2, 1, 3));
        assert!(!in_range(0, 1, 3));
    }

    #[test]
    fn should_clamp_value_into_range() {
        assert_eq!(clamp(0, 1, 3), 1);
        assert_eq!(clamp(2, 1, 3), 2);
        assert_eq!(clamp(4, 1, 3), 3);
    }
}
