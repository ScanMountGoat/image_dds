use half::f16;

use crate::{snorm_to_unorm, unorm_to_snorm, SurfaceError};

pub fn rgba8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;
    Ok(data.to_vec())
}

pub fn rgba8_from_rgbaf32(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    let expected = validate_length(width, height, 16, data)?;

    // Use expected length to ensure the slice is an integral number of floats.
    let rgba_f32: &[f32] = bytemuck::cast_slice(&data[..expected]);
    Ok(rgba_f32.iter().map(|f| (f * 255.0) as u8).collect())
}

pub fn rgba8_from_rgbaf16(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    let expected = validate_length(width, height, 8, data)?;

    // Use expected length to ensure the slice is an integral number of floats.
    let rgba_f16: &[f16] = bytemuck::cast_slice(&data[..expected]);
    Ok(rgba_f16
        .iter()
        .map(|f| (f.to_f32() * 255.0) as u8)
        .collect())
}

pub fn rgbaf32_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let rgba_f32: Vec<_> = data.iter().map(|u| *u as f32 / 255.0).collect();
    Ok(bytemuck::cast_slice(&rgba_f32).to_vec())
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

pub fn rgbaf16_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let rgba_f16: Vec<_> = data
        .iter()
        .map(|u| f16::from_f32(*u as f32 / 255.0))
        .collect();
    Ok(bytemuck::cast_slice(&rgba_f16).to_vec())
}

pub fn rgba8_from_bgra8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let mut bgra = data.to_vec();
    swap_red_blue_rgba(width, height, &mut bgra);
    Ok(bgra)
}

pub fn r8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let mut r = vec![0u8; width as usize * height as usize];
    for i in 0..r.len() {
        r[i] = data[i * 4];
    }
    Ok(r)
}

pub fn r8_snorm_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let mut r = vec![0u8; width as usize * height as usize];
    for i in 0..r.len() {
        r[i] = unorm_to_snorm(data[i * 4]);
    }
    Ok(r)
}

pub fn rgba8_from_bgr8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 3, data)?;

    let pixel_count = width as usize * height as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        rgba[i * 4] = data[i * 3 + 2];
        rgba[i * 4 + 1] = data[i * 3 + 1];
        rgba[i * 4 + 2] = data[i * 3];
        rgba[i * 4 + 3] = 255u8;
    }
    Ok(rgba)
}

pub fn rgba8_from_r8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 1, data)?;
    Ok(data.iter().flat_map(|r| [*r, *r, *r, 255u8]).collect())
}

pub fn rgba8_from_r8_snorm(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 1, data)?;
    Ok(data
        .iter()
        .flat_map(|r| {
            let r = snorm_to_unorm(*r);
            [r, r, r, 255u8]
        })
        .collect())
}

pub fn rgba8_from_rg8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 2, data)?;

    let pixel_count = width as usize * height as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        rgba[i * 4] = data[i * 2];
        rgba[i * 4 + 1] = data[i * 2 + 1];
        rgba[i * 4 + 3] = 255u8;
    }
    Ok(rgba)
}

pub fn rgba8_from_rg8_snorm(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 2, data)?;

    let pixel_count = width as usize * height as usize;
    let mut rgba = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        rgba[i * 4] = snorm_to_unorm(data[i * 2]);
        rgba[i * 4 + 1] = snorm_to_unorm(data[i * 2 + 1]);
        rgba[i * 4 + 2] = 128u8;
        rgba[i * 4 + 3] = 255u8;
    }
    Ok(rgba)
}

pub fn rg8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let pixel_count = width as usize * height as usize;
    let mut rg = vec![0u8; pixel_count * 2];
    for i in 0..pixel_count {
        rg[i * 2] = data[i * 4];
        rg[i * 2 + 1] = data[i * 4 + 1];
    }
    Ok(rg)
}

pub fn rg8_snorm_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let pixel_count = width as usize * height as usize;
    let mut rg = vec![0u8; pixel_count * 2];
    for i in 0..pixel_count {
        rg[i * 2] = unorm_to_snorm(data[i * 4]);
        rg[i * 2 + 1] = unorm_to_snorm(data[i * 4 + 1]);
    }
    Ok(rg)
}

pub fn bgra8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let mut bgra = data.to_vec();
    swap_red_blue_rgba(width, height, &mut bgra);
    Ok(bgra)
}

pub fn bgr8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let pixel_count = width as usize * height as usize;
    let mut bgr = vec![0u8; pixel_count * 3];
    for i in 0..pixel_count {
        bgr[i * 3] = data[i * 4 + 2];
        bgr[i * 3 + 1] = data[i * 4 + 1];
        bgr[i * 3 + 2] = data[i * 4];
    }
    Ok(bgr)
}

pub fn rgba8_from_bgra4(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 2, data)?;

    // TODO: How to implement this efficiently?
    // Expand 4 bit input channels to 8 bit output channels.
    // Most significant bit -> ARGB -> least significant bit.
    let rgba = data
        .chunks_exact(2)
        .flat_map(|c| {
            [
                (c[1] & 0xF) * 17,
                (c[0] >> 4) * 17,
                (c[0] & 0xF) * 17,
                (c[1] >> 4) * 17,
            ]
        })
        .collect();
    Ok(rgba)
}

pub fn bgra4_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    // TODO: How to implement this efficiently?
    // Pack each channel into 4 bits.
    // Most significant bit -> ARGB -> least significant bit.
    let bgra = data
        .chunks_exact(4)
        .flat_map(|c| {
            [
                ((c[1] / 17) << 4) | (c[2] / 17),
                ((c[3] / 17) << 4) | (c[0] / 17),
            ]
        })
        .collect();
    Ok(bgra)
}

fn swap_red_blue_rgba(width: u32, height: u32, rgba: &mut [u8]) {
    for i in 0..(width as usize * height as usize) {
        // RGBA -> BGRA.
        rgba.swap(i * 4, i * 4 + 2);
    }
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
        assert_eq!(vec![1], r8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn r8_from_rgba8_invalid() {
        let result = r8_from_rgba8(1, 1, &[1, 2, 3]);
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
        assert_eq!(vec![64, 64, 64, 255], rgba8_from_r8(1, 1, &[64]).unwrap());
    }

    #[test]
    fn rgba8_from_r8_invalid() {
        let result = rgba8_from_r8(4, 4, &[64]);
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
        assert_eq!(vec![130], r8_snorm_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn r8_snorm_from_rgba8_invalid() {
        let result = r8_snorm_from_rgba8(1, 1, &[1, 2, 3]);
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
        assert_eq!(vec![64, 64, 64, 255], rgba8_from_r8(1, 1, &[64]).unwrap());
    }

    #[test]
    fn rgba8_from_r8_snorm_invalid() {
        let result = rgba8_from_r8(4, 4, &[64]);
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
        assert_eq!(vec![1, 2], rg8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn rg8_from_rgba8_invalid() {
        let result = rg8_from_rgba8(1, 1, &[1, 2, 3]);
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
            rg8_snorm_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rg8_snorm_from_rgba8_invalid() {
        let result = rg8_snorm_from_rgba8(1, 1, &[1, 2, 3]);
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
        assert_eq!(vec![1, 2, 0, 255], rgba8_from_rg8(1, 1, &[1, 2]).unwrap());
    }

    #[test]
    fn rgba8_from_rg8_snorm_invalid() {
        let result = rgba8_from_rg8(1, 1, &[64]);
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
            bgra8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn bgra8_from_rgba8_invalid() {
        let result = bgra8_from_rgba8(1, 1, &[1, 2, 3]);
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
            rgba8_from_bgra8(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra8_invalid() {
        let result = rgba8_from_bgra8(1, 1, &[1, 2, 3]);
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
        assert_eq!(vec![3, 2, 1], bgr8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn rgb8_from_rgba8_invalid() {
        let result = bgr8_from_rgba8(1, 1, &[1, 2]);
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
            rgba8_from_bgr8(1, 1, &[1, 2, 3]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgr8_invalid() {
        let result = rgba8_from_bgr8(1, 1, &[1, 2]);
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
            rgba8_from_rgbaf32(
                1,
                1,
                bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32, 1.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_invalid() {
        let result = rgba8_from_rgbaf32(1, 1, &[0; 15]);
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
            rgbaf32_from_rgba8(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_invalid() {
        let result = rgbaf32_from_rgba8(1, 1, &[1, 2, 3]);
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
            rgba8_from_rgbaf16(
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
        let result = rgba8_from_rgbaf16(1, 1, &[0; 7]);
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
            rgbaf16_from_rgba8(1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf16_from_rgba8_invalid() {
        let result = rgbaf16_from_rgba8(1, 1, &[1, 2, 3]);
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
            rgba8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgba8_invalid() {
        let result = rgba8_from_rgba8(1, 1, &[1, 2, 3]);
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
            bgra4_from_rgba8(1, 1, &[255, 51, 0, 204]).unwrap()
        );
    }

    #[test]
    fn bgra4_from_rgba8_invalid() {
        let result = bgra4_from_rgba8(1, 1, &[1, 2, 3]);
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
            rgba8_from_bgra4(1, 1, &[0x30, 0xCF]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra4_invalid() {
        let result = rgba8_from_bgra4(1, 1, &[1]);
        assert_eq!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 2,
                actual: 1
            })
        );
    }
}
