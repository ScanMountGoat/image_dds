use bytemuck::{Pod, Zeroable};
use half::f16;

use crate::{
    float_to_snorm16, float_to_snorm8, snorm16_to_float, snorm16_to_unorm8, snorm8_to_float,
    snorm8_to_unorm8, unorm16_to_unorm8, unorm4_to_unorm8, unorm8_to_snorm16, unorm8_to_snorm8,
    unorm8_to_unorm16, unorm8_to_unorm4, SurfaceError,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgba8([u8; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rf16([f16; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgf16([f16; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgbaf16([f16; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rf32([f32; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgf32([f32; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgbaf32([f32; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R8([u8; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R8Snorm([u8; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rg8([u8; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rg8Snorm([u8; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgb8([u8; 3]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgr8([u8; 3]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgra8([u8; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgra4([u8; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R16([u16; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R16Snorm([u16; 1]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rg16([u16; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rg16Snorm([u16; 2]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgba16([u16; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgba16Snorm([u16; 4]);

pub trait Pixel {
    fn get_pixel(data: &[u8], index: usize) -> Self;
}

macro_rules! pixel_impl {
    ($($ty:ty),*) => {
        $(
            impl Pixel for $ty {
                fn get_pixel(data: &[u8], index: usize) -> Self {
                    Self(pixel_from_bytes(data, index))
                }
            }
        )*
    };
}
pixel_impl!(
    R8,
    R8Snorm,
    R16,
    R16Snorm,
    Rg8,
    Rg8Snorm,
    Rg16,
    Rg16Snorm,
    Bgra4,
    Rgb8,
    Bgr8,
    Rgba8,
    Bgra8,
    Rgba16,
    Rgba16Snorm,
    Rf16,
    Rgf16,
    Rgbaf16,
    Rf32,
    Rgf32,
    Rgbaf32
);

// TODO: Implement using macro or generic function?
// num channels, channel swizzles, function or value for each channel conversion?
pub trait ToRgba<T> {
    fn to_rgba(self) -> [T; 4];
}

pub trait FromRgba<T> {
    fn from_rgba(rgba: [T; 4]) -> Self;
}

pub trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl FromBytes for u8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

macro_rules! from_bytes_impl {
    ($($ty:ty),*) => {
        $(
            impl FromBytes for $ty {
                fn from_bytes(bytes: &[u8]) -> Self {
                    // Don't assume system endianness.
                    Self::from_le_bytes(bytes[..std::mem::size_of::<Self>()].try_into().unwrap())
                }
            }
        )*
    };
}
from_bytes_impl!(u16, f16, f32);

fn pixel_from_bytes<const N: usize, P: FromBytes>(data: &[u8], index: usize) -> [P; N] {
    std::array::from_fn(|i| {
        let size = std::mem::size_of::<P>();
        let start = (index + i) * size;
        P::from_bytes(&data[start..start + size])
    })
}

fn get_pixel<T>(data: &[T], index: usize, size: usize) -> &[T] {
    &data[index * size..index * size + size]
}

impl ToRgba<u8> for Rgba8 {
    fn to_rgba(self) -> [u8; 4] {
        self.0
    }
}

impl FromRgba<u8> for Rgba8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba)
    }
}

impl ToRgba<u8> for Rgbaf16 {
    fn to_rgba(self) -> [u8; 4] {
        self.0.map(|f| (f.to_f32() * 255.0) as u8)
    }
}

impl FromRgba<u8> for Rgf16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([
            f16::from_f32(rgba[0] as f32 / 255.0),
            f16::from_f32(rgba[1] as f32 / 255.0),
        ])
    }
}

impl ToRgba<f32> for Rgf16 {
    fn to_rgba(self) -> [f32; 4] {
        [self.0[0].to_f32(), self.0[1].to_f32(), 0.0, 1.0]
    }
}

impl FromRgba<f32> for Rgf16 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([f16::from_f32(rgba[0]), f16::from_f32(rgba[1])])
    }
}

impl ToRgba<u8> for Rgf16 {
    fn to_rgba(self) -> [u8; 4] {
        [
            (self.0[0].to_f32() * 255.0) as u8,
            (self.0[1].to_f32() * 255.0) as u8,
            0u8,
            255u8,
        ]
    }
}

impl FromRgba<u8> for Rgbaf16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba.map(|u| f16::from_f32(u as f32 / 255.0)))
    }
}

impl ToRgba<f32> for Rgbaf16 {
    fn to_rgba(self) -> [f32; 4] {
        self.0.map(f16::to_f32)
    }
}

impl FromRgba<f32> for Rgbaf16 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self(rgba.map(f16::from_f32))
    }
}

impl FromRgba<u8> for Rgf32 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[0] as f32 / 255.0, rgba[1] as f32 / 255.0])
    }
}

impl ToRgba<f32> for Rgf32 {
    fn to_rgba(self) -> [f32; 4] {
        [self.0[0], self.0[1], 0.0, 1.0]
    }
}

impl FromRgba<f32> for Rgf32 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([rgba[0], rgba[1]])
    }
}

impl ToRgba<u8> for Rgf32 {
    fn to_rgba(self) -> [u8; 4] {
        [
            (self.0[0] * 255.0) as u8,
            (self.0[1] * 255.0) as u8,
            0u8,
            255u8,
        ]
    }
}

impl ToRgba<u8> for Rgbaf32 {
    fn to_rgba(self) -> [u8; 4] {
        self.0.map(|f| (f * 255.0) as u8)
    }
}

impl FromRgba<u8> for Rgbaf32 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba.map(|u| u as f32 / 255.0))
    }
}

impl ToRgba<f32> for Rgbaf32 {
    fn to_rgba(self) -> [f32; 4] {
        self.0
    }
}

impl FromRgba<f32> for Rgbaf32 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self(rgba)
    }
}

impl ToRgba<u8> for Rf32 {
    fn to_rgba(self) -> [u8; 4] {
        let r = (self.0[0] * 255.0) as u8;
        [r, r, r, 255u8]
    }
}

impl FromRgba<u8> for Rf32 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[0] as f32 / 255.0])
    }
}

impl ToRgba<f32> for Rf32 {
    fn to_rgba(self) -> [f32; 4] {
        [self.0[0], self.0[0], self.0[0], 1.0]
    }
}

impl FromRgba<f32> for Rf32 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([rgba[0]])
    }
}

impl ToRgba<u8> for Rf16 {
    fn to_rgba(self) -> [u8; 4] {
        let r = (self.0[0].to_f32() * 255.0) as u8;
        [r, r, r, 255u8]
    }
}

impl FromRgba<u8> for Rf16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([f16::from_f32(rgba[0] as f32 / 255.0)])
    }
}

impl ToRgba<f32> for Rf16 {
    fn to_rgba(self) -> [f32; 4] {
        let r = self.0[0].to_f32();
        [r, r, r, 1.0]
    }
}

impl FromRgba<f32> for Rf16 {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([f16::from_f32(rgba[0])])
    }
}

impl ToRgba<u8> for R8 {
    fn to_rgba(self) -> [u8; 4] {
        [self.0[0], self.0[0], self.0[0], 255u8]
    }
}

impl FromRgba<u8> for R8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[0]])
    }
}

impl ToRgba<u8> for R8Snorm {
    fn to_rgba(self) -> [u8; 4] {
        let r = snorm8_to_unorm8(self.0[0]);
        [r, r, r, 255u8]
    }
}

impl FromRgba<u8> for R8Snorm {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([unorm8_to_snorm8(rgba[0])])
    }
}

impl ToRgba<f32> for R8Snorm {
    fn to_rgba(self) -> [f32; 4] {
        let r = snorm8_to_float(self.0[0]);
        [r, r, r, 1.0]
    }
}

impl FromRgba<f32> for R8Snorm {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([float_to_snorm8(rgba[0]) as u8])
    }
}

impl ToRgba<u8> for Rg8 {
    fn to_rgba(self) -> [u8; 4] {
        [self.0[0], self.0[1], 0u8, 255u8]
    }
}

impl FromRgba<u8> for Rg8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[0], rgba[1]])
    }
}

impl ToRgba<u8> for Rg8Snorm {
    fn to_rgba(self) -> [u8; 4] {
        [
            snorm8_to_unorm8(self.0[0]),
            snorm8_to_unorm8(self.0[1]),
            snorm8_to_unorm8(0u8),
            255u8,
        ]
    }
}

impl FromRgba<u8> for Rg8Snorm {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([unorm8_to_snorm8(rgba[0]), unorm8_to_snorm8(rgba[1])])
    }
}

impl ToRgba<f32> for Rg8Snorm {
    fn to_rgba(self) -> [f32; 4] {
        [
            snorm8_to_float(self.0[0]),
            snorm8_to_float(self.0[1]),
            snorm8_to_float(0u8),
            1.0,
        ]
    }
}

impl FromRgba<f32> for Rg8Snorm {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([
            float_to_snorm8(rgba[0]) as u8,
            float_to_snorm8(rgba[1]) as u8,
        ])
    }
}

impl ToRgba<u8> for Rgb8 {
    fn to_rgba(self) -> [u8; 4] {
        [self.0[0], self.0[1], self.0[2], 255u8]
    }
}

impl FromRgba<u8> for Rgb8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[0], rgba[1], rgba[2]])
    }
}

impl ToRgba<u8> for Bgr8 {
    fn to_rgba(self) -> [u8; 4] {
        [self.0[2], self.0[1], self.0[0], 255u8]
    }
}

impl FromRgba<u8> for Bgr8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[2], rgba[1], rgba[0]])
    }
}

impl ToRgba<u8> for Bgra8 {
    fn to_rgba(self) -> [u8; 4] {
        [self.0[2], self.0[1], self.0[0], self.0[3]]
    }
}

impl FromRgba<u8> for Bgra8 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([rgba[2], rgba[1], rgba[0], rgba[3]])
    }
}

impl ToRgba<u8> for Bgra4 {
    fn to_rgba(self) -> [u8; 4] {
        // Expand 4 bit input channels to 8 bit output channels.
        // Most significant bit -> ARGB -> least significant bit.
        [
            unorm4_to_unorm8(self.0[1] & 0xF),
            unorm4_to_unorm8(self.0[0] >> 4),
            unorm4_to_unorm8(self.0[0] & 0xF),
            unorm4_to_unorm8(self.0[1] >> 4),
        ]
    }
}

impl FromRgba<u8> for Bgra4 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        // Pack each channel into 4 bits.
        // Most significant bit -> ARGB -> least significant bit.
        Self([
            ((unorm8_to_unorm4(rgba[1])) << 4) | (unorm8_to_unorm4(rgba[2])),
            ((unorm8_to_unorm4(rgba[3])) << 4) | (unorm8_to_unorm4(rgba[0])),
        ])
    }
}

impl ToRgba<u8> for R16 {
    fn to_rgba(self) -> [u8; 4] {
        let r = unorm16_to_unorm8(self.0[0]);
        [r, r, r, 255u8]
    }
}

impl FromRgba<u8> for R16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([unorm8_to_unorm16(rgba[0])])
    }
}

impl ToRgba<u8> for R16Snorm {
    fn to_rgba(self) -> [u8; 4] {
        let r = snorm16_to_unorm8(self.0[0]);
        [r, r, r, 255u8]
    }
}

impl FromRgba<u8> for R16Snorm {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([unorm8_to_snorm16(rgba[0]) as u16])
    }
}

impl ToRgba<f32> for R16Snorm {
    fn to_rgba(self) -> [f32; 4] {
        let r = snorm16_to_float(self.0[0]);
        [r, r, r, 1.0]
    }
}

impl FromRgba<f32> for R16Snorm {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([float_to_snorm16(rgba[0]) as u16])
    }
}

impl ToRgba<u8> for Rg16 {
    fn to_rgba(self) -> [u8; 4] {
        [
            unorm16_to_unorm8(self.0[0]),
            unorm16_to_unorm8(self.0[1]),
            0u8,
            255u8,
        ]
    }
}

impl FromRgba<u8> for Rg16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([unorm8_to_unorm16(rgba[0]), unorm8_to_unorm16(rgba[1])])
    }
}

impl ToRgba<u8> for Rg16Snorm {
    fn to_rgba(self) -> [u8; 4] {
        [
            snorm16_to_unorm8(self.0[0]),
            snorm16_to_unorm8(self.0[1]),
            snorm16_to_unorm8(0u16),
            255u8,
        ]
    }
}

impl FromRgba<u8> for Rg16Snorm {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([
            unorm8_to_snorm16(rgba[0]) as u16,
            unorm8_to_snorm16(rgba[1]) as u16,
        ])
    }
}

impl ToRgba<f32> for Rg16Snorm {
    fn to_rgba(self) -> [f32; 4] {
        [
            snorm16_to_float(self.0[0]),
            snorm16_to_float(self.0[1]),
            snorm16_to_float(0u16),
            1.0,
        ]
    }
}

impl FromRgba<f32> for Rg16Snorm {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([
            float_to_snorm16(rgba[0]) as u16,
            float_to_snorm16(rgba[1]) as u16,
        ])
    }
}

impl ToRgba<u8> for Rgba16 {
    fn to_rgba(self) -> [u8; 4] {
        self.0.map(unorm16_to_unorm8)
    }
}

impl FromRgba<u8> for Rgba16 {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba.map(unorm8_to_unorm16))
    }
}

impl ToRgba<u8> for Rgba16Snorm {
    fn to_rgba(self) -> [u8; 4] {
        self.0.map(snorm16_to_unorm8)
    }
}

impl FromRgba<u8> for Rgba16Snorm {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba.map(|u| unorm8_to_snorm16(u) as u16))
    }
}

impl ToRgba<f32> for Rgba16Snorm {
    fn to_rgba(self) -> [f32; 4] {
        self.0.map(snorm16_to_float)
    }
}

impl FromRgba<f32> for Rgba16Snorm {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self(rgba.map(|u| float_to_snorm16(u) as u16))
    }
}

pub fn encode_rgba<P, T>(width: u32, height: u32, data: &[T]) -> Result<Vec<u8>, SurfaceError>
where
    P: FromRgba<T> + Pod,
    T: Pod + FromBytes,
{
    validate_length(width, height, 4, data)?;
    // TODO: Find a better way to convert to bytes.
    Ok(bytemuck::cast_slice(
        &(0..width * height)
            .map(|i| P::from_rgba(get_pixel(data, i as usize, 4).try_into().unwrap()))
            .collect::<Vec<_>>(),
    )
    .to_vec())
}

pub fn decode_rgba<P, T>(width: u32, height: u32, data: &[u8]) -> Result<Vec<T>, SurfaceError>
where
    P: Pixel + ToRgba<T>,
{
    validate_length(width, height, std::mem::size_of::<P>(), data)?;
    Ok((0..width * height)
        .flat_map(|i| P::get_pixel(data, i as usize).to_rgba())
        .collect::<Vec<_>>())
}

fn validate_length<T>(
    width: u32,
    height: u32,
    elements_per_pixel: usize,
    data: &[T],
) -> Result<usize, SurfaceError> {
    let expected = expected_size(width, height, elements_per_pixel).ok_or(
        SurfaceError::PixelCountWouldOverflow {
            width,
            height,
            depth: 1,
        },
    )?;

    if data.len() < expected {
        Err(SurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    } else {
        Ok(expected)
    }
}

fn expected_size(width: u32, height: u32, bytes_per_pixel: usize) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(bytes_per_pixel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r8_from_rgba8_valid() {
        assert_eq!(vec![1], encode_rgba::<R8, u8>(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn r8_from_rgba8_invalid() {
        let result = encode_rgba::<R8, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_r8_valid() {
        assert_eq!(
            vec![64, 64, 64, 255],
            decode_rgba::<R8, u8>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_invalid() {
        let result = decode_rgba::<R8, u8>(4, 4, &[64]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 1
            })
        );
    }

    #[test]
    fn r8_snorm_from_rgba8_valid() {
        assert_eq!(
            vec![130],
            encode_rgba::<R8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn r8_snorm_from_rgba8_invalid() {
        let result = encode_rgba::<R8Snorm, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn r8_snorm_from_rgbaf32_valid() {
        assert_eq!(
            vec![129],
            encode_rgba::<R8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn r8_snorm_from_rgbaf32_invalid() {
        let result = encode_rgba::<R8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_r8_snorm_valid() {
        assert_eq!(
            vec![192, 192, 192, 255],
            decode_rgba::<R8Snorm, u8>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_snorm_invalid() {
        let result = decode_rgba::<R8Snorm, u8>(4, 4, &[64]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 1
            })
        );
    }

    #[test]
    fn rgbaf32_from_r8_snorm_valid() {
        assert_eq!(
            vec![-1.0, -1.0, -1.0, 1.0],
            decode_rgba::<R8Snorm, f32>(1, 1, &[128]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_r8_snorm_invalid() {
        let result = decode_rgba::<R8Snorm, f32>(4, 4, &[128]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 1
            })
        );
    }

    #[test]
    fn rg8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2],
            encode_rgba::<Rg8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_from_rgba8_invalid() {
        let result = encode_rgba::<Rg8, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rg8_snorm_from_rgba8_valid() {
        assert_eq!(
            vec![130, 131],
            encode_rgba::<Rg8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgba8_invalid() {
        let result = encode_rgba::<Rg8Snorm, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rg8_snorm_from_rgbaf32_valid() {
        assert_eq!(
            vec![129, 0],
            encode_rgba::<Rg8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgbaf32_invalid() {
        let result = encode_rgba::<Rg8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rg8_snorm_valid() {
        assert_eq!(
            vec![129, 130, 128, 255],
            decode_rgba::<Rg8Snorm, u8>(1, 1, &[1, 2]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg8_snorm_invalid() {
        let result = decode_rgba::<Rg8Snorm, u8>(1, 1, &[64]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn rgbaf32_from_rg8_snorm_valid() {
        assert_eq!(
            vec![-1.0, 0.0, 0.0, 1.0],
            decode_rgba::<Rg8Snorm, f32>(1, 1, &[128, 0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rg8_snorm_invalid() {
        let result = decode_rgba::<Rg8Snorm, f32>(1, 1, &[64]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn bgra8_from_rgba8_valid() {
        assert_eq!(
            vec![3, 2, 1, 4],
            encode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn bgra8_from_rgba8_invalid() {
        let result = encode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_bgra8_valid() {
        assert_eq!(
            vec![3, 2, 1, 4],
            decode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra8_invalid() {
        let result = decode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgb8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2, 3],
            encode_rgba::<Rgb8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgb8_from_rgba8_invalid() {
        let result = encode_rgba::<Rgb8, u8>(1, 1, &[1, 2]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 2
            })
        );
    }

    #[test]
    fn rgba8_from_bgr8_valid() {
        assert_eq!(
            vec![3, 2, 1, 255],
            decode_rgba::<Bgr8, u8>(1, 1, &[1, 2, 3]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgr8_invalid() {
        let result = decode_rgba::<Bgr8, u8>(1, 1, &[1, 2]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 3,
                actual: 2
            })
        );
    }

    #[test]
    fn rgba8_from_rgf32_valid() {
        assert_eq!(
            vec![0, 63, 0, 255],
            decode_rgba::<Rgf32, u8>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgf32_invalid() {
        let result = decode_rgba::<Rgf32, u8>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgf32_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32]),
            encode_rgba::<Rgf32, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgf32_from_rgba8_invalid() {
        let result = encode_rgba::<Rgf32, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rf16_valid() {
        assert_eq!(
            vec![63, 63, 63, 255],
            decode_rgba::<Rf16, u8>(1, 1, bytemuck::cast_slice(&[f16::from_f32(0.25f32)])).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rf16_invalid() {
        let result = decode_rgba::<Rf16, u8>(1, 1, &[0; 1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn rf16_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[f16::from_f32(0.2f32)]),
            encode_rgba::<Rf16, u8>(1, 1, &[51, 27, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rf16_from_rgba8_invalid() {
        let result = encode_rgba::<Rf16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rf32_valid() {
        assert_eq!(
            vec![63, 63, 63, 255],
            decode_rgba::<Rf32, u8>(1, 1, bytemuck::cast_slice(&[0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rf32_invalid() {
        let result = decode_rgba::<Rf32, u8>(1, 1, &[0; 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rf32_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.2f32]),
            encode_rgba::<Rf32, u8>(1, 1, &[51, 27, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rf32_from_rgba8_invalid() {
        let result = encode_rgba::<Rf32, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_valid() {
        assert_eq!(
            vec![0, 63, 127, 255],
            decode_rgba::<Rgbaf32, u8>(
                1,
                1,
                bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32, 1.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_invalid() {
        let result = decode_rgba::<Rgbaf32, u8>(1, 1, &[0; 15]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32, 0.6f32, 1.0f32]),
            encode_rgba::<Rgbaf32, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_invalid() {
        let result = encode_rgba::<Rgbaf32, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgf16_valid() {
        assert_eq!(
            vec![0, 63, 0, 255],
            decode_rgba::<Rgf16, u8>(
                1,
                1,
                bytemuck::cast_slice(&[f16::from_f32(0.0f32), f16::from_f32(0.25f32),])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgf16_invalid() {
        let result = decode_rgba::<Rgf16, u8>(1, 1, &[0; 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgf16_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<f16, u8>(&[f16::from_f32(0.0f32), f16::from_f32(0.2f32),]),
            encode_rgba::<Rgf16, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgf16_from_rgba8_invalid() {
        let result = encode_rgba::<Rgf16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgbaf16_valid() {
        assert_eq!(
            vec![0, 63, 127, 255],
            decode_rgba::<Rgbaf16, u8>(
                1,
                1,
                bytemuck::cast_slice(&[
                    f16::from_f32(0.0f32),
                    f16::from_f32(0.25f32),
                    f16::from_f32(0.5f32),
                    f16::from_f32(1.0f32)
                ])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbaf16_invalid() {
        let result = decode_rgba::<Rgbaf16, u8>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgbaf16_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<f16, u8>(&[
                f16::from_f32(0.0f32),
                f16::from_f32(0.2f32),
                f16::from_f32(0.6f32),
                f16::from_f32(1.0f32)
            ]),
            encode_rgba::<Rgbaf16, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf16_from_rgba8_invalid() {
        let result = encode_rgba::<Rgbaf16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2, 3, 4],
            decode_rgba::<Rgba8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba8_invalid() {
        let result = decode_rgba::<Rgba8, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_decode_valid() {
        assert_eq!(
            vec![1.0, 2.0, 3.0, 4.0],
            decode_rgba::<Rgbaf32, f32>(
                1,
                1,
                bytemuck::cast_slice(&[1.0f32, 2.0f32, 3.0f32, 4.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_encode_valid() {
        assert_eq!(
            bytemuck::cast_slice::<f32, u8>(&[1.0f32, 2.0f32, 3.0f32, 4.0f32]),
            &encode_rgba::<Rgbaf32, f32>(1, 1, &[1.0f32, 2.0f32, 3.0f32, 4.0f32]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_decode_invalid() {
        let result = decode_rgba::<Rgbaf32, f32>(1, 1, &[0; 15]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_encode_invalid() {
        let result = encode_rgba::<Rgbaf32, f32>(1, 1, &[0.0; 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf16_valid() {
        assert_eq!(
            vec![0.0, 0.25, 0.5, 1.0],
            decode_rgba::<Rgbaf16, f32>(
                1,
                1,
                bytemuck::cast_slice(&[
                    f16::from_f32(0.0f32),
                    f16::from_f32(0.25f32),
                    f16::from_f32(0.5f32),
                    f16::from_f32(1.0f32)
                ])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf16_invalid() {
        let result = decode_rgba::<Rgbaf16, f32>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgf16_valid() {
        assert_eq!(
            vec![0.0, 0.25, 0.0, 1.0],
            decode_rgba::<Rgf16, f32>(
                1,
                1,
                bytemuck::cast_slice(&[f16::from_f32(0.0f32), f16::from_f32(0.25f32),])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgf16_invalid() {
        let result = decode_rgba::<Rgf16, f32>(1, 1, &[0; 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgf32_valid() {
        assert_eq!(
            vec![0.0, 0.25, 0.0, 1.0],
            decode_rgba::<Rgf32, f32>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgf32_invalid() {
        let result = decode_rgba::<Rgf32, f32>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgbaf32_from_rf32_valid() {
        assert_eq!(
            vec![0.25, 0.25, 0.25, 1.0],
            decode_rgba::<Rf32, f32>(1, 1, bytemuck::cast_slice(&[0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rf32_invalid() {
        let result = decode_rgba::<Rf32, f32>(1, 1, &[0; 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rf16_valid() {
        assert_eq!(
            vec![0.25, 0.25, 0.25, 1.0],
            decode_rgba::<Rf16, f32>(1, 1, bytemuck::cast_slice(&[f16::from_f32(0.25f32)]))
                .unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rf16_invalid() {
        let result = decode_rgba::<Rf16, f32>(1, 1, &[0; 1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn bgra4_from_rgba8_valid() {
        assert_eq!(
            vec![0x30, 0xCF],
            encode_rgba::<Bgra4, u8>(1, 1, &[255, 51, 0, 204]).unwrap()
        );
    }

    #[test]
    fn bgra4_from_rgba8_invalid() {
        let result = encode_rgba::<Bgra4, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_bgra4_valid() {
        assert_eq!(
            vec![255, 51, 0, 204],
            decode_rgba::<Bgra4, u8>(1, 1, &[0x30, 0xCF]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra4_invalid() {
        let result = decode_rgba::<Bgra4, u8>(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn r16_from_rgba8_valid() {
        assert_eq!(
            vec![127, 127],
            encode_rgba::<R16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn r16_from_rgba8_invalid() {
        let result = encode_rgba::<R16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_r16_valid() {
        assert_eq!(
            vec![127, 127, 127, 255],
            decode_rgba::<R16, u8>(1, 1, &[127, 127]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r16_invalid() {
        let result = decode_rgba::<R16, u8>(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn r16_snorm_from_rgba8_valid() {
        assert_eq!(
            vec![128, 255],
            encode_rgba::<R16Snorm, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn r16_snorm_from_rgba8_invalid() {
        let result = encode_rgba::<R16Snorm, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_r16_snorm_valid() {
        assert_eq!(
            vec![255, 255, 255, 255],
            decode_rgba::<R16Snorm, u8>(1, 1, &[127, 127]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r16_snorm_invalid() {
        let result = decode_rgba::<R16Snorm, u8>(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn r16_snorm_from_rgbaf32_valid() {
        assert_eq!(
            vec![1, 128],
            encode_rgba::<R16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn r16_snorm_from_rgbaf32_invalid() {
        let result = encode_rgba::<R16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_r16_snorm_valid() {
        assert_eq!(
            vec![-1.0, -1.0, -1.0, 1.0],
            decode_rgba::<R16Snorm, f32>(1, 1, &[1, 128]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_r16_snorm_invalid() {
        let result = decode_rgba::<R16Snorm, f32>(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn rg16_from_rgba8_valid() {
        assert_eq!(
            vec![127, 127, 128, 128],
            encode_rgba::<Rg16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rg16_from_rgba8_invalid() {
        let result = encode_rgba::<Rg16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rg16_valid() {
        assert_eq!(
            vec![127, 128, 0, 255],
            decode_rgba::<Rg16, u8>(1, 1, &[127, 127, 128, 128]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg16_invalid() {
        let result = decode_rgba::<Rg16, u8>(1, 1, &[1, 1, 1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rg16_snorm_from_rgba8_valid() {
        assert_eq!(
            vec![128, 255, 128, 0],
            encode_rgba::<Rg16Snorm, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rg16_snorm_from_rgba8_invalid() {
        let result = encode_rgba::<Rg16Snorm, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rg16_snorm_valid() {
        assert_eq!(
            vec![255, 0, 128, 255],
            decode_rgba::<Rg16Snorm, u8>(1, 1, &[127, 127, 128, 128]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg16_snorm_invalid() {
        let result = decode_rgba::<Rg16Snorm, u8>(1, 1, &[1, 1, 1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rg16_snorm_from_rgbaf32_valid() {
        assert_eq!(
            vec![1, 128, 0, 0],
            encode_rgba::<Rg16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rg16_snorm_from_rgbaf32_invalid() {
        let result = encode_rgba::<Rg16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rg16_snorm_valid() {
        assert_eq!(
            vec![-1.0, 1.0, 0.0, 1.0],
            decode_rgba::<Rg16Snorm, f32>(1, 1, &[1, 128, 255, 127]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rg16_snorm_invalid() {
        let result = decode_rgba::<Rg16Snorm, f32>(1, 1, &[1, 1, 1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba16_from_rgba8_valid() {
        assert_eq!(
            vec![127, 127, 128, 128, 129, 129, 130, 130],
            encode_rgba::<Rgba16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba16_from_rgba8_invalid() {
        let result = encode_rgba::<Rgba16, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgba16_valid() {
        assert_eq!(
            vec![127, 128, 129, 130],
            decode_rgba::<Rgba16, u8>(1, 1, &[127, 127, 128, 128, 129, 129, 130, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba16_invalid() {
        let result = decode_rgba::<Rgba16, u8>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgba16_snorm_from_rgba8_valid() {
        assert_eq!(
            vec![128, 255, 128, 0, 129, 1, 130, 2],
            encode_rgba::<Rgba16Snorm, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba16_snorm_from_rgba8_invalid() {
        let result = encode_rgba::<Rgba16Snorm, u8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgba8_from_rgba16_snorm_valid() {
        assert_eq!(
            vec![255, 0, 1, 2],
            decode_rgba::<Rgba16Snorm, u8>(1, 1, &[127, 127, 128, 128, 129, 129, 130, 130])
                .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba16_snorm_invalid() {
        let result = decode_rgba::<Rgba16Snorm, u8>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn rgba16_snorm_from_rgbaf32_valid() {
        assert_eq!(
            vec![1, 128, 0, 0, 0, 64, 255, 127],
            encode_rgba::<Rgba16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgba16_snorm_from_rgbaf32_invalid() {
        let result = encode_rgba::<Rgba16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgba16_snorm_valid() {
        assert_eq!(
            vec![-1.0, 0.0, -1.0, 1.0],
            decode_rgba::<Rgba16Snorm, f32>(1, 1, &[1, 128, 0, 0, 0, 128, 255, 127]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgba16_snorm_invalid() {
        let result = decode_rgba::<Rgba16Snorm, f32>(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }
}
