use half::f16;

pub trait Channel: Copy {
    fn to_unorm8(self) -> u8;
    fn from_unorm8(u: u8) -> Self;
    fn to_f32(self) -> f32;
    fn from_f32(f: f32) -> Self;
    const ZERO: Self;
}

impl Channel for u8 {
    const ZERO: Self = 0;

    fn to_unorm8(self) -> u8 {
        self
    }

    fn from_unorm8(u: u8) -> Self {
        u
    }

    fn to_f32(self) -> f32 {
        self as f32 / 255.0
    }

    fn from_f32(f: f32) -> Self {
        (f * 255.0) as u8
    }
}

impl Channel for i8 {
    const ZERO: Self = 0;

    fn to_unorm8(self) -> u8 {
        snorm8_to_unorm8(self as u8)
    }

    fn from_unorm8(u: u8) -> Self {
        unorm8_to_snorm8(u) as i8
    }

    fn to_f32(self) -> f32 {
        snorm8_to_float(self as u8)
    }

    fn from_f32(f: f32) -> Self {
        float_to_snorm8(f)
    }
}

impl Channel for u16 {
    const ZERO: Self = 0;

    fn to_unorm8(self) -> u8 {
        unorm16_to_unorm8(self)
    }

    fn from_unorm8(u: u8) -> Self {
        unorm8_to_unorm16(u)
    }

    fn to_f32(self) -> f32 {
        self as f32 / 65535.0
    }

    fn from_f32(f: f32) -> Self {
        (f * 65535.0) as u16
    }
}

impl Channel for i16 {
    const ZERO: Self = 0;

    fn to_unorm8(self) -> u8 {
        snorm16_to_unorm8(self as u16)
    }

    fn from_unorm8(u: u8) -> Self {
        unorm8_to_snorm16(u)
    }

    fn to_f32(self) -> f32 {
        snorm16_to_float(self as u16)
    }

    fn from_f32(f: f32) -> Self {
        float_to_snorm16(f)
    }
}

impl Channel for f16 {
    const ZERO: Self = f16::ZERO;

    fn to_unorm8(self) -> u8 {
        (self.to_f32() * 255.0) as u8
    }

    fn from_unorm8(u: u8) -> Self {
        f16::from_f32(u as f32 / 255.0)
    }

    fn to_f32(self) -> f32 {
        self.to_f32()
    }

    fn from_f32(f: f32) -> Self {
        f16::from_f32(f)
    }
}

impl Channel for f32 {
    const ZERO: Self = 0.0;

    fn to_unorm8(self) -> u8 {
        (self * 255.0) as u8
    }

    fn from_unorm8(u: u8) -> Self {
        u as f32 / 255.0
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn from_f32(f: f32) -> Self {
        f
    }
}

pub fn snorm8_to_unorm8(x: u8) -> u8 {
    // Validated against decoding R8Snorm DDS with GPU and paint.net (DirectXTex).
    if x < 128 {
        x + 128
    } else if x == 128 {
        0
    } else {
        x - 129
    }
}

pub fn unorm8_to_snorm8(x: u8) -> u8 {
    // Inverse of snorm_to_unorm.
    if x >= 128 {
        x - 128
    } else if x == 127 {
        0
    } else {
        x + 129
    }
}

pub fn snorm8_to_float(x: u8) -> f32 {
    ((x as i8) as f32 / 127.0).max(-1.0)
}

pub fn float_to_snorm8(x: f32) -> i8 {
    ((x.clamp(-1.0, 1.0)) * 127.0).round() as i8
}

pub fn snorm16_to_float(x: u16) -> f32 {
    ((x as i16) as f32 / 32767.0).max(-1.0)
}

pub fn float_to_snorm16(x: f32) -> i16 {
    ((x.clamp(-1.0, 1.0)) * 32767.0).round() as i16
}

// https://rundevelopment.github.io/blog/fast-unorm-conversions
pub fn unorm4_to_unorm8(x: u8) -> u8 {
    x * 17
}

pub fn unorm8_to_unorm4(x: u8) -> u8 {
    ((x as u16 * 15 + 135) >> 8) as u8
}

pub fn unorm16_to_unorm8(x: u16) -> u8 {
    ((x as u32 * 255 + 32895) >> 16) as u8
}

pub fn unorm8_to_unorm16(x: u8) -> u16 {
    x as u16 * 257
}

// TODO: Find an efficient way to do this and add tests.
pub fn snorm16_to_unorm8(x: u16) -> u8 {
    // Remap [-1, 1] to [0, 1] to fit in an unsigned integer.
    ((snorm16_to_float(x) * 0.5 + 0.5) * 255.0).round() as u8
}

pub fn unorm8_to_snorm16(x: u8) -> i16 {
    // Remap [0, 1] to [-1, 1] to fit in a signed integer.
    (((x as f32 / 255.0) * 2.0 - 1.0) * 32767.0).round() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snorm8_to_unorm8_reference(x: u8) -> u8 {
        // Remap [-1, 1] to [0, 1] to fit in an unsigned integer.
        ((snorm8_to_float(x) * 0.5 + 0.5) * 255.0).round() as u8
    }

    fn unorm8_to_snorm8_reference(x: u8) -> i8 {
        // Remap [0, 1] to [-1, 1] to fit in a signed integer.
        (((x as f32 / 255.0) * 2.0 - 1.0) * 127.0).round() as i8
    }

    #[test]
    fn convert_snorm8_to_unorm8() {
        // 128, ..., 255, 0, ..., 126
        for i in 0..=255 {
            assert_eq!(snorm8_to_unorm8(i), snorm8_to_unorm8_reference(i));
        }
    }

    #[test]
    fn convert_unorm8_to_snorm8() {
        // 129, ..., 255, 0, ..., 127
        for i in 0..=255 {
            assert_eq!(unorm8_to_snorm8(i) as i8, unorm8_to_snorm8_reference(i));
        }
    }

    #[test]
    fn snorm8_unorm8_inverse() {
        for i in 0..=255 {
            if i != 128 {
                assert_eq!(unorm8_to_snorm8(snorm8_to_unorm8(i)), i);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(unorm8_to_snorm8(128), 0);
    }

    #[test]
    fn snorm8_unorm8_float() {
        for i in 0..=255 {
            if i != 128 {
                assert_eq!(float_to_snorm8(snorm8_to_float(i)), i as i8);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(snorm8_to_float(128), -1.0);
    }

    fn unorm4_to_unorm8_reference(x: u8) -> u8 {
        (x as f32 / 15.0 * 255.0).round() as u8
    }

    fn unorm8_to_unorm4_reference(x: u8) -> u8 {
        (x as f32 / 255.0 * 15.0).round() as u8
    }

    #[test]
    fn convert_unorm8_to_unorm4() {
        for i in 0..=255 {
            assert_eq!(unorm8_to_unorm4(i), unorm8_to_unorm4_reference(i));
        }
    }

    #[test]
    fn convert_unorm4_to_unorm8() {
        for i in 0..=15 {
            assert_eq!(unorm4_to_unorm8(i), unorm4_to_unorm8_reference(i));
        }
    }

    fn unorm16_to_unorm8_reference(x: u16) -> u8 {
        (x as f32 / 65535.0 * 255.0).round() as u8
    }

    fn unorm8_to_unorm16_reference(x: u8) -> u16 {
        (x as f32 / 255.0 * 65535.0).round() as u16
    }

    #[test]
    fn convert_unorm8_to_unorm16() {
        for i in 0..=255 {
            assert_eq!(unorm8_to_unorm16(i), unorm8_to_unorm16_reference(i));
        }
    }

    #[test]
    fn convert_unorm16_to_unorm8() {
        for i in 0..=65535 {
            assert_eq!(unorm16_to_unorm8(i), unorm16_to_unorm8_reference(i));
        }
    }

    #[test]
    fn snorm16_unorm16_float() {
        for i in 0..=65535 {
            if i != 32768 {
                assert_eq!(float_to_snorm16(snorm16_to_float(i)), i as i16);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(snorm16_to_float(32768), -1.0);
    }
}
