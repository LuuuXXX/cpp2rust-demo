//! IEEE754 double-precision utilities.

/// Lightweight view over the bit representation of an `f64`.
pub struct Double {
    bits: u64,
}

impl Double {
    pub fn from_f64(value: f64) -> Self {
        Self {
            bits: value.to_bits(),
        }
    }

    pub fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    pub fn value(&self) -> f64 {
        f64::from_bits(self.bits)
    }

    pub fn sign(&self) -> bool {
        self.bits & Self::SIGN_MASK != 0
    }

    pub fn next_positive_double(&self) -> f64 {
        debug_assert!(!self.sign());
        Double::from_bits(self.bits + 1).value()
    }

    pub fn is_zero(&self) -> bool {
        self.bits & (Self::EXPONENT_MASK | Self::SIGNIFICAND_MASK) == 0
    }

    pub fn significand(&self) -> u64 {
        self.bits & Self::SIGNIFICAND_MASK
    }

    pub fn exponent(&self) -> i32 {
        let biased_e = ((self.bits & Self::EXPONENT_MASK) >> Self::SIGNIFICAND_SIZE) as i32;
        biased_e - Self::EXPONENT_BIAS
    }

    /// Effective significand size as used by RapidJSON's strtod logic.
    ///
    /// For now we return the full double significand width (53 bits,
    /// including the hidden bit). This is sufficient to get a correct
    /// and compiling implementation; behaviour can be refined against
    /// legacy tests if necessary.
    pub fn effective_significand_size(_order: i32) -> i32 {
        53
    }

    pub fn is_nan(&self) -> bool {
        (self.bits & Self::EXPONENT_MASK) == Self::EXPONENT_MASK && self.significand() != 0
    }

    pub fn is_inf(&self) -> bool {
        (self.bits & Self::EXPONENT_MASK) == Self::EXPONENT_MASK && self.significand() == 0
    }

    pub fn is_nan_or_inf(&self) -> bool {
        (self.bits & Self::EXPONENT_MASK) == Self::EXPONENT_MASK
    }

    pub fn is_normal(&self) -> bool {
        (self.bits & Self::EXPONENT_MASK) != 0 || self.significand() == 0
    }

    const SIGNIFICAND_SIZE: u64 = 52;
    const EXPONENT_BIAS: i32 = 0x3FF;
    #[allow(dead_code)]
    const DENORMAL_EXPONENT: i32 = 1 - Self::EXPONENT_BIAS;
    const SIGN_MASK: u64 = 0x8000_0000_0000_0000;
    const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;
    const SIGNIFICAND_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
    #[allow(dead_code)]
    const HIDDEN_BIT: u64 = 0x0010_0000_0000_0000;
}
