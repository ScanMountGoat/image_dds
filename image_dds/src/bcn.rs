use thiserror::Error;

use crate::{div_round_up, CompressionFormat, Quality};

// Not all compressed formats use 4x4 blocks.
const BLOCK_WIDTH: usize = 4;
const BLOCK_HEIGHT: usize = 4;

#[derive(Debug, Error)]
pub enum CompressSurfaceError {
    #[error("surface dimensions {width} x {height} are not valid.")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("expected surface to have at least {expected} bytes but found {actual}.")]
    NotEnoughData { expected: usize, actual: usize },

    #[error("compressing data to format {format:?} is not supported.")]
    UnsupportedFormat { format: CompressionFormat },
}

#[derive(Debug, Error)]
pub enum DecompressSurfaceError {
    #[error("surface dimensions {width} x {height} are not valid.")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("expected surface to have at least {expected} bytes but found {actual}.")]
    NotEnoughData { expected: usize, actual: usize },
}

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

trait Bcn {
    const BYTES_PER_BLOCK: usize;

    // Expect all formats to pad to RGBA even if they have fewer channels.
    // Fixing the length should reduce the amount of bounds checking.
    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK];

    fn compress_surface(
        width: u32,
        height: u32,
        rgba8_data: &[u8],
        quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError>;
}

// TODO: Make a trait for this and make the functions generic?
struct Rgba;
impl Rgba {
    const BYTES_PER_PIXEL: usize = 4;
    const BYTES_PER_BLOCK: usize = 64;
}

struct Bc1;
impl Bcn for Bc1 {
    const BYTES_PER_BLOCK: usize = 8;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc1(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc1::compress_blocks(&surface))
    }
}

struct Bc2;
impl Bcn for Bc2 {
    const BYTES_PER_BLOCK: usize = 16;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc2(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }

    fn compress_surface(
        _width: u32,
        _height: u32,
        _rgba8_data: &[u8],
        _quality: Quality,
    ) -> Result<Vec<u8>, CompressSurfaceError> {
        // TODO: Find an implementation that supports this?
        Err(CompressSurfaceError::UnsupportedFormat {
            format: CompressionFormat::Bc2,
        })
    }
}

struct Bc3;
impl Bcn for Bc3 {
    const BYTES_PER_BLOCK: usize = 16;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc3(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc3::compress_blocks(&surface))
    }
}

struct Bc4;
impl Bcn for Bc4 {
    const BYTES_PER_BLOCK: usize = 8;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
        let mut decompressed_r = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc4(
                block.as_ptr(),
                decompressed_r.as_mut_ptr(),
                (BLOCK_WIDTH) as i32,
            );
        }

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];
        for i in 0..decompressed_r.len() {
            // It's a convention in some programs display BC4 in the red channel.
            // Use grayscale instead to avoid confusing it with colored data.
            // TODO: Match how channels handled when compressing RGBA data to BC4?
            decompressed[i * Rgba::BYTES_PER_PIXEL] = decompressed_r[i];
            decompressed[i * Rgba::BYTES_PER_PIXEL + 1] = decompressed_r[i];
            decompressed[i * Rgba::BYTES_PER_PIXEL + 2] = decompressed_r[i];
            decompressed[i * Rgba::BYTES_PER_PIXEL + 3] = 255u8;
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc4::compress_blocks(&surface))
    }
}

struct Bc5;
impl Bcn for Bc5 {
    const BYTES_PER_BLOCK: usize = 8;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc5(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc5::compress_blocks(&surface))
    }
}

struct Bc6;
impl Bcn for Bc6 {
    const BYTES_PER_BLOCK: usize = 8;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        // TODO: signed vs unsigned?
        // TODO: Float vs half?
        // TODO: Should these be allowed to be converted to rgba8?
        // TODO: This doesn't return rgba8 bytes?
        // TODO: Perform a naive conversion to rgba8?
        // TODO: Also support exr or radiance hdr under feature flags?
        // exr or radiance only make sense for bc6
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc6h_half(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
                0,
            );
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        // TODO: is this handled correctly for floating point data?
        Ok(intel_tex_2::bc6h::compress_blocks(
            &quality.into(),
            &surface,
        ))
    }
}

struct Bc7;
impl Bcn for Bc7 {
    const BYTES_PER_BLOCK: usize = 16;

    fn decompress_block(block: &[u8]) -> [u8; Rgba::BYTES_PER_BLOCK] {
        let mut decompressed = [0u8; BLOCK_WIDTH * BLOCK_HEIGHT * Rgba::BYTES_PER_PIXEL];

        unsafe {
            bcndecode_sys::bcdec_bc7(
                block.as_ptr(),
                decompressed.as_mut_ptr(),
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }

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
            stride: width * Rgba::BYTES_PER_PIXEL as u32,
            data: rgba8_data,
        };

        Ok(intel_tex_2::bc7::compress_blocks(&quality.into(), &surface))
    }
}

/// Decompress the bytes in `data` to the uncompressed RGBA8 format.
pub fn rgba8_from_bcn(
    width: u32,
    height: u32,
    data: &[u8],
    format: CompressionFormat,
) -> Result<Vec<u8>, DecompressSurfaceError> {
    // TODO: Handle signed variants.
    // TODO: Handle 2 channel (BC5) and 1 channel (BC4)?
    // TODO: How to handle the zero case?
    match format {
        CompressionFormat::Bc1 => rgba8_from_bcn_inner::<Bc1>(width, height, data),
        CompressionFormat::Bc2 => rgba8_from_bcn_inner::<Bc2>(width, height, data),
        CompressionFormat::Bc3 => rgba8_from_bcn_inner::<Bc3>(width, height, data),
        CompressionFormat::Bc4 => rgba8_from_bcn_inner::<Bc4>(width, height, data),
        CompressionFormat::Bc5 => rgba8_from_bcn_inner::<Bc5>(width, height, data),
        CompressionFormat::Bc6 => rgba8_from_bcn_inner::<Bc6>(width, height, data),
        CompressionFormat::Bc7 => rgba8_from_bcn_inner::<Bc7>(width, height, data),
    }
}

// TODO: Add a separate function that handles array layers and mipmaps.
fn rgba8_from_bcn_inner<T: Bcn>(
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    // TODO: Should surface dimensions always be a multiple of the block dimensions?
    // TODO: Add an option to parallelize this using rayon?
    // Each block can be decoded independently.

    // Surface dimensions are not validated yet and may cause overflow.
    let expected_size = div_round_up(width as usize, BLOCK_WIDTH)
        .checked_mul(div_round_up(height as usize, BLOCK_HEIGHT))
        .and_then(|v| v.checked_mul(T::BYTES_PER_BLOCK))
        .ok_or(DecompressSurfaceError::InvalidDimensions { width, height })?;

    if data.len() < expected_size {
        return Err(DecompressSurfaceError::NotEnoughData {
            expected: expected_size,
            actual: data.len(),
        });
    }

    // TODO: What's the most efficient way to zero initialize the vec?
    let mut rgba = Vec::new();
    rgba.resize(width as usize * height as usize * Rgba::BYTES_PER_PIXEL, 0);

    // BCN formats lay out blocks in row-major order.
    // TODO: calculate x and y using division and mod?
    let mut block_start = 0;
    for y in (0..height).step_by(BLOCK_HEIGHT) {
        for x in (0..width).step_by(BLOCK_WIDTH) {
            // TODO: Validate lengths and enforce alignment for safety.
            let block = &data[block_start..block_start + T::BYTES_PER_BLOCK];

            let decompressed_block = T::decompress_block(block);

            // Each block is 4x4, so we need to update multiple rows.
            put_rgba_block(
                &mut rgba,
                decompressed_block,
                x as usize,
                y as usize,
                width as usize,
            );

            block_start += T::BYTES_PER_BLOCK;
        }
    }

    Ok(rgba)
}

fn put_rgba_block(
    surface: &mut [u8],
    pixels: [u8; Rgba::BYTES_PER_BLOCK],
    x: usize,
    y: usize,
    width: usize,
) {
    // Place the compressed block into the decompressed surface.
    // For most formats this will be contiguous 4x4 pixel blocks.
    // The data from each block will update 4 rows of the RGBA surface.
    // TODO: Examine the assembly for this.
    // TODO: How to handle grayscale formats like BC4?
    // This should have tunable parameters for Rgba::bytes_per_pixel.
    let bytes_per_row = BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL;

    for row in 0..BLOCK_HEIGHT {
        let surface_index = ((y + row) * width + x) * Rgba::BYTES_PER_PIXEL;
        let pixel_index = row * BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL;
        surface[surface_index..surface_index + bytes_per_row]
            .copy_from_slice(&pixels[pixel_index..pixel_index + bytes_per_row]);
    }
}

/// Compress the uncompressed RGBA8 bytes in `data` to the given `format`.
pub fn bcn_from_rgba8(
    width: u32,
    height: u32,
    data: &[u8],
    format: CompressionFormat,
    quality: Quality,
) -> Result<Vec<u8>, CompressSurfaceError> {
    // TODO: Handle signed variants.
    // TODO: Handle 2 channel (BC5) and 1 channel (BC4)?
    match format {
        CompressionFormat::Bc1 => bcn_from_rgba8_inner::<Bc1>(width, height, data, quality),
        CompressionFormat::Bc2 => bcn_from_rgba8_inner::<Bc2>(width, height, data, quality),
        CompressionFormat::Bc3 => bcn_from_rgba8_inner::<Bc3>(width, height, data, quality),
        CompressionFormat::Bc4 => bcn_from_rgba8_inner::<Bc4>(width, height, data, quality),
        CompressionFormat::Bc5 => bcn_from_rgba8_inner::<Bc5>(width, height, data, quality),
        CompressionFormat::Bc6 => bcn_from_rgba8_inner::<Bc6>(width, height, data, quality),
        CompressionFormat::Bc7 => bcn_from_rgba8_inner::<Bc7>(width, height, data, quality),
    }
}

fn bcn_from_rgba8_inner<T: Bcn>(
    width: u32,
    height: u32,
    data: &[u8],
    quality: Quality,
) -> Result<Vec<u8>, CompressSurfaceError> {
    // TODO: Should surface dimensions always be a multiple of the block dimensions?
    // TODO: How to handle the zero case?
    if width == 0 || height == 0 {
        return Err(CompressSurfaceError::InvalidDimensions { width, height });
    }

    // Surface dimensions are not validated yet and may cause overflow.
    let expected_size = div_round_up(width as usize, BLOCK_WIDTH)
        .checked_mul(div_round_up(height as usize, BLOCK_HEIGHT))
        .and_then(|v| v.checked_mul(Rgba::BYTES_PER_BLOCK))
        .ok_or(CompressSurfaceError::InvalidDimensions { width, height })?;

    if data.len() < expected_size {
        return Err(CompressSurfaceError::NotEnoughData {
            expected: expected_size,
            actual: data.len(),
        });
    }

    T::compress_surface(width, height, data, quality)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Quality;

    // TODO: Create tests for data length since we can't know what the compressed blocks should be?
    // TODO: Test edge cases and type conversions?
    // TODO: Add tests for validating the input length.
    // TODO: Will compression fail for certain pixel values (test with fuzz tests?)

    fn check_decompress_compressed_bcn<T: Bcn>(quality: Quality) {
        // Compress the data once to introduce some errors.
        let rgba = vec![64u8; 4 * 4 * Rgba::BYTES_PER_BLOCK];
        let compressed_block = bcn_from_rgba8_inner::<T>(4, 4, &rgba, quality).unwrap();
        let decompressed_block = rgba8_from_bcn_inner::<T>(4, 4, &compressed_block).unwrap();

        // Compressing and decompressing should give back the same data.
        // TODO: Is this guaranteed in general?
        let compressed_block2 =
            bcn_from_rgba8_inner::<T>(4, 4, &decompressed_block, quality).unwrap();
        let decompressed_block2 = rgba8_from_bcn_inner::<T>(4, 4, &compressed_block2).unwrap();

        assert_eq!(decompressed_block2, decompressed_block);
    }

    #[test]
    fn bc1_decompress_compressed() {
        check_decompress_compressed_bcn::<Bc1>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc1>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc1>(Quality::Slow);
    }

    #[test]
    fn bc2_decompress_compressed() {
        check_decompress_compressed_bcn::<Bc2>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc2>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc2>(Quality::Slow);
    }

    #[test]
    fn bc3_decompress_compressed() {
        check_decompress_compressed_bcn::<Bc3>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc3>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc3>(Quality::Slow);
    }

    #[test]
    fn bc4_decompress_compressed() {
        check_decompress_compressed_bcn::<Bc4>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc4>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc4>(Quality::Slow);
    }

    #[test]
    fn bc5_decompress_compressed() {
        // TODO: Account for BC5 only using the RG channels.
        check_decompress_compressed_bcn::<Bc5>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc5>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc5>(Quality::Slow);
    }

    #[test]
    fn bc6_decompress_compressed() {
        // TODO: Account for BC6h using f16 data.
        check_decompress_compressed_bcn::<Bc6>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc6>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc6>(Quality::Slow);
    }

    #[test]
    fn bc7_decompress_compressed() {
        check_decompress_compressed_bcn::<Bc7>(Quality::Fast);
        check_decompress_compressed_bcn::<Bc7>(Quality::Normal);
        check_decompress_compressed_bcn::<Bc7>(Quality::Slow);
    }
}
