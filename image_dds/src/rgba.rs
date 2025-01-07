use bytemuck::{Pod, Zeroable};
use half::f16;

use crate::{snorm_to_unorm, unorm_to_snorm, SurfaceError};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgba8([u8; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgbaf16([f16; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rgbaf32([f32; 4]);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R8(u8);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct R8Snorm(u8);

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

// TODO: make this generic over the pixel type for better f32 conversions.
pub trait Pixel {
    const SIZE: usize;

    fn to_rgba8(self) -> [u8; 4];

    fn from_rgba8(rgba: [u8; 4]) -> Self;

    fn get_pixel(data: &[u8], index: usize) -> Self;
}

fn get_pixel<const N: usize>(data: &[u8], index: usize, size: usize) -> [u8; N] {
    data[index * size..index * size + size].try_into().unwrap()
}

impl Pixel for Rgba8 {
    const SIZE: usize = 4;

    fn to_rgba8(self) -> [u8; 4] {
        self.0
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self(rgba)
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Rgbaf16 {
    const SIZE: usize = 8;

    fn to_rgba8(self) -> [u8; 4] {
        self.0.map(|f| (f.to_f32() * 255.0) as u8)
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self(rgba.map(|u| f16::from_f32(u as f32 / 255.0)))
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        // TODO: Implement this automatically?
        // TODO: Don't assume endianness.
        Self(
            bytemuck::cast_slice(&get_pixel::<8>(data, index, Self::SIZE))
                .try_into()
                .unwrap(),
        )
    }
}

impl Pixel for Rgbaf32 {
    const SIZE: usize = 16;

    fn to_rgba8(self) -> [u8; 4] {
        self.0.map(|f| (f * 255.0) as u8)
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self(rgba.map(|u| u as f32 / 255.0))
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        // TODO: Implement this automatically?
        // TODO: Don't assume endianness.
        Self(
            bytemuck::cast_slice(&get_pixel::<16>(data, index, Self::SIZE))
                .try_into()
                .unwrap(),
        )
    }
}

impl Pixel for R8 {
    const SIZE: usize = 1;

    fn to_rgba8(self) -> [u8; 4] {
        [self.0, self.0, self.0, 255u8]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self(rgba[0])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(data[index])
    }
}

impl Pixel for R8Snorm {
    const SIZE: usize = 1;

    fn to_rgba8(self) -> [u8; 4] {
        let r = snorm_to_unorm(self.0);
        [r, r, r, 255u8]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self(unorm_to_snorm(rgba[0]))
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(data[index])
    }
}

impl Pixel for Rg8 {
    const SIZE: usize = 2;

    fn to_rgba8(self) -> [u8; 4] {
        [self.0[0], self.0[1], 0, 255u8]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self([rgba[0], rgba[1]])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Rg8Snorm {
    const SIZE: usize = 2;

    fn to_rgba8(self) -> [u8; 4] {
        [
            snorm_to_unorm(self.0[0]),
            snorm_to_unorm(self.0[1]),
            0,
            255u8,
        ]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self([unorm_to_snorm(rgba[0]), unorm_to_snorm(rgba[1])])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Rgb8 {
    const SIZE: usize = 3;

    fn to_rgba8(self) -> [u8; 4] {
        [self.0[0], self.0[1], self.0[2], 255u8]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self([rgba[0], rgba[1], rgba[2]])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Bgr8 {
    const SIZE: usize = 3;

    fn to_rgba8(self) -> [u8; 4] {
        [self.0[2], self.0[1], self.0[0], 255u8]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self([rgba[2], rgba[1], rgba[0]])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Bgra8 {
    const SIZE: usize = 4;

    fn to_rgba8(self) -> [u8; 4] {
        [self.0[2], self.0[1], self.0[0], self.0[3]]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        Self([rgba[2], rgba[1], rgba[0], rgba[3]])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

impl Pixel for Bgra4 {
    const SIZE: usize = 2;

    fn to_rgba8(self) -> [u8; 4] {
        // TODO: How to implement this efficiently?
        // Expand 4 bit input channels to 8 bit output channels.
        // Most significant bit -> ARGB -> least significant bit.
        [
            (self.0[1] & 0xF) * 17,
            (self.0[0] >> 4) * 17,
            (self.0[0] & 0xF) * 17,
            (self.0[1] >> 4) * 17,
        ]
    }

    fn from_rgba8(rgba: [u8; 4]) -> Self {
        // TODO: How to implement this efficiently?
        // Pack each channel into 4 bits.
        // Most significant bit -> ARGB -> least significant bit.
        Self([
            ((rgba[1] / 17) << 4) | (rgba[2] / 17),
            ((rgba[3] / 17) << 4) | (rgba[0] / 17),
        ])
    }

    fn get_pixel(data: &[u8], index: usize) -> Self {
        Self(get_pixel(data, index, Self::SIZE))
    }
}

pub fn rgba8_from_pixels<P: Pixel>(
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, P::SIZE, data)?;
    Ok((0..width * height)
        .map(|i| P::get_pixel(data, i as usize).to_rgba8())
        .flatten()
        .collect())
}

pub fn pixels_from_rgba8<P: Pixel + Pod>(
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;
    // TODO: Find a better way to convert to bytes.
    Ok(bytemuck::cast_slice(
        &(0..width * height)
            .map(|i| P::from_rgba8(Rgba8::get_pixel(data, i as usize).0))
            .collect::<Vec<_>>(),
    )
    .to_vec())
}

pub fn rgbaf32_from_rgbaf32(
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<f32>, SurfaceError> {
    validate_length(width, height, 16, data)?;
    Ok(bytemuck::cast_slice(data).to_vec())
}

pub fn rgbaf32_from_rgbaf16(
    width: u32,
    height: u32,

    data: &[u8],
) -> Result<Vec<f32>, SurfaceError> {
    let expected = validate_length(width, height, 8, data)?;

    // Use expected length to ensure the slice is an integral number of floats.
    let rgba_f16: &[f16] = bytemuck::cast_slice(&data[..expected]);
    Ok(rgba_f16.iter().copied().map(f16::to_f32).collect())
}

pub fn rgbaf16_from_rgbaf32(
    width: u32,
    height: u32,

    data: &[u8],
) -> Result<Vec<f16>, SurfaceError> {
    let expected = validate_length(width, height, 16, data)?;

    // Use expected length to ensure the slice is an integral number of floats.
    let rgba_f32: &[f32] = bytemuck::cast_slice(&data[..expected]);
    Ok(rgba_f32.iter().copied().map(f16::from_f32).collect())
}

fn validate_length(
    width: u32,
    height: u32,

    bytes_per_pixel: usize,
    data: &[u8],
) -> Result<usize, SurfaceError> {
    let expected = expected_size(width, height, bytes_per_pixel).ok_or(
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
        assert_eq!(
            vec![1],
            pixels_from_rgba8::<R8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn r8_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<R8>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<R8>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_invalid() {
        let result = rgba8_from_pixels::<R8>(4, 4, &[64]);
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
            pixels_from_rgba8::<R8Snorm>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn r8_snorm_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<R8Snorm>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<R8Snorm>(1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_snorm_invalid() {
        let result = rgba8_from_pixels::<R8Snorm>(4, 4, &[64]);
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
            pixels_from_rgba8::<Rg8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Rg8>(1, 1, &[1, 2, 3]);
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
            pixels_from_rgba8::<Rg8Snorm>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Rg8Snorm>(1, 1, &[1, 2, 3]);
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
            vec![129, 130, 0, 255],
            rgba8_from_pixels::<Rg8Snorm>(1, 1, &[1, 2]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rg8_snorm_invalid() {
        let result = rgba8_from_pixels::<Rg8Snorm>(1, 1, &[64]);
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
            pixels_from_rgba8::<Bgra8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn bgra8_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Bgra8>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<Bgra8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra8_invalid() {
        let result = rgba8_from_pixels::<Bgra8>(1, 1, &[1, 2, 3]);
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
            pixels_from_rgba8::<Rgb8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgb8_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Rgb8>(1, 1, &[1, 2]);
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
            rgba8_from_pixels::<Bgr8>(1, 1, &[1, 2, 3]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgr8_invalid() {
        let result = rgba8_from_pixels::<Bgr8>(1, 1, &[1, 2]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 3,
                actual: 2
            })
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_valid() {
        assert_eq!(
            vec![0, 63, 127, 255],
            rgba8_from_pixels::<Rgbaf32>(
                1,
                1,
                bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32, 1.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_invalid() {
        let result = rgba8_from_pixels::<Rgbaf32>(1, 1, &[0; 15]);
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
            pixels_from_rgba8::<Rgbaf32>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Rgbaf32>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<Rgbaf16>(
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
        let result = rgba8_from_pixels::<Rgbaf16>(1, 1, &[0; 7]);
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
            pixels_from_rgba8::<Rgbaf16>(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf16_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Rgbaf16>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<Rgba8>(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba8_invalid() {
        let result = rgba8_from_pixels::<Rgba8>(1, 1, &[1, 2, 3]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_valid() {
        assert_eq!(
            vec![1.0, 2.0, 3.0, 4.0],
            rgbaf32_from_rgbaf32(
                1,
                1,
                bytemuck::cast_slice(&[1.0f32, 2.0f32, 3.0f32, 4.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf32_invalid() {
        let result = rgbaf32_from_rgbaf32(1, 1, &[0; 15]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        );
    }

    #[test]
    fn rgbaf32_from_rgbaf16_valid() {
        assert_eq!(
            vec![0.0, 0.25, 0.5, 1.0],
            rgbaf32_from_rgbaf16(
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
        let result = rgbaf32_from_rgbaf16(1, 1, &[0; 7]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        );
    }

    #[test]
    fn bgra4_from_rgba8_valid() {
        assert_eq!(
            vec![0x30, 0xCF],
            pixels_from_rgba8::<Bgra4>(1, 1, &[255, 51, 0, 204]).unwrap()
        );
    }

    #[test]
    fn bgra4_from_rgba8_invalid() {
        let result = pixels_from_rgba8::<Bgra4>(1, 1, &[1, 2, 3]);
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
            rgba8_from_pixels::<Bgra4>(1, 1, &[0x30, 0xCF]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra4_invalid() {
        let result = rgba8_from_pixels::<Bgra4>(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }
}
