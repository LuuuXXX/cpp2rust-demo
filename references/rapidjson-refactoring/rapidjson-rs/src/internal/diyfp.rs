//! DIY floating point representation used by dtoa.

use crate::internal::clzll::clzll;

/// A floating point number with a 64-bit significand and a 32-bit exponent.
#[derive(Clone, Copy, Debug)]
pub struct DiyFp {
    pub f: u64,
    pub e: i32,
}

impl DiyFp {
    pub fn new(f: u64, e: i32) -> Self {
        Self { f, e }
    }

    pub fn minus(self, rhs: DiyFp) -> DiyFp {
        DiyFp {
            f: self.f - rhs.f,
            e: self.e,
        }
    }

    // Internal multiplication helper; we do not implement the standard
    // `Mul` trait because DiyFp is only used inside dtoa.
    pub fn mul_internal(self, rhs: DiyFp) -> DiyFp {
        // 64x64 -> 128 bit multiplication with rounding.
        let p = (self.f as u128) * (rhs.f as u128);
        let mut h = (p >> 64) as u64;
        let l = p as u64;
        if l & (1u64 << 63) != 0 {
            h = h.wrapping_add(1);
        }
        DiyFp {
            f: h,
            e: self.e + rhs.e + 64,
        }
    }

    pub fn normalize(self) -> DiyFp {
        let s = clzll(self.f) as i32;
        DiyFp {
            f: self.f << s,
            e: self.e - s,
        }
    }

    pub fn normalize_boundary(self) -> DiyFp {
        let mut res = self;
        while (res.f & (Self::K_DP_HIDDEN_BIT << 1)) == 0 {
            res.f <<= 1;
            res.e -= 1;
        }
        res.f <<= Self::K_DIY_SIGNIFICAND_SIZE - Self::K_DP_SIGNIFICAND_SIZE - 2;
        res.e -= Self::K_DIY_SIGNIFICAND_SIZE as i32 - Self::K_DP_SIGNIFICAND_SIZE as i32 - 2;
        res
    }

    pub fn normalized_boundaries(self) -> (DiyFp, DiyFp) {
        let pl = DiyFp::new((self.f << 1) + 1, self.e - 1).normalize_boundary();
        let mut mi = if self.f == Self::K_DP_HIDDEN_BIT {
            DiyFp::new((self.f << 2) - 1, self.e - 2)
        } else {
            DiyFp::new((self.f << 1) - 1, self.e - 1)
        };
        mi.f <<= (mi.e - pl.e) as u64;
        mi.e = pl.e;
        (mi, pl)
    }

    pub fn from_f64(value: f64) -> Self {
        let bits = value.to_bits();
        let biased_e = ((bits & Self::K_DP_EXPONENT_MASK) >> Self::K_DP_SIGNIFICAND_SIZE) as i32;
        let significand = bits & Self::K_DP_SIGNIFICAND_MASK;
        if biased_e != 0 {
            DiyFp {
                f: significand + Self::K_DP_HIDDEN_BIT,
                e: biased_e - Self::K_DP_EXPONENT_BIAS,
            }
        } else {
            DiyFp {
                f: significand,
                e: Self::K_DP_MIN_EXPONENT + 1,
            }
        }
    }

    /// Converts this DiyFp back into an `f64`, following the same
    /// semantics as the original C++ `ToDouble` implementation.
    #[must_use]
    pub fn to_f64(self) -> f64 {
        debug_assert!(self.f <= Self::K_DP_HIDDEN_BIT + Self::K_DP_SIGNIFICAND_MASK);

        if self.e < Self::K_DP_DENORMAL_EXPONENT {
            // Underflow.
            return 0.0;
        }

        if self.e >= Self::K_DP_MAX_EXPONENT {
            // Overflow.
            return f64::INFINITY;
        }

        let be = if self.e == Self::K_DP_DENORMAL_EXPONENT && (self.f & Self::K_DP_HIDDEN_BIT) == 0
        {
            0_u64
        } else {
            (self.e + Self::K_DP_EXPONENT_BIAS) as u64
        };

        let bits = (self.f & Self::K_DP_SIGNIFICAND_MASK) | (be << Self::K_DP_SIGNIFICAND_SIZE);
        f64::from_bits(bits)
    }

    const K_DIY_SIGNIFICAND_SIZE: u64 = 64;
    const K_DP_SIGNIFICAND_SIZE: u64 = 52;
    const K_DP_EXPONENT_BIAS: i32 = 0x3FF + Self::K_DP_SIGNIFICAND_SIZE as i32;
    const K_DP_MAX_EXPONENT: i32 = 0x7FF - Self::K_DP_EXPONENT_BIAS;
    const K_DP_MIN_EXPONENT: i32 = -Self::K_DP_EXPONENT_BIAS;
    const K_DP_DENORMAL_EXPONENT: i32 = -Self::K_DP_EXPONENT_BIAS + 1;
    const K_DP_EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;
    const K_DP_SIGNIFICAND_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
    pub const K_DP_HIDDEN_BIT: u64 = 0x0010_0000_0000_0000;
}

pub fn get_cached_power(e: i32, k: &mut i32) -> DiyFp {
    // k = ceil((-61 - e) * log10(2)) + 347; but implemented as in C++.
    let dk = (-61 - e) as f64 * 0.301_029_995_663_981_14_f64 + 347.0;
    let mut kk = dk as i32;
    if dk - kk as f64 > 0.0 {
        kk += 1;
    }

    let index = ((kk >> 3) + 1) as usize;
    *k = -(-348 + (index as i32) * 8);
    get_cached_power_by_index(index)
}

/// Returns a cached power-of-ten DiyFp for the given decimal exponent
/// `exp`, along with the actual exponent used via `out_exp`.
#[must_use]
pub fn get_cached_power10(exp: i32, out_exp: &mut i32) -> DiyFp {
    debug_assert!(exp >= -348);
    let index = ((exp + 348) / 8) as usize;
    *out_exp = -348 + (index as i32) * 8;
    get_cached_power_by_index(index)
}

fn get_cached_power_by_index(index: usize) -> DiyFp {
    const K_CACHED_POWERS_F: [u64; 87] = [
        0xfa8f_d5a0_081c_0288,
        0xbaae_e17f_a23e_bf76,
        0x8b16_fb20_3055_ac76,
        0xcf42_894a_5dce_35ea,
        0x9a6b_b0aa_5565_3b2d,
        0xe61a_cf03_3d1a_45df,
        0xab70_fe17_c79a_c6ca,
        0xff77_b1fc_bebc_dc4f,
        0xbe56_91ef_416b_d60c,
        0x8dd0_1fad_907f_fc3c,
        0xd351_5c28_3155_9a83,
        0x9d71_ac8f_ada6_c9b5,
        0xea9c_2277_23ee_8bcb,
        0xaecc_4991_4078_536d,
        0x823c_1279_5db6_ce57,
        0xc210_9436_4dfb_5637,
        0x9096_ea6f_3848_984f,
        0xd774_85cb_2582_3ac7,
        0xa086_cfcd_97bf_97f4,
        0xef34_0a98_172a_ace5,
        0xb238_67fb_2a35_b28e,
        0x84c8_d4df_d2c6_3f3b,
        0xc5dd_4427_1ad3_cdba,
        0x936b_9fce_bb25_c996,
        0xdbac_6c24_7d62_a584,
        0xa3ab_6658_0d5f_daf6,
        0xf3e2_f893_dec3_f126,
        0xb5b5_ada8_aaff_80b8,
        0x8762_5f05_6c7c_4a8b,
        0xc9bc_ff60_34c1_3053,
        0x964e_858c_91ba_2655,
        0xdff9_7724_7029_7ebd,
        0xa6df_bd9f_b8e5_b88f,
        0xf8a9_5fcf_8874_7d94,
        0xb944_7093_8fa8_9bcf,
        0x8a08_f0f8_bf0f_156b,
        0xcdb0_2555_6531_31b6,
        0x993f_e2c6_d07b_7fac,
        0xe45c_10c4_2a2b_3b06,
        0xaa24_2499_6973_92d3,
        0xfd87_b5f2_8300_ca0e,
        0xbce5_0864_9211_1aeb,
        0x8cbc_cc09_6f50_88cc,
        0xd1b7_1758_e219_652c,
        0x9c40_0000_0000_0000,
        0xe8d4_a510_0000_0000,
        0xad78_ebc5_ac62_0000,
        0x813f_3978_f894_0984,
        0xc097_ce7b_c907_15b3,
        0x8f7e_32ce_7bea_5c70,
        0xd5d2_38a4_abe9_8068,
        0x9f4f_2726_179a_2245,
        0xed63_a231_d4c4_fb27,
        0xb0de_6538_8cc8_ada8,
        0x83c7_088e_1aab_65db,
        0xc45d_1df9_4271_1d9a,
        0x924d_692c_a61b_e758,
        0xda01_ee64_1a70_8dea,
        0xa26d_a399_9aef_774a,
        0xf209_787b_b47d_6b85,
        0xb454_e4a1_79dd_1877,
        0x865b_8692_5b9b_c5c2,
        0xc835_53c5_c896_5d3d,
        0x952a_b45c_fa97_a0b3,
        0xde46_9fbd_99a0_5fe3,
        0xa59b_c234_db39_8c25,
        0xf6c6_9a72_a398_9f5c,
        0xb7dc_bf53_54e9_bece,
        0x88fc_f317_f222_41e2,
        0xcc20_ce9b_d35c_78a5,
        0x9816_5af3_7b21_53df,
        0xe2a0_b5dc_971f_303a,
        0xa8d9_d153_5ce3_b396,
        0xfb9b_7cd9_a4a7_443c,
        0xbb76_4c4c_a7a4_4410,
        0x8bab_8eef_b640_9c1a,
        0xd01f_ef10_a657_842c,
        0x9b10_a4e5_e991_3129,
        0xe710_9bfb_a19c_0c9d,
        0xac28_20d9_623b_f429,
        0x8044_4b5e_7aa7_cf85,
        0xbf21_e440_03ac_dd2d,
        0x8e67_9c2f_5e44_ff8f,
        0xd433_179d_9c8c_b841,
        0x9e19_db92_b4e3_1ba9,
        0xeb96_bf6e_badf_77d9,
        0xaf87_023b_9bf0_ee6b,
    ];
    const K_CACHED_POWERS_E: [i16; 87] = [
        -1220, -1193, -1166, -1140, -1113, -1087, -1060, -1034, -1007, -980, -954, -927, -901,
        -874, -847, -821, -794, -768, -741, -715, -688, -661, -635, -608, -582, -555, -529, -502,
        -475, -449, -422, -396, -369, -343, -316, -289, -263, -236, -210, -183, -157, -130, -103,
        -77, -50, -24, 3, 30, 56, 83, 109, 136, 162, 189, 216, 242, 269, 295, 322, 348, 375, 402,
        428, 455, 481, 508, 534, 561, 588, 614, 641, 667, 694, 720, 747, 774, 800, 827, 853, 880,
        907, 933, 960, 986, 1013, 1039, 1066,
    ];

    debug_assert!(index < K_CACHED_POWERS_F.len());
    DiyFp {
        f: K_CACHED_POWERS_F[index],
        e: K_CACHED_POWERS_E[index] as i32,
    }
}
