use bytemuck::{Pod, Zeroable};
use half::f16;

use crate::SurfaceError;
use convert::*;

pub mod convert;

pub type R8 = R<u8>;
pub type Rg8 = Rg<u8>;
pub type Rgb8 = Rgb<u8>;
pub type Rgba8 = Rgba<u8>;

pub type R8Snorm = R<i8>;
pub type Rg8Snorm = Rg<i8>;
pub type Rgba8Snorm = Rgba<i8>;

pub type Rf16 = R<f16>;
pub type Rgf16 = Rg<f16>;
pub type Rgbaf16 = Rgba<f16>;

pub type Rf32 = R<f32>;
pub type Rgf32 = Rg<f32>;
pub type Rgbf32 = Rgb<f32>;
pub type Rgbaf32 = Rgba<f32>;

pub type R16 = R<u16>;
pub type Rg16 = Rg<u16>;
pub type Rgba16 = Rgba<u16>;

pub type R16Snorm = R<i16>;
pub type Rg16Snorm = Rg<i16>;
pub type Rgba16Snorm = Rgba<i16>;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgr8([u8; 3]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgra8([u8; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Bgra4([u8; 2]);

pub trait GetPixel {
    fn get_pixel(data: &[u8], index: usize) -> Self;
}

macro_rules! pixel_impl {
    ($($ty:ty),*) => {
        $(
            impl GetPixel for $ty {
                fn get_pixel(data: &[u8], index: usize) -> Self {
                    Self(pixel_from_bytes(data, index))
                }
            }
        )*
    };
}
// TODO: Implement automatically for generic types?
pixel_impl!(
    R8,
    Rg8,
    Rgb8,
    Rgba8,
    R8Snorm,
    Rg8Snorm,
    Rgba8Snorm,
    R16Snorm,
    Rg16Snorm,
    Rgba16Snorm,
    R16,
    Rg16,
    Rgba16,
    Bgra4,
    Bgr8,
    Bgra8,
    Rf16,
    Rgf16,
    Rgbaf16,
    Rf32,
    Rgf32,
    Rgbf32,
    Rgbaf32
);

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

impl FromBytes for i8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0] as i8
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
from_bytes_impl!(u16, i16, f16, f32);

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R<T>([T; 1]);

impl<T: Channel> FromRgba<u8> for R<T> {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([T::from_unorm8(rgba[0])])
    }
}

impl<T: Channel> ToRgba<u8> for R<T> {
    fn to_rgba(self) -> [u8; 4] {
        let r = T::to_unorm8(self.0[0]);
        [r, r, r, 255u8]
    }
}

impl<T: Channel> FromRgba<f32> for R<T> {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([T::from_f32(rgba[0])])
    }
}

impl<T: Channel> ToRgba<f32> for R<T> {
    fn to_rgba(self) -> [f32; 4] {
        let r = T::to_f32(self.0[0]);
        [r, r, r, 1.0]
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rg<T>([T; 2]);

impl<T: Channel> FromRgba<u8> for Rg<T> {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([T::from_unorm8(rgba[0]), T::from_unorm8(rgba[1])])
    }
}

impl<T: Channel> ToRgba<u8> for Rg<T> {
    fn to_rgba(self) -> [u8; 4] {
        // The blue channel converts 0 for unorm/snorm by convention.
        [
            T::to_unorm8(self.0[0]),
            T::to_unorm8(self.0[1]),
            T::to_unorm8(T::ZERO),
            255u8,
        ]
    }
}

impl<T: Channel> FromRgba<f32> for Rg<T> {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([T::from_f32(rgba[0]), T::from_f32(rgba[1])])
    }
}

impl<T: Channel> ToRgba<f32> for Rg<T> {
    fn to_rgba(self) -> [f32; 4] {
        // The blue channel converts 0 for unorm/snorm by convention.
        [
            T::to_f32(self.0[0]),
            T::to_f32(self.0[1]),
            T::to_f32(T::ZERO),
            1.0,
        ]
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgb<T>([T; 3]);

impl<T: Channel> FromRgba<u8> for Rgb<T> {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self([
            T::from_unorm8(rgba[0]),
            T::from_unorm8(rgba[1]),
            T::from_unorm8(rgba[2]),
        ])
    }
}

impl<T: Channel> ToRgba<u8> for Rgb<T> {
    fn to_rgba(self) -> [u8; 4] {
        [
            T::to_unorm8(self.0[0]),
            T::to_unorm8(self.0[1]),
            T::to_unorm8(self.0[2]),
            255u8,
        ]
    }
}

impl<T: Channel> FromRgba<f32> for Rgb<T> {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self([
            T::from_f32(rgba[0]),
            T::from_f32(rgba[1]),
            T::from_f32(rgba[2]),
        ])
    }
}

impl<T: Channel> ToRgba<f32> for Rgb<T> {
    fn to_rgba(self) -> [f32; 4] {
        [
            T::to_f32(self.0[0]),
            T::to_f32(self.0[1]),
            T::to_f32(self.0[2]),
            1.0,
        ]
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgba<T>([T; 4]);

impl<T: Channel> FromRgba<u8> for Rgba<T> {
    fn from_rgba(rgba: [u8; 4]) -> Self {
        Self(rgba.map(T::from_unorm8))
    }
}

impl<T: Channel> ToRgba<u8> for Rgba<T> {
    fn to_rgba(self) -> [u8; 4] {
        self.0.map(T::to_unorm8)
    }
}

impl<T: Channel> FromRgba<f32> for Rgba<T> {
    fn from_rgba(rgba: [f32; 4]) -> Self {
        Self(rgba.map(T::from_f32))
    }
}

impl<T: Channel> ToRgba<f32> for Rgba<T> {
    fn to_rgba(self) -> [f32; 4] {
        self.0.map(T::to_f32)
    }
}

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
    P: GetPixel + ToRgba<T>,
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
    fn decode_rgba8_invalid_length() {
        let err = |expected| {
            Err(SurfaceError::NotEnoughData {
                expected,
                actual: 0,
            })
        };
        assert_eq!(decode_rgba::<R8, u8>(1, 1, &[]), err(1));
        assert_eq!(decode_rgba::<Rg8, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgb8, u8>(1, 1, &[]), err(3));
        assert_eq!(decode_rgba::<Rgba8, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<R8Snorm, u8>(1, 1, &[]), err(1));
        assert_eq!(decode_rgba::<Rg8Snorm, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgba8Snorm, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<R16Snorm, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rg16Snorm, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgba16Snorm, u8>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<R16, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rg16, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgba16, u8>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Bgra4, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Bgr8, u8>(1, 1, &[]), err(3));
        assert_eq!(decode_rgba::<Bgra8, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rf16, u8>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgf16, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgbaf16, u8>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Rf32, u8>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgf32, u8>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Rgbf32, u8>(1, 1, &[]), err(12));
        assert_eq!(decode_rgba::<Rgbaf32, u8>(1, 1, &[]), err(16));
    }

    #[test]
    fn encode_rgba8_invalid_length() {
        let err = Err(SurfaceError::NotEnoughData {
            expected: 4,
            actual: 0,
        });
        assert_eq!(encode_rgba::<R8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgb8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R8Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg8Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba8Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R16Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg16Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba16Snorm, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Bgra4, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Bgr8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Bgra8, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rf16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgf16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbaf16, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rf32, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgf32, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbf32, u8>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbaf32, u8>(1, 1, &[]), err);
    }

    #[test]
    fn decode_rgbaf32_invalid_length() {
        let err = |expected| {
            Err(SurfaceError::NotEnoughData {
                expected,
                actual: 0,
            })
        };
        assert_eq!(decode_rgba::<R8, f32>(1, 1, &[]), err(1));
        assert_eq!(decode_rgba::<Rg8, f32>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgb8, f32>(1, 1, &[]), err(3));
        assert_eq!(decode_rgba::<Rgba8, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<R8Snorm, f32>(1, 1, &[]), err(1));
        assert_eq!(decode_rgba::<Rg8Snorm, f32>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgba8Snorm, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<R16Snorm, f32>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rg16Snorm, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgba16Snorm, f32>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<R16, f32>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rg16, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgba16, f32>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Rf16, f32>(1, 1, &[]), err(2));
        assert_eq!(decode_rgba::<Rgf16, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgbaf16, f32>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Rf32, f32>(1, 1, &[]), err(4));
        assert_eq!(decode_rgba::<Rgf32, f32>(1, 1, &[]), err(8));
        assert_eq!(decode_rgba::<Rgbf32, f32>(1, 1, &[]), err(12));
        assert_eq!(decode_rgba::<Rgbaf32, f32>(1, 1, &[]), err(16));
    }

    #[test]
    fn encode_rgbaf32_invalid_length() {
        let err = Err(SurfaceError::NotEnoughData {
            expected: 4,
            actual: 0,
        });
        assert_eq!(encode_rgba::<R8, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg8, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgb8, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba8, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R8Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg8Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba8Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R16Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg16Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba16Snorm, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<R16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rg16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgba16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rf16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgf16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbaf16, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rf32, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgf32, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbf32, f32>(1, 1, &[]), err);
        assert_eq!(encode_rgba::<Rgbaf32, f32>(1, 1, &[]), err);
    }

    #[test]
    fn r8_from_rgba8() {
        assert_eq!(vec![1], encode_rgba::<R8, u8>(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn rgba8_from_r8() {
        assert_eq!(
            vec![64, 64, 64, 255],
            decode_rgba::<R8, u8>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn r8_snorm_from_rgba8() {
        assert_eq!(
            vec![130],
            encode_rgba::<R8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn r8_snorm_from_rgbaf32() {
        assert_eq!(
            vec![129],
            encode_rgba::<R8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_snorm() {
        assert_eq!(
            vec![192, 192, 192, 255],
            decode_rgba::<R8Snorm, u8>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_r8_snorm() {
        assert_eq!(
            vec![-1.0, -1.0, -1.0, 1.0],
            decode_rgba::<R8Snorm, f32>(1, 1, &[128]).unwrap()
        );
    }

    #[test]
    fn rg8_from_rgba8() {
        assert_eq!(
            vec![1, 2],
            encode_rgba::<Rg8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgba8() {
        assert_eq!(
            vec![130, 131],
            encode_rgba::<Rg8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgbaf32() {
        assert_eq!(
            vec![129, 0],
            encode_rgba::<Rg8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg8_snorm() {
        assert_eq!(
            vec![129, 130, 128, 255],
            decode_rgba::<Rg8Snorm, u8>(1, 1, &[1, 2]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rg8_snorm() {
        assert_eq!(
            vec![-1.0, 0.0, 0.0, 1.0],
            decode_rgba::<Rg8Snorm, f32>(1, 1, &[128, 0]).unwrap()
        );
    }

    #[test]
    fn rgba8_snorm_from_rgba8() {
        assert_eq!(
            vec![130, 131, 132, 133],
            encode_rgba::<Rgba8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_snorm_from_rgbaf32() {
        assert_eq!(
            vec![129, 0, 127, 127],
            encode_rgba::<Rgba8Snorm, f32>(1, 1, &[-1.0, 0.0, 1.0, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba8_snorm() {
        assert_eq!(
            vec![129, 130, 131, 132],
            decode_rgba::<Rgba8Snorm, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_snorm() {
        assert_eq!(
            vec![-1.0, 0.0, -1.0, 1.0],
            decode_rgba::<Rgba8Snorm, f32>(1, 1, &[128, 0, 129, 127]).unwrap()
        );
    }

    #[test]
    fn bgra8_from_rgba8() {
        assert_eq!(
            vec![3, 2, 1, 4],
            encode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra8() {
        assert_eq!(
            vec![3, 2, 1, 4],
            decode_rgba::<Bgra8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgb8_from_rgba8() {
        assert_eq!(
            vec![1, 2, 3],
            encode_rgba::<Rgb8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgr8() {
        assert_eq!(
            vec![3, 2, 1, 255],
            decode_rgba::<Bgr8, u8>(1, 1, &[1, 2, 3]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgf32() {
        assert_eq!(
            vec![0, 63, 0, 255],
            decode_rgba::<Rgf32, u8>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgf32_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32]),
            encode_rgba::<Rgf32, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgf32() {
        assert_eq!(
            vec![0.0, 0.25, 0.0, 1.0],
            decode_rgba::<Rgf32, f32>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbf32() {
        assert_eq!(
            vec![0, 63, 127, 255],
            decode_rgba::<Rgbf32, u8>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32]))
                .unwrap()
        );
    }

    #[test]
    fn rgbf32_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32, 0.6f32]),
            encode_rgba::<Rgbf32, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgbf32() {
        assert_eq!(
            vec![0.0, 0.25, 0.5, 1.0],
            decode_rgba::<Rgbf32, f32>(1, 1, bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32]))
                .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rf16() {
        assert_eq!(
            vec![63, 63, 63, 255],
            decode_rgba::<Rf16, u8>(1, 1, bytemuck::cast_slice(&[f16::from_f32(0.25f32)])).unwrap()
        );
    }

    #[test]
    fn rf16_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[f16::from_f32(0.2f32)]),
            encode_rgba::<Rf16, u8>(1, 1, &[51, 27, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgba8_from_rf32() {
        assert_eq!(
            vec![63, 63, 63, 255],
            decode_rgba::<Rf32, u8>(1, 1, bytemuck::cast_slice(&[0.25f32])).unwrap()
        );
    }

    #[test]
    fn rf32_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.2f32]),
            encode_rgba::<Rf32, u8>(1, 1, &[51, 27, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgba8_from_rgbaf32() {
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
    fn rgbaf32_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32, 0.6f32, 1.0f32]),
            encode_rgba::<Rgbaf32, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgba8_from_rgf16() {
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
    fn rgf16_from_rgba8() {
        assert_eq!(
            bytemuck::cast_slice::<f16, u8>(&[f16::from_f32(0.0f32), f16::from_f32(0.2f32),]),
            encode_rgba::<Rgf16, u8>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgba8_from_rgbaf16() {
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
    fn rgbaf16_from_rgba8() {
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
    fn rgba8_from_rgba8() {
        assert_eq!(
            vec![1, 2, 3, 4],
            decode_rgba::<Rgba8, u8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_decode() {
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
    fn rgbaf32_from_rgbaf32_encode() {
        assert_eq!(
            bytemuck::cast_slice::<f32, u8>(&[1.0f32, 2.0f32, 3.0f32, 4.0f32]),
            &encode_rgba::<Rgbaf32, f32>(1, 1, &[1.0f32, 2.0f32, 3.0f32, 4.0f32]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf16() {
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
    fn rgbaf32_from_rgf16() {
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
    fn rgbaf32_from_rf32() {
        assert_eq!(
            vec![0.25, 0.25, 0.25, 1.0],
            decode_rgba::<Rf32, f32>(1, 1, bytemuck::cast_slice(&[0.25f32])).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rf16() {
        assert_eq!(
            vec![0.25, 0.25, 0.25, 1.0],
            decode_rgba::<Rf16, f32>(1, 1, bytemuck::cast_slice(&[f16::from_f32(0.25f32)]))
                .unwrap()
        );
    }

    #[test]
    fn bgra4_from_rgba8() {
        assert_eq!(
            vec![0x30, 0xCF],
            encode_rgba::<Bgra4, u8>(1, 1, &[255, 51, 0, 204]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra4() {
        assert_eq!(
            vec![255, 51, 0, 204],
            decode_rgba::<Bgra4, u8>(1, 1, &[0x30, 0xCF]).unwrap()
        );
    }

    #[test]
    fn r16_from_rgba8() {
        assert_eq!(
            vec![127, 127],
            encode_rgba::<R16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r16() {
        assert_eq!(
            vec![127, 127, 127, 255],
            decode_rgba::<R16, u8>(1, 1, &[127, 127]).unwrap()
        );
    }

    #[test]
    fn r16_from_rgbaf32() {
        assert_eq!(
            vec![255, 63],
            encode_rgba::<R16, f32>(1, 1, &[0.25, 0.75, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_r16() {
        assert_eq!(
            vec![1.0, 1.0, 1.0, 1.0],
            decode_rgba::<R16, f32>(1, 1, &[255, 255]).unwrap()
        );
    }

    #[test]
    fn r16_snorm_from_rgba8() {
        assert_eq!(
            vec![128, 255],
            encode_rgba::<R16Snorm, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r16_snorm() {
        assert_eq!(
            vec![255, 255, 255, 255],
            decode_rgba::<R16Snorm, u8>(1, 1, &[127, 127]).unwrap()
        );
    }

    #[test]
    fn r16_snorm_from_rgbaf32() {
        assert_eq!(
            vec![1, 128],
            encode_rgba::<R16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_r16_snorm() {
        assert_eq!(
            vec![-1.0, -1.0, -1.0, 1.0],
            decode_rgba::<R16Snorm, f32>(1, 1, &[1, 128]).unwrap()
        );
    }

    #[test]
    fn rg16_from_rgba8() {
        assert_eq!(
            vec![127, 127, 128, 128],
            encode_rgba::<Rg16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg16() {
        assert_eq!(
            vec![127, 128, 0, 255],
            decode_rgba::<Rg16, u8>(1, 1, &[127, 127, 128, 128]).unwrap()
        );
    }

    #[test]
    fn rg16_from_rgbaf32() {
        assert_eq!(
            vec![255, 63, 255, 191],
            encode_rgba::<Rg16, f32>(1, 1, &[0.25, 0.75, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rg16() {
        assert_eq!(
            vec![1.0, 0.0, 0.0, 1.0],
            decode_rgba::<Rg16, f32>(1, 1, &[255, 255, 0, 0]).unwrap()
        );
    }

    #[test]
    fn rg16_snorm_from_rgba8() {
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
    fn rgba8_from_rg16_snorm() {
        assert_eq!(
            vec![255, 0, 128, 255],
            decode_rgba::<Rg16Snorm, u8>(1, 1, &[127, 127, 128, 128]).unwrap()
        );
    }

    #[test]
    fn rg16_snorm_from_rgbaf32() {
        assert_eq!(
            vec![1, 128, 0, 0],
            encode_rgba::<Rg16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rg16_snorm() {
        assert_eq!(
            vec![-1.0, 1.0, 0.0, 1.0],
            decode_rgba::<Rg16Snorm, f32>(1, 1, &[1, 128, 255, 127]).unwrap()
        );
    }

    #[test]
    fn rgba16_from_rgba8() {
        assert_eq!(
            vec![127, 127, 128, 128, 129, 129, 130, 130],
            encode_rgba::<Rgba16, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba16() {
        assert_eq!(
            vec![127, 128, 129, 130],
            decode_rgba::<Rgba16, u8>(1, 1, &[127, 127, 128, 128, 129, 129, 130, 130]).unwrap()
        );
    }

    #[test]
    fn rgba16_from_rgbaf32() {
        assert_eq!(
            vec![255, 63, 255, 191, 255, 127, 255, 255],
            encode_rgba::<Rgba16, f32>(1, 1, &[0.25, 0.75, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgba16() {
        assert_eq!(
            vec![1.0, 0.0, 1.0, 0.0],
            decode_rgba::<Rgba16, f32>(1, 1, &[255, 255, 0, 0, 255, 255, 0, 0]).unwrap()
        );
    }

    #[test]
    fn rgba16_snorm_from_rgba8() {
        assert_eq!(
            vec![128, 255, 128, 0, 129, 1, 130, 2],
            encode_rgba::<Rgba16Snorm, u8>(1, 1, &[127, 128, 129, 130]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba16_snorm() {
        assert_eq!(
            vec![255, 0, 1, 2],
            decode_rgba::<Rgba16Snorm, u8>(1, 1, &[127, 127, 128, 128, 129, 129, 130, 130])
                .unwrap()
        );
    }

    #[test]
    fn rgba16_snorm_from_rgbaf32() {
        assert_eq!(
            vec![1, 128, 0, 0, 0, 64, 255, 127],
            encode_rgba::<Rgba16Snorm, f32>(1, 1, &[-1.0, 0.0, 0.5, 1.0]).unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgba16_snorm() {
        assert_eq!(
            vec![-1.0, 0.0, -1.0, 1.0],
            decode_rgba::<Rgba16Snorm, f32>(1, 1, &[1, 128, 0, 0, 0, 128, 255, 127]).unwrap()
        );
    }
}
