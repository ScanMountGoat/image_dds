use half::f16;

use crate::SurfaceError;

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
    swap_red_blue(width, height, &mut bgra);
    Ok(bgra)
}

pub fn r8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;
    Ok(data.iter().copied().step_by(4).collect())
}

pub fn rgba8_from_r8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 1, data)?;
    Ok(data.iter().flat_map(|r| [*r, *r, *r, 255u8]).collect())
}

pub fn bgra8_from_rgba8(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>, SurfaceError> {
    validate_length(width, height, 4, data)?;

    let mut bgra = data.to_vec();
    swap_red_blue(width, height, &mut bgra);
    Ok(bgra)
}

fn swap_red_blue(width: u32, height: u32, rgba: &mut [u8]) {
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn rgba8_from_r8_valid() {
        assert_eq!(vec![64, 64, 64, 255], rgba8_from_r8(1, 1, &[64]).unwrap());
    }

    #[test]
    fn rgba8_from_r8_invalid() {
        let result = rgba8_from_r8(4, 4, &[64]);
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 1
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        ));
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
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn encode_rgba8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2, 3, 4],
            rgba8_from_rgba8(1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn encode_rgba8_from_rgba8_invalid() {
        let result = rgba8_from_rgba8(1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn encode_rgbaf32_from_rgbaf32_valid() {
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
    fn encode_rgbaf32_from_rgbaf32_invalid() {
        let result = rgbaf32_from_rgbaf32(1, 1, &[0; 15]);
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        ));
    }

    #[test]
    fn encode_rgbaf32_from_rgbaf16_valid() {
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
    fn encode_rgbaf32_from_rgbaf16_invalid() {
        let result = rgbaf32_from_rgbaf16(1, 1, &[0; 7]);
        assert!(matches!(
            result,
            Err(SurfaceError::NotEnoughData {
                expected: 8,
                actual: 7
            })
        ));
    }
}
