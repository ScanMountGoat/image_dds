use crate::{mip_size, CompressSurfaceError, ImageFormat, Quality};
use half::f16;

use super::{Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7, Rgba8, BLOCK_HEIGHT, BLOCK_WIDTH};

// Quality modes are optimized for a balance of speed and quality.
impl From<Quality> for intel_tex_2::bc6h::EncodeSettings {
    fn from(value: Quality) -> Self {
        // TODO: Test quality settings and speed for bc6h.
        match value {
            Quality::Fast => intel_tex_2::bc6h::very_fast_settings(),
            Quality::Normal => intel_tex_2::bc6h::basic_settings(),
            Quality::Slow => intel_tex_2::bc6h::slow_settings(),
        }
    }
}

impl From<Quality> for intel_tex_2::bc7::EncodeSettings {
    fn from(value: Quality) -> Self {
        // bc7 has almost imperceptible errors even at ultra_fast
        // 4k rgba ultra fast (2s), very fast (7s), fast (12s)
        match value {
            Quality::Fast => intel_tex_2::bc7::alpha_ultra_fast_settings(),
            Quality::Normal => intel_tex_2::bc7::alpha_very_fast_settings(),
            Quality::Slow => intel_tex_2::bc7::alpha_fast_settings(),
        }
    }
}

pub trait BcnEncode<Pixel> {
    // TODO: Should this take &[Pixel] instead of &[u8]?
    // TODO: How to handle depth with intel-tex-rs-2?
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError>;
}

impl BcnEncode<[u8; 4]> for Bc1 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        _: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // RGBA with 4 bytes per pixel.
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * Rgba8::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc1::compress_blocks(&surface))
    }
}

impl BcnEncode<[u8; 4]> for Bc2 {
    fn compress_surface(
        _width: u32,
        _height: u32,
        _rgba8_data: &[u8],
        _quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // TODO: Find an implementation that supports this?
        Err(CompressSurfaceError::UnsupportedFormat {
            format: ImageFormat::BC2Unorm,
        })
    }
}

impl BcnEncode<[u8; 4]> for Bc3 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        _: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // RGBA with 4 bytes per pixel.
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * Rgba8::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc3::compress_blocks(&surface))
    }
}

impl BcnEncode<[u8; 4]> for Bc4 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        _: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // RGBA with 4 bytes per pixel.
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * Rgba8::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc4::compress_blocks(&surface))
    }
}

impl BcnEncode<[u8; 4]> for Bc5 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        _: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // RGBA with 4 bytes per pixel.
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * Rgba8::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc5::compress_blocks(&surface))
    }
}

impl BcnEncode<[u8; 4]> for Bc6 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // The BC6H encoder expects the data to be in half precision floating point.
        // This differs from the other formats that expect [u8; 4] for each pixel.
        let f16_data: Vec<f16> = rgba8_data
            .iter()
            .map(|v| half::f16::from_f32(*v as f32 / 255.0))
            .collect();

        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * 4 * std::mem::size_of::<f16>() as u32,
            data: bytemuck::cast_slice(&f16_data),
        };

        Ok(intel_tex_2::bc6h::compress_blocks(
            &quality.into(),
            &surface,
        ))
    }
}

impl BcnEncode<[u8; 4]> for Bc7 {
    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // RGBA with 4 bytes per pixel.
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * Rgba8::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc7::compress_blocks(&quality.into(), &surface))
    }
}

/// Compress the uncompressed RGBA8 bytes in `data` to the given format `T`.
pub fn bcn_from_rgba8<T: BcnEncode<[u8; 4]>>(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
    quality: Quality,
) -> Result<Vec<u8>, CompressSurfaceError> {
    // Surface dimensions are not validated yet and may cause overflow.
    let expected_size = mip_size(
        width as usize,
        height as usize,
        depth as usize,
        BLOCK_WIDTH,
        BLOCK_HEIGHT,
        1,
        Rgba8::BYTES_PER_BLOCK,
    )
    .ok_or(CompressSurfaceError::PixelCountWouldOverflow {
        width,
        height,
        depth,
    })?;

    // The surface must be a multiple of the block dimensions for safety.
    if data.len() < expected_size {
        return Err(CompressSurfaceError::NotEnoughData {
            expected: expected_size,
            actual: data.len(),
        });
    }

    T::compress_surface(width, height, data, quality)
}

// TODO: Rework these tests.
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{bcn::encode::bcn_from_rgba8, bcn::Rgba8, Quality};

    // TODO: Create tests for data length since we can't know what the compressed blocks should be?
    // TODO: Test edge cases and type conversions?
    // TODO: Add tests for validating the input length.
    // TODO: Will compression fail for certain pixel values (test with fuzz tests?)

    fn check_compress_bcn<T: BcnEncode<[u8; 4]>>(rgba: &[u8], quality: Quality) {
        bcn_from_rgba8::<T>(4, 4, 1, &rgba, quality).unwrap();
    }

    #[test]
    fn bc1_compress() {
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc1>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc1>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc1>(&rgba, Quality::Slow);
    }

    #[test]
    #[ignore]
    fn bc2_compress() {
        // TODO: Revise this test to check each direction separately.
        // TODO: BC2 compression should return an error.
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc2>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc2>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc2>(&rgba, Quality::Slow);
    }

    #[test]
    fn bc3_compress() {
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc3>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc3>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc3>(&rgba, Quality::Slow);
    }

    #[test]
    fn bc4_compress() {
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc4>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc4>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc4>(&rgba, Quality::Slow);
    }

    #[test]
    fn bc5_compress() {
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc5>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc5>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc5>(&rgba, Quality::Slow);
    }

    #[test]
    #[ignore]
    fn bc6_compress() {
        // TODO: Revise this test to check each direction separately.
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc6>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc6>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc6>(&rgba, Quality::Slow);
    }

    #[test]
    fn bc7_compress() {
        let rgba = vec![64u8; Rgba8::BYTES_PER_BLOCK];
        check_compress_bcn::<Bc7>(&rgba, Quality::Fast);
        check_compress_bcn::<Bc7>(&rgba, Quality::Normal);
        check_compress_bcn::<Bc7>(&rgba, Quality::Slow);
    }
}
