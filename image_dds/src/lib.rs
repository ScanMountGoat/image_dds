use bcn::{CompressSurfaceError, DecompressSurfaceError};
use thiserror::Error;

// TODO: Module level documentation explaining limitations and showing basic usage.

// TODO: pub use some of the functions?
pub mod bcn;

// TODO: Document that this is only available on certain features?
#[cfg(feature = "ddsfile")]
mod dds;
#[cfg(feature = "ddsfile")]
pub use dds::*;

/// The conversion quality when converting to compressed formats.
///
/// Higher quality settings run significantly slower.
/// Block compressed formats like BC7 use a fixed compression ratio,
/// so lower quality settings do not use less space than slower ones.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    /// Faster exports with slightly lower quality.
    Fast,
    /// Normal export speed and quality.
    Normal,
    /// Slower exports for slightly higher quality.
    Slow,
}

// TODO: Nested enums to handle uncompressed and compressed?

// TODO: Add "decoders" for uncompressed formats as well in an uncompressed module.
// Each format should have conversions to and from rgba8 and rgbaf32 for convenience.
// Document the channels and bit depths for each format (i.e bc6 is half precision float, bc7 is rgba8, etc).
// TODO: Document that not all DDS formats are supported.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageFormat {
    BC1Unorm,
    BC1Srgb,
    BC2Unorm,
    BC2Srgb,
    BC3Unorm,
    BC3Srgb,
    BC4Unorm,
    BC4Snorm,
    BC5Unorm,
    BC5Snorm,
    BC6Ufloat,
    BC6Sfloat,
    BC7Unorm,
    BC7Srgb,
}

impl ImageFormat {
    // TODO: Is it worth making these public?
    fn block_width(&self) -> u32 {
        match self {
            ImageFormat::BC1Unorm => 4,
            ImageFormat::BC1Srgb => 4,
            ImageFormat::BC2Unorm => 4,
            ImageFormat::BC2Srgb => 4,
            ImageFormat::BC3Unorm => 4,
            ImageFormat::BC3Srgb => 4,
            ImageFormat::BC4Unorm => 4,
            ImageFormat::BC4Snorm => 4,
            ImageFormat::BC5Unorm => 4,
            ImageFormat::BC5Snorm => 4,
            ImageFormat::BC6Ufloat => 4,
            ImageFormat::BC6Sfloat => 4,
            ImageFormat::BC7Unorm => 4,
            ImageFormat::BC7Srgb => 4,
        }
    }

    fn block_height(&self) -> u32 {
        match self {
            ImageFormat::BC1Unorm => 4,
            ImageFormat::BC1Srgb => 4,
            ImageFormat::BC2Unorm => 4,
            ImageFormat::BC2Srgb => 4,
            ImageFormat::BC3Unorm => 4,
            ImageFormat::BC3Srgb => 4,
            ImageFormat::BC4Unorm => 4,
            ImageFormat::BC4Snorm => 4,
            ImageFormat::BC5Unorm => 4,
            ImageFormat::BC5Snorm => 4,
            ImageFormat::BC6Ufloat => 4,
            ImageFormat::BC6Sfloat => 4,
            ImageFormat::BC7Unorm => 4,
            ImageFormat::BC7Srgb => 4,
        }
    }
}

#[derive(Debug, Error)]
pub enum CreateImageError {
    #[error("data length {data_length} is not valid for a {width}x{height} image")]
    InvalidSurfaceDimensions {
        width: u32,
        height: u32,
        data_length: usize,
    },

    #[error("error decompressing surface")]
    DecompressSurface(#[from] DecompressSurfaceError),
}

fn max_mipmap_count(max_dimension: u32) -> u32 {
    // log2(x) + 1
    u32::BITS - max_dimension.leading_zeros()
}

/// Decodes a surface of dimensions `width` x `height` with the given `format` to RGBA8.
pub fn decode_surface_rgba8(width: u32, height: u32, data: &[u8], format: ImageFormat) -> Result<Vec<u8>, DecompressSurfaceError> {
    // TODO: This won't always be BCN?
    // TODO: Move the match on format into this function?
    // TODO: Make it possible to decode/encode a format known at compile time?
    crate::bcn::rgba8_from_bcn(width, height, data, format.into())
}

// TODO: Use an enum for mipmaps that could use tightly packed mipmaps.
// TODO: Add an option for depth or array layers.
// TODO: Add documentation showing how to use this.
/// Encodes an RGBA8 surface of dimensions `width` x `height` to the given `format`.
///
/// Mipmaps are automatically generated when `generate_mipmaps` is `true`.
/// The `rgba8_data` only needs to contain enough data for the base mip level of `width` x `height` pixels.
pub fn encode_surface_rgba8_generated_mipmaps(
    width: u32,
    height: u32,
    rgba8_data: &[u8],
    format: ImageFormat,
    quality: Quality,
    generate_mipmaps: bool,
) -> Result<Vec<u8>, CompressSurfaceError> {
    // The width and height must be a multiple of the block dimensions.
    // This only applies to the base level.
    let block_width = format.block_width();
    let block_height = format.block_height();
    if width % block_width != 0 || height % block_height != 0 {
        return Err(CompressSurfaceError::NonIntegralDimensionsInBlocks {
            width,
            height,
            block_width,
            block_height,
        });
    }

    let num_mipmaps = if generate_mipmaps {
        max_mipmap_count(width.max(height))
    } else {
        1
    };

    let mut surface_data = Vec::new();

    let mut mip_image = rgba8_data.to_vec();

    let compression_format = format.into();

    for i in 0..num_mipmaps {
        let mip_width = (width >> i).max(1);
        let mip_height = (height >> i).max(1);

        // TODO: This function should depend on the format.
        // The physical size must be at least 4x4 to have enough data for a full block.
        // Applications or the GPU will use the smaller virtual size and ignore padding.
        // https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression
        let mip_data = bcn::bcn_from_rgba8(
            mip_width.max(4),
            mip_height.max(4),
            &mip_image,
            compression_format,
            quality,
        )?;
        surface_data.extend_from_slice(&mip_data);

        // Halve the width and height for the next mipmap.
        // TODO: Find a better way to pad the size.
        if mip_width > 4 && mip_height > 4 {
            mip_image = downsample_rgba8(mip_width, mip_height, &mip_image);
        }
    }

    Ok(surface_data)
}

fn downsample_rgba8(width: u32, height: u32, data: &[u8]) -> Vec<u8> {
    // Halve the width and height by averaging pixels.
    // This is faster than resizing using the image crate.
    // TODO: Handle the case where the dimensions aren't even.
    let width = width as usize;
    let height = height as usize;

    let new_width = width / 2;
    let new_height = height / 2;

    let mut new_data = vec![0u8; new_width * new_height * 4];
    for x in 0..new_width {
        for y in 0..new_height {
            let new_index = y * new_width + x;

            // Average a 4x4 pixel region from data.
            let top_left = y * 2 * width + x * 2;
            let top_right = y * 2 * width + ((x * 2) + 1);
            let bottom_left = ((y * 2) + 1) * width + x * 2;
            let bottom_right = ((y * 2) + 1) * width + (x * 2) + 1;

            for c in 0..4 {
                let average = (data[top_left * 4 + c] as f32
                    + data[top_right * 4 + c] as f32
                    + data[bottom_left * 4 + c] as f32
                    + data[bottom_right * 4 + c] as f32)
                    / 4.0;
                new_data[new_index * 4 + c] = average as u8;
            }
        }
    }
    new_data
}

fn div_round_up(x: usize, d: usize) -> usize {
    (x + d - 1) / d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_mipmap_count_zero() {
        assert_eq!(0, max_mipmap_count(0));
    }

    #[test]
    fn max_mipmap_count_1() {
        assert_eq!(1, max_mipmap_count(1));
    }

    #[test]
    fn max_mipmap_count_4() {
        assert_eq!(4, max_mipmap_count(12));
    }

    #[test]
    fn downsample_rgba8_4x4() {
        // Test that a checkerboard is averaged.
        let original: Vec<_> = std::iter::repeat([0u8, 0u8, 0u8, 0u8, 255u8, 255u8, 255u8, 255u8])
            .take(4 * 4 / 2)
            .flatten()
            .collect();
        assert_eq!(vec![127u8; 2 * 2 * 4], downsample_rgba8(4, 4, &original));
    }

    #[test]
    fn downsample_rgba8_3x3() {
        // Test that a checkerboard is averaged.
        let original: Vec<_> = std::iter::repeat([
            0u8, 0u8, 0u8, 0u8, 255u8, 255u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8,
        ])
        .take(3 * 3 / 3)
        .flatten()
        .collect();
        assert_eq!(vec![127u8; 1 * 1 * 4], downsample_rgba8(3, 3, &original));
    }

    #[test]
    fn downsample_rgba8_0x0() {
        assert!(downsample_rgba8(0, 0, &[]).is_empty());
    }

    #[test]
    fn create_surface_integral_dimensions() {
        // It's ok for mipmaps to not be divisible by the block width.
        let result = encode_surface_rgba8_generated_mipmaps(
            4,
            4,
            &[0u8; 64],
            ImageFormat::BC7Srgb,
            Quality::Fast,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn create_surface_non_integral_dimensions() {
        // This should still fail even though there is enough data.
        let result = encode_surface_rgba8_generated_mipmaps(
            3,
            5,
            &[0u8; 256],
            ImageFormat::BC7Srgb,
            Quality::Fast,
            true,
        );
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NonIntegralDimensionsInBlocks {
                width: 3,
                height: 5,
                block_width: 4,
                block_height: 4
            })
        ));
    }
}
