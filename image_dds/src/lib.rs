//! # Introduction
//! DDS can store the vast majority of both compressed and uncompressed GPU texture data.
//! This includes uncompressed formats like [ImageFormat::R8G8B8A8Unorm].
//! Libraries and applications for working with custom GPU texture file formats often support DDS.
//! This makes DDS a good interchange format for texture conversion workflows.
//!
//! DDS has more limited application support compared to
//! standard formats like TIFF or PNG especially on Linux and MacOS.
//! GPU compression formats tend to be lossy, which makes it a poor choice for archival purposes.
//! For this reason, it's often more convenient to work with texture data in an uncompressed format.
//!
//! image_dds enables safe and efficient compressed GPU texture conversion across platforms.
//! A conversion pipeline may look like GPU Texture <-> DDS <-> image with the
//! conversions to and from image and DDS provided by image_dds.
//!
//! Although widely supported by modern desktop and console hardware, not all contexts
//! support compressed texture formats. DDS plugins for image editors often don't support newer
//! compression formats like BC7. Rendering APIs may not support compressed formats or only make it available
//! via an extension such as in the browser.
//! image_dds supports decoding surfaces to RGBA8 for
//! better compatibility at the cost of increased memory usage.
//!
//! # Features
//! Despite the name, neither the `ddsfile` nor `image` crates are required
//! and can be disabled in the Cargo.toml by setting `default-features = false`.
//! The `"ddsfile"` and `"image"` features can then be enabled individually.
//! Surface data can still be encoded and decoded using lower level functions like
//! [decode_surface_rgba8] or [encode_surface_rgba8]. These lower level functions are
//! ideal for internal conversions in libraries or applications that want to skip intermediate formats like DDS.
//! Texture conversion utilities will probably want to use the higher level functions like
//! [image_from_dds] for convenience.
//!
//! # Limitations
//! BC2 data can be decoded but not encoded due to limitations in intel-tex-rs-2.
//! This format is very rarely used in practice.
//! Not all targets will compile by default due to intel-tex-rs-2 using the Intel ISPC compiler.
//! Precompiled kernels aren't available for all targets but can be compiled from source if needed.
//! 3D textures as well as cube map and array layers are not supported but will be added in a future update.
//! Creating DDS files with custom mipmaps or extracting mipmap data is not yet supported.
//! Supporting for floating point data will also be added in a future update.
//! This mostly impacts BC6H compression since it encodes half precision floating point data.

use bcn::*;
use rgba::*;

mod bcn;
mod rgba;
mod surface;

pub use surface::{Surface, SurfaceRgba8};

pub mod error;
use error::*;

#[cfg(feature = "ddsfile")]
mod dds;
#[cfg(feature = "ddsfile")]
pub use dds::*;

/// The conversion quality when converting to compressed formats.
///
/// Higher quality settings run significantly slower.
/// Block compressed formats like BC7 use a fixed compression ratio,
/// so lower quality settings do not use less space than slower ones.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    /// Faster exports with slightly lower quality.
    Fast,
    /// Normal export speed and quality.
    Normal,
    /// Slower exports for slightly higher quality.
    Slow,
}

/// Options for how many mipmaps to generate.
/// Mipmaps are counted starting from the base level,
/// so a surface with only the full resolution base level has 1 mipmap.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mipmaps {
    /// No mipmapping. Only the base mip level will be used.
    Disabled,
    /// Use the number of mipmaps specified in the input surface.
    FromSurface,
    /// Generate mipmaps to create a surface with a desired number of mipmaps.
    /// A value of `0` or `1` is equivalent to [Mipmaps::Disabled].
    GeneratedExact(u32),
    /// Generate mipmaps starting from the base level
    /// until dimensions can be reduced no further.
    GeneratedAutomatic,
}

// Each format should have conversions to and from rgba8 and rgbaf32 for convenience.
// Document the channels and bit depths for each format (i.e bc6 is half precision float, bc7 is rgba8, etc).
/// Supported image formats for encoding and decoding.
///
/// Not all DDS formats are supported,
/// but all current variants for [ImageFormat] are supported by some version of DDS.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::EnumString, strum::Display, strum::EnumIter)]
pub enum ImageFormat {
    R8Unorm,
    R8G8B8A8Unorm,
    R8G8B8A8Srgb,
    R32G32B32A32Float,
    B8G8R8A8Unorm,
    B8G8R8A8Srgb,
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
    fn block_dimensions(&self) -> (u32, u32, u32) {
        match self {
            ImageFormat::BC1Unorm => (4, 4, 1),
            ImageFormat::BC1Srgb => (4, 4, 1),
            ImageFormat::BC2Unorm => (4, 4, 1),
            ImageFormat::BC2Srgb => (4, 4, 1),
            ImageFormat::BC3Unorm => (4, 4, 1),
            ImageFormat::BC3Srgb => (4, 4, 1),
            ImageFormat::BC4Unorm => (4, 4, 1),
            ImageFormat::BC4Snorm => (4, 4, 1),
            ImageFormat::BC5Unorm => (4, 4, 1),
            ImageFormat::BC5Snorm => (4, 4, 1),
            ImageFormat::BC6Ufloat => (4, 4, 1),
            ImageFormat::BC6Sfloat => (4, 4, 1),
            ImageFormat::BC7Unorm => (4, 4, 1),
            ImageFormat::BC7Srgb => (4, 4, 1),
            ImageFormat::R8Unorm => (1, 1, 1),
            ImageFormat::R8G8B8A8Unorm => (1, 1, 1),
            ImageFormat::R8G8B8A8Srgb => (1, 1, 1),
            ImageFormat::R32G32B32A32Float => (1, 1, 1),
            ImageFormat::B8G8R8A8Unorm => (1, 1, 1),
            ImageFormat::B8G8R8A8Srgb => (1, 1, 1),
        }
    }

    fn block_size_in_bytes(&self) -> usize {
        match self {
            ImageFormat::R8Unorm => 1,
            ImageFormat::R8G8B8A8Unorm => 4,
            ImageFormat::R8G8B8A8Srgb => 4,
            ImageFormat::R32G32B32A32Float => 16,
            ImageFormat::B8G8R8A8Unorm => 4,
            ImageFormat::B8G8R8A8Srgb => 4,
            ImageFormat::BC1Unorm => 8,
            ImageFormat::BC1Srgb => 8,
            ImageFormat::BC2Unorm => 16,
            ImageFormat::BC2Srgb => 16,
            ImageFormat::BC3Unorm => 16,
            ImageFormat::BC3Srgb => 16,
            ImageFormat::BC4Unorm => 8,
            ImageFormat::BC4Snorm => 8,
            ImageFormat::BC5Unorm => 16,
            ImageFormat::BC5Snorm => 16,
            ImageFormat::BC6Ufloat => 16,
            ImageFormat::BC6Sfloat => 16,
            ImageFormat::BC7Unorm => 16,
            ImageFormat::BC7Srgb => 16,
        }
    }
}

struct Rgba;
impl Rgba {
    const BYTES_PER_PIXEL: usize = 4;
    const BYTES_PER_BLOCK: usize = 64;
}

fn max_mipmap_count(max_dimension: u32) -> u32 {
    // log2(x) + 1
    u32::BITS - max_dimension.leading_zeros()
}

/// The reduced value for `base_dimension` at level `mipmap`.
pub fn mip_dimension(base_dimension: u32, mipmap: u32) -> u32 {
    // Halve for each mip level.
    (base_dimension >> mipmap).max(1)
}

/// Decode all layers and mipmaps from `surface` to RGBA8.
pub fn decode_surface_rgba8<T: AsRef<[u8]>>(
    surface: Surface<T>,
) -> Result<SurfaceRgba8<Vec<u8>>, DecompressSurfaceError> {
    let Surface {
        width,
        height,
        depth,
        layers,
        mipmaps,
        image_format,
        data: _,
    } = surface;

    surface.validate()?;

    let mut combined_surface_data = Vec::new();
    for layer in 0..layers {
        for mipmap in 0..mipmaps {
            let data = surface
                .get(layer, mipmap)
                .ok_or(DecompressSurfaceError::MipmapDataOutOfBounds { layer, mipmap })?;

            // The mipmap index is already validated by get above.
            let width = mip_dimension(width, mipmap);
            let height = mip_dimension(height, mipmap);
            let depth = mip_dimension(depth, mipmap);

            // TODO: Avoid additional copies?
            let data = decode_data_rgba8(width, height, depth, image_format, data)?;
            combined_surface_data.extend_from_slice(&data);
        }
    }

    Ok(SurfaceRgba8 {
        width,
        height,
        depth,
        layers,
        mipmaps,
        data: combined_surface_data,
    })
}

fn decode_data_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    image_format: ImageFormat,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError> {
    use ImageFormat as F;
    let data = match image_format {
        F::BC1Unorm | F::BC1Srgb => rgba8_from_bcn::<Bc1>(width, height, depth, data),
        F::BC2Unorm | F::BC2Srgb => rgba8_from_bcn::<Bc2>(width, height, depth, data),
        F::BC3Unorm | F::BC3Srgb => rgba8_from_bcn::<Bc3>(width, height, depth, data),
        F::BC4Unorm | F::BC4Snorm => rgba8_from_bcn::<Bc4>(width, height, depth, data),
        F::BC5Unorm | F::BC5Snorm => rgba8_from_bcn::<Bc5>(width, height, depth, data),
        F::BC6Ufloat | F::BC6Sfloat => rgba8_from_bcn::<Bc6>(width, height, depth, data),
        F::BC7Unorm | F::BC7Srgb => rgba8_from_bcn::<Bc7>(width, height, depth, data),
        F::R8Unorm => rgba8_from_r8(width, height, depth, data),
        F::R8G8B8A8Unorm => decode_rgba8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Srgb => decode_rgba8_from_rgba8(width, height, depth, data),
        F::R32G32B32A32Float => rgba8_from_rgbaf32(width, height, depth, data),
        F::B8G8R8A8Unorm => rgba8_from_bgra8(width, height, depth, data),
        F::B8G8R8A8Srgb => rgba8_from_bgra8(width, height, depth, data),
    }?;
    Ok(data)
}

// TODO: Take the surface by reference?
// TODO: Add documentation showing how to use this.
/// Encode an RGBA8 surface to the given `format`.
///
/// The number of mipmaps generated depends on the `mipmaps` parameter.
/// The `rgba8_data` only needs to contain enough data for the base mip level of `width` x `height` pixels.
pub fn encode_surface_rgba8<T: AsRef<[u8]>>(
    surface: SurfaceRgba8<T>,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Surface<Vec<u8>>, CompressSurfaceError> {
    let width = surface.width;
    let height = surface.height;
    let depth = surface.depth;
    let layers = surface.layers;

    surface.validate_encode(format)?;

    // TODO: Encode the correct number of array layers.
    let num_mipmaps = match mipmaps {
        Mipmaps::Disabled => 1,
        Mipmaps::FromSurface => surface.mipmaps,
        Mipmaps::GeneratedExact(count) => count,
        Mipmaps::GeneratedAutomatic => max_mipmap_count(width.max(height).max(depth)),
    };

    let use_surface = mipmaps == Mipmaps::FromSurface;

    // TODO: Does this work if the base mip level is smaller than 4x4?
    let mut surface_data = Vec::new();

    for layer in 0..layers {
        encode_mipmaps_rgba8(
            &mut surface_data,
            &surface,
            format,
            quality,
            num_mipmaps,
            use_surface,
            layer,
        )?;
    }

    Ok(Surface {
        width,
        height,
        depth,
        layers,
        mipmaps: num_mipmaps,
        image_format: format,
        data: surface_data,
    })
}

fn encode_mipmaps_rgba8<T: AsRef<[u8]>>(
    encoded_data: &mut Vec<u8>,
    surface: &SurfaceRgba8<T>,
    format: ImageFormat,
    quality: Quality,
    num_mipmaps: u32,
    use_surface: bool,
    layer: u32,
) -> Result<(), CompressSurfaceError> {
    let width = surface.width;
    let height = surface.height;
    let depth = surface.depth;
    let (block_width, block_height, block_depth) = format.block_dimensions();

    // The base mip level is always included.
    let base_layer = encode_rgba8(
        width.max(block_width),
        height.max(block_height),
        depth.max(block_depth),
        surface.data.as_ref(),
        format,
        quality,
    )?;
    encoded_data.extend_from_slice(&base_layer);

    let mut mip_image = surface.data.as_ref().to_vec();

    let mut previous_width = width as usize;
    let mut previous_height = height as usize;
    let mut previous_depth = depth as usize;

    for mipmap in 1..num_mipmaps {
        // The physical size must have integral dimensions in blocks.
        // Applications or the GPU will use the smaller virtual size and ignore padding.
        // For example, a 1x1 BCN block still requires 4x4 pixels of data.
        // https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression
        let mip_dimension_rounded = |x, n| round_up(mip_dimension(x, mipmap) as usize, n as usize);
        let mip_width = mip_dimension_rounded(width, block_width);
        let mip_height = mip_dimension_rounded(height, block_height);
        let mip_depth = mip_dimension_rounded(depth, block_depth);

        // TODO: Find a simpler way to choose a data source.
        mip_image = if use_surface {
            // TODO: Array layers.
            // TODO: Avoid unwrap
            let data = surface.get(layer, mipmap).unwrap();
            let expected_size = mip_width * mip_height * mip_depth * 4;

            if data.len() < expected_size {
                // Zero pad the data to the appropriate size.
                let mut padded_data = vec![0u8; expected_size];
                for z in 0..mip_depth {
                    for y in 0..mip_height {
                        for x in 0..mip_width {
                            // TODO: Make this copy technique a helper function?
                            // TODO: Optimize this for known pixel sizes?
                            // This can't be a memory copy because of the stride.
                            let i = (z * mip_width * mip_height) + y * mip_width + x;
                            padded_data[i] = data[i];
                        }
                    }
                }

                padded_data
            } else {
                data.to_vec()
            }
        } else {
            // Downsample the previous mip level.
            // This also handles padding since the new dimensions are rounded.
            downsample_rgba8(
                mip_width,
                mip_height,
                mip_depth,
                previous_width,
                previous_height,
                previous_depth,
                &mip_image,
            )
        };

        let mip_data = encode_rgba8(
            mip_width as u32,
            mip_height as u32,
            mip_depth as u32,
            &mip_image,
            format,
            quality,
        )?;
        encoded_data.extend_from_slice(&mip_data);

        // Update the dimensions for the previous mipmap image data.
        previous_width = mip_width;
        previous_height = mip_height;
        previous_depth = mip_depth;
    }
    Ok(())
}

fn encode_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
    format: ImageFormat,
    quality: Quality,
) -> Result<Vec<u8>, CompressSurfaceError> {
    // TODO: Handle unorm vs srgb for uncompressed or leave the data as is?

    use ImageFormat as F;
    match format {
        F::BC1Unorm | F::BC1Srgb => bcn_from_rgba8::<Bc1>(width, height, depth, data, quality),
        F::BC2Unorm | F::BC2Srgb => bcn_from_rgba8::<Bc2>(width, height, depth, data, quality),
        F::BC3Unorm | F::BC3Srgb => bcn_from_rgba8::<Bc3>(width, height, depth, data, quality),
        F::BC4Unorm | F::BC4Snorm => bcn_from_rgba8::<Bc4>(width, height, depth, data, quality),
        F::BC5Unorm | F::BC5Snorm => bcn_from_rgba8::<Bc5>(width, height, depth, data, quality),
        F::BC6Ufloat | F::BC6Sfloat => bcn_from_rgba8::<Bc6>(width, height, depth, data, quality),
        F::BC7Unorm | F::BC7Srgb => bcn_from_rgba8::<Bc7>(width, height, depth, data, quality),
        F::R8Unorm => r8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Unorm => encode_rgba8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Srgb => encode_rgba8_from_rgba8(width, height, depth, data),
        F::R32G32B32A32Float => rgbaf32_from_rgba8(width, height, depth, data),
        F::B8G8R8A8Unorm => bgra8_from_rgba8(width, height, depth, data),
        F::B8G8R8A8Srgb => bgra8_from_rgba8(width, height, depth, data),
    }
}

fn downsample_rgba8(
    new_width: usize,
    new_height: usize,
    new_depth: usize,
    width: usize,
    height: usize,
    depth: usize,
    data: &[u8],
) -> Vec<u8> {
    // Halve the width and height by averaging pixels.
    // This is faster than resizing using the image crate.
    let mut new_data = vec![0u8; new_width * new_height * new_depth * 4];
    for z in 0..new_depth {
        for x in 0..new_width {
            for y in 0..new_height {
                let new_index = (z * new_width * new_height) + y * new_width + x;

                // Average a 2x2x2 pixel region from data into a 1x1x1 pixel region.
                // This is equivalent to a 3D convolution or pooling operation over the pixels.
                for c in 0..4 {
                    let mut sum = 0;
                    let mut count = 0;
                    for z2 in 0..2 {
                        let sampled_z = (z * 2) + z2;
                        if sampled_z < depth {
                            for y2 in 0..2 {
                                let sampled_y = (y * 2) + y2;
                                if sampled_y < height {
                                    for x2 in 0..2 {
                                        let sampled_x = (x * 2) + x2;
                                        if sampled_x < width {
                                            let index = (sampled_z * width * height)
                                                + (sampled_y * width)
                                                + sampled_x;
                                            sum += data[index * 4 + c] as usize;
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    new_data[new_index * 4 + c] = (sum as f64 / count as f64) as u8;
                }
            }
        }
    }

    new_data
}

#[inline(always)]
fn div_round_up(x: usize, d: usize) -> usize {
    (x + d - 1) / d
}

#[inline(always)]
fn round_up(x: usize, n: usize) -> usize {
    ((x + n - 1) / n) * n
}

fn calculate_offset(
    layer: u32,
    mipmap: u32,
    dimensions: (u32, u32, u32),
    block_dimensions: (u32, u32, u32),
    block_size_in_bytes: usize,
    mipmaps_per_layer: u32,
) -> Option<usize> {
    // Surfaces typically use a row-major memory layout like surface[layer][mipmap][z][y][x].
    // Not all mipmaps are the same size, so the offset calculation is slightly more complex.
    let (width, height, depth) = dimensions;
    let (block_width, block_height, block_depth) = block_dimensions;

    let mip_sizes = (0..mipmaps_per_layer)
        .map(|i| {
            let mip_width = mip_dimension(width, i) as usize;
            let mip_height = mip_dimension(height, i) as usize;
            let mip_depth = mip_dimension(depth, i) as usize;

            mip_size(
                mip_width,
                mip_height,
                mip_depth,
                block_width as usize,
                block_height as usize,
                block_depth as usize,
                block_size_in_bytes,
            )
        })
        .collect::<Option<Vec<_>>>()?;

    // Assume mipmaps are tightly packed.
    // This is the case for DDS surface data.
    let layer_size: usize = mip_sizes.iter().sum();

    // Each layer should have the same number of mipmaps.
    let layer_offset = layer as usize * layer_size;
    let mip_offset: usize = mip_sizes.get(0..mipmap as usize)?.iter().sum();
    Some(layer_offset + mip_offset)
}

fn mip_size(
    width: usize,
    height: usize,
    depth: usize,
    block_width: usize,
    block_height: usize,
    block_depth: usize,
    block_size_in_bytes: usize,
) -> Option<usize> {
    div_round_up(width, block_width)
        .checked_mul(div_round_up(height, block_height))
        .and_then(|v| v.checked_mul(div_round_up(depth, block_depth)))
        .and_then(|v| v.checked_mul(block_size_in_bytes))
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
        assert_eq!(
            vec![127u8; 2 * 2 * 1 * 4],
            downsample_rgba8(2, 2, 1, 4, 4, 1, &original)
        );
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
        assert_eq!(
            vec![127u8; 1 * 1 * 4],
            downsample_rgba8(1, 1, 1, 3, 3, 1, &original)
        );
    }

    #[test]
    fn downsample_rgba8_2x2x2() {
        // Test that two slices of 2x2 pixels are averaged.
        let original = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255,
        ];
        assert_eq!(
            vec![127u8; 1 * 1 * 1 * 4],
            downsample_rgba8(1, 1, 1, 2, 2, 2, &original)
        );
    }

    #[test]
    fn downsample_rgba8_0x0() {
        assert_eq!(vec![0u8; 4], downsample_rgba8(1, 1, 1, 0, 0, 1, &[]));
    }

    #[test]
    fn encode_surface_integral_dimensions() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 12,
                height: 12,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: &[0u8; 12 * 12 * 4],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(12, surface.width);
        assert_eq!(12, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(4, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!((9 + 4 + 1 + 1) * 16, surface.data.len());
    }

    #[test]
    fn encode_surface_cube_mipmaps() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 6,
                mipmaps: 3,
                data: &[0u8; (4 * 4 + 2 * 2 + 1 * 1) * 6 * 4],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(6, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!(3 * 16 * 6, surface.data.len());
    }

    #[test]
    fn encode_surface_disabled_mipmaps() {
        let surface = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 3,
                data: &[0u8; 64 + 16 + 4],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::Disabled,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(1, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        assert_eq!(16, surface.data.len());
    }

    #[test]
    fn encode_surface_mipmaps_from_surface() {
        let surface = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 2,
                data: &[0u8; 64 + 16],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::FromSurface,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(2, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        assert_eq!(16 * 2, surface.data.len());
    }

    #[test]
    fn encode_surface_non_integral_dimensions() {
        // This should still fail even though there is enough data.
        let result = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 3,
                height: 5,
                depth: 2,
                layers: 1,
                mipmaps: 1,
                data: &[0u8; 256],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        );
        assert!(matches!(
            result,
            Err(CompressSurfaceError::NonIntegralDimensionsInBlocks {
                width: 3,
                height: 5,
                depth: 2,
                block_width: 4,
                block_height: 4
            })
        ));
    }

    #[test]
    fn encode_surface_zero_size() {
        let result = encode_surface_rgba8(
            SurfaceRgba8 {
                width: 0,
                height: 0,
                depth: 0,
                layers: 1,
                mipmaps: 1,
                data: &[0u8; 0],
            },
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        );
        assert!(matches!(
            result,
            Err(CompressSurfaceError::ZeroSizedSurface {
                width: 0,
                height: 0,
                depth: 0,
            })
        ));
    }

    #[test]
    fn decode_surface_zero_size() {
        let result = decode_surface_rgba8(Surface {
            width: 0,
            height: 0,
            depth: 0,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::R8G8B8A8Srgb,
            data: &[0u8; 0],
        });
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::ZeroSizedSurface {
                width: 0,
                height: 0,
                depth: 0,
            })
        ));
    }

    #[test]
    fn decode_surface_dimensions_overflow() {
        let result = decode_surface_rgba8(Surface {
            width: u32::MAX,
            height: u32::MAX,
            depth: u32::MAX,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::R8G8B8A8Srgb,
            data: &[0u8; 0],
        });
        assert!(matches!(
            result,
            Err(DecompressSurfaceError::PixelCountWouldOverflow {
                width: u32::MAX,
                height: u32::MAX,
                depth: u32::MAX,
            })
        ));
    }

    #[test]
    fn decode_surface_too_many_mipmaps() {
        let result = decode_surface_rgba8(Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 10,
            image_format: ImageFormat::R8G8B8A8Srgb,
            data: &[0u8; 4 * 4 * 4],
        });

        assert!(matches!(
            result,
            Err(DecompressSurfaceError::UnexpectedMipmapCount {
                mipmaps: 10,
                max_mipmaps: 3
            })
        ));
    }

    #[test]
    fn calculate_offset_layer0_mip0() {
        assert_eq!(
            0,
            calculate_offset(0, 0, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer0_mip2() {
        // The sum of the first 2 mipmaps.
        assert_eq!(
            128 + 16,
            calculate_offset(0, 2, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer2_mip0() {
        // The sum of the first 2 array layers.
        // Each mipmap must have at least a full block of data.
        assert_eq!(
            (128 + 16 + 16 + 16) * 2,
            calculate_offset(2, 0, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer2_mip2() {
        // The sum of the first two layers and two more mipmaps.
        // Each mipmap must have at least a full block of data.
        assert_eq!(
            (128 + 16 + 16 + 16) * 2 + 128 + 16,
            calculate_offset(2, 2, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }
}
