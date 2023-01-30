use crate::{CompressSurfaceError, DecompressSurfaceError};

// TODO: Share code for the rgba8 methods.
pub fn decode_rgba8_from_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    let expected = expected_size(width, height, depth, 4).ok_or(
        DecompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        },
    )?;

    if data.len() >= expected {
        Ok(data.to_vec())
    } else {
        Err(DecompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn encode_rgba8_from_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, CompressSurfaceError> {
    let expected =
        expected_size(width, height, depth, 4).ok_or(CompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        })?;
    if data.len() >= expected {
        Ok(data.to_vec())
    } else {
        Err(CompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn rgba8_from_rgbaf32(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    let expected = expected_size(width, height, depth, 16).ok_or(
        DecompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        },
    )?;

    if data.len() >= expected {
        // Use expected length to ensure the slice is an integral number of floats.
        let rgba_f32: &[f32] = bytemuck::cast_slice(&data[..expected]);
        Ok(rgba_f32.iter().map(|f| (f * 255.0) as u8).collect())
    } else {
        Err(DecompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn rgbaf32_from_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, CompressSurfaceError> {
    let expected =
        expected_size(width, height, depth, 4).ok_or(CompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        })?;
    if data.len() >= expected {
        let rgba_f32: Vec<_> = data.iter().map(|u| *u as f32 / 255.0).collect();
        Ok(bytemuck::cast_slice(&rgba_f32).to_vec())
    } else {
        Err(CompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn rgba8_from_bgra8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    let expected = expected_size(width, height, depth, 4).ok_or(
        DecompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        },
    )?;

    if data.len() >= expected {
        let mut bgra = data.to_vec();
        swap_red_blue(width, height, &mut bgra);
        Ok(bgra)
    } else {
        Err(DecompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn r8_from_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, CompressSurfaceError> {
    let expected =
        expected_size(width, height, depth, 4).ok_or(CompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        })?;
    if data.len() >= expected {
        Ok(data.iter().copied().step_by(4).collect())
    } else {
        Err(CompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn rgba8_from_r8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    let expected = expected_size(width, height, depth, 1).ok_or(
        DecompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        },
    )?;

    if data.len() >= expected {
        Ok(data.iter().flat_map(|r| [*r, *r, *r, 255u8]).collect())
    } else {
        Err(DecompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

pub fn bgra8_from_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, CompressSurfaceError> {
    let expected =
        expected_size(width, height, depth, 4).ok_or(CompressSurfaceError::InvalidDimensions {
            width,
            height,
            depth,
        })?;
    if data.len() >= expected {
        let mut bgra = data.to_vec();
        swap_red_blue(width, height, &mut bgra);
        Ok(bgra)
    } else {
        Err(CompressSurfaceError::NotEnoughData {
            expected,
            actual: data.len(),
        })
    }
}

fn swap_red_blue(width: u32, height: u32, rgba: &mut [u8]) {
    for i in 0..(width as usize * height as usize) {
        // RGBA -> BGRA.
        rgba.swap(i * 4, i * 4 + 2);
    }
}

fn expected_size(width: u32, height: u32, depth: u32, bytes_per_pixel: usize) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(depth as usize)?
        .checked_mul(bytes_per_pixel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r8_from_rgba8_valid() {
        assert_eq!(vec![1], r8_from_rgba8(1, 1, 1, &[1, 2, 3, 4]).unwrap());
    }

    #[test]
    fn r8_from_rgba8_invalid() {
        let result = r8_from_rgba8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn rgba8_from_r8_valid() {
        assert_eq!(
            vec![64, 64, 64, 255],
            rgba8_from_r8(1, 1, 1, &[64]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_r8_invalid() {
        let result = rgba8_from_r8(4, 4, 1, &[64]);
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::NotEnoughData {
                expected: 16,
                actual: 1
            })
        ));
    }

    #[test]
    fn bgra8_from_rgba8_valid() {
        assert_eq!(
            vec![3, 2, 1, 4],
            bgra8_from_rgba8(1, 1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn bgra8_from_rgba8_invalid() {
        let result = bgra8_from_rgba8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn rgba8_from_bgra8_valid() {
        assert_eq!(
            vec![3, 2, 1, 4],
            rgba8_from_bgra8(1, 1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn rgba8_from_bgra8_invalid() {
        let result = rgba8_from_bgra8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::NotEnoughData {
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
                1,
                bytemuck::cast_slice(&[0.0f32, 0.25f32, 0.5f32, 1.0f32])
            )
            .unwrap()
        );
    }

    #[test]
    fn rgba8_from_rgbaf32_invalid() {
        let result = rgba8_from_rgbaf32(1, 1, 1, &[0; 15]);
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::NotEnoughData {
                expected: 16,
                actual: 15
            })
        ));
    }

    #[test]
    fn rgbaf32_from_rgba8_valid() {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&[0.0f32, 0.2f32, 0.6f32, 1.0f32]),
            rgbaf32_from_rgba8(1, 1, 1, &[0, 51, 153, 255])
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn rgbaf32_from_rgba8_invalid() {
        let result = rgbaf32_from_rgba8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn decode_rgba8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2, 3, 4],
            decode_rgba8_from_rgba8(1, 1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn decode_rgba8_from_rgba8_invalid() {
        let result = decode_rgba8_from_rgba8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn encode_rgba8_from_rgba8_valid() {
        assert_eq!(
            vec![1, 2, 3, 4],
            encode_rgba8_from_rgba8(1, 1, 1, &[1, 2, 3, 4]).unwrap()
        );
    }

    #[test]
    fn encode_rgba8_from_rgba8_invalid() {
        let result = encode_rgba8_from_rgba8(1, 1, 1, &[1, 2, 3]);
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NotEnoughData {
                expected: 4,
                actual: 3
            })
        ));
    }
}
