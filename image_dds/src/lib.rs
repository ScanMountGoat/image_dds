//! # image_dds
//! image_dds enables converting uncompressed image data to and from compressed formats.
//!
//! Start converting image data by creating a [Surface] and using one of the provided methods.
//!
//! # Usage
//! The main conversion functions [image_from_dds] and [dds_from_image] convert between [ddsfile] and [image].
//! For working with floating point images like EXR files, use [imagef32_from_dds] and [dds_from_imagef32].
//!
//! These functions are wrappers over conversion methods for [Surface], [SurfaceRgba8], and [SurfaceRgba32Float].
//! These methods are ideal for internal conversions in libraries
//! or applications that want to use [Surface] instead of DDS as an intermediate format.
//!
//! Surfaces may use owned or borrowed data depending on whether the operation is lossless or not.
//! A [SurfaceRgba8] can represent a view over an [image::RgbaImage] without any copies, for example.
//!
//! For working with custom texture file formats like in video games,
//! consider defining conversion methods to and from [Surface] to enable chaining operations.
//! These methods may need to return an error if not all texture formats are supported by [ImageFormat].
//!
//! ```rust no_run
//! # struct CustomTex;
//! # impl CustomTex {
//! #     fn to_surface(&self) -> Result<image_dds::Surface<Vec<u8>>, Box<dyn std::error::Error>> {
//! #         todo!()
//! #     }
//! #     fn from_surface<T: AsRef<[u8]>>(
//! #         surface: image_dds::Surface<T>,
//! #     ) -> Result<image_dds::Surface<Vec<u8>>, Box<dyn std::error::Error>> {
//! #         todo!()
//! #     }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let custom_tex = CustomTex;
//! let dds = custom_tex.to_surface()?.to_dds()?;
//!
//! let image = image::open("cat.png").unwrap().to_rgba8();
//! let surface = image_dds::SurfaceRgba8::from_image(&image).encode(
//!     image_dds::ImageFormat::BC7RgbaUnorm,
//!     image_dds::Quality::Normal,
//!     image_dds::Mipmaps::GeneratedAutomatic,
//! )?;
//! let new_custom_tex = CustomTex::from_surface(surface)?;
//! # Ok(()) }
//! ```
//!
//! # Features
//! Despite the name, neither the `ddsfile` nor `image` crates are required
//! and can be disabled in the Cargo.toml by setting `default-features = false`.
//! The `"ddsfile"` and `"image"` features can then be enabled individually.
//! The `"encode"` feature is enabled by default but can be disabled
//! to resolve compilation errors on some targets if not needed.
//!
//! # Direct Draw Surface (DDS)
//! DDS can store GPU texture data in a variety of formats.
//! This includes compressed formats like [ImageFormat::BC7RgbaUnorm] or uncompressed formats like [ImageFormat::Rgba8Unorm].
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
//! support compressed texture formats. DDS plugins for image editors may not support newer
//! compression formats like BC7. Rendering APIs may not support some compressed formats or only make it available
//! via an extension such as in the browser.
//! image_dds supports decoding surfaces to RGBA `u8` or `f32` for
//! better compatibility at the cost of increased memory usage.
//!
//! # Limitations
//! Not all targets will compile by default due to intel-tex-rs-2 using the Intel ISPC compiler
//! and lacking precompiled kernels for all targets.
//! Disable the `"encode"` feature if not needed.

mod bcn;
mod rgba;
mod surface;

use rgba::convert::Channel;
pub use surface::{Surface, SurfaceRgba32Float, SurfaceRgba8};

pub mod error;
use error::*;

#[cfg(feature = "ddsfile")]
pub use ddsfile;

#[cfg(feature = "image")]
pub use image;

mod decode;

#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "ddsfile")]
mod dds;
#[cfg(feature = "ddsfile")]
pub use dds::*;

/// The conversion quality when encoding to compressed formats.
///
/// Higher quality settings run significantly slower.
/// Block compressed formats like BC7 use a fixed compression ratio,
/// so lower quality settings do not use less space than slower ones.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "strum",
    derive(strum::EnumString, strum::Display, strum::EnumIter)
)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "strum",
    derive(strum::EnumString, strum::Display, strum::EnumIter)
)]
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

/// Supported image formats for encoding and decoding.
///
/// Not all DDS formats are supported,
/// but all current variants for [ImageFormat] are supported by some version of DDS.
#[non_exhaustive]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "strum",
    derive(strum::EnumString, strum::Display, strum::EnumIter)
)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ImageFormat {
    R8Unorm,
    R8Snorm,
    Rg8Unorm,
    Rg8Snorm,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    Bgr8Unorm,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Bgra4Unorm,
    /// DXT1
    BC1RgbaUnorm,
    BC1RgbaUnormSrgb,
    /// DXT3
    BC2RgbaUnorm,
    BC2RgbaUnormSrgb,
    /// DXT5
    BC3RgbaUnorm,
    BC3RgbaUnormSrgb,
    /// RGTC1
    BC4RUnorm,
    BC4RSnorm,
    /// RGTC2
    BC5RgUnorm,
    BC5RgSnorm,
    /// BPTC (float)
    BC6hRgbUfloat,
    BC6hRgbSfloat,
    /// BPTC (unorm)
    BC7RgbaUnorm,
    BC7RgbaUnormSrgb,
    Rgba8Snorm,
    R16Unorm,
    R16Snorm,
    Rg16Unorm,
    Rg16Snorm,
    Rgba16Unorm,
    Rgba16Snorm,
    R16Float,
    Rg16Float,
    R32Float,
    Rg32Float,
    Rgb32Float,
    Bgr5A1Unorm,
}

impl ImageFormat {
    // TODO: Is it worth making these public?
    fn block_dimensions(&self) -> (u32, u32, u32) {
        match self {
            ImageFormat::BC1RgbaUnorm => (4, 4, 1),
            ImageFormat::BC1RgbaUnormSrgb => (4, 4, 1),
            ImageFormat::BC2RgbaUnorm => (4, 4, 1),
            ImageFormat::BC2RgbaUnormSrgb => (4, 4, 1),
            ImageFormat::BC3RgbaUnorm => (4, 4, 1),
            ImageFormat::BC3RgbaUnormSrgb => (4, 4, 1),
            ImageFormat::BC4RUnorm => (4, 4, 1),
            ImageFormat::BC4RSnorm => (4, 4, 1),
            ImageFormat::BC5RgUnorm => (4, 4, 1),
            ImageFormat::BC5RgSnorm => (4, 4, 1),
            ImageFormat::BC6hRgbUfloat => (4, 4, 1),
            ImageFormat::BC6hRgbSfloat => (4, 4, 1),
            ImageFormat::BC7RgbaUnorm => (4, 4, 1),
            ImageFormat::BC7RgbaUnormSrgb => (4, 4, 1),
            _ => (1, 1, 1),
        }
    }

    fn block_size_in_bytes(&self) -> usize {
        // Size of a block if compressed or pixel if uncompressed.
        match self {
            ImageFormat::R8Unorm => 1,
            ImageFormat::R8Snorm => 1,
            ImageFormat::Rg8Unorm => 2,
            ImageFormat::Rg8Snorm => 2,
            ImageFormat::Rgba8Unorm => 4,
            ImageFormat::Rgba8UnormSrgb => 4,
            ImageFormat::Rgba16Float => 8,
            ImageFormat::Rgba32Float => 16,
            ImageFormat::Bgra8Unorm => 4,
            ImageFormat::Bgra8UnormSrgb => 4,
            ImageFormat::BC1RgbaUnorm => 8,
            ImageFormat::BC1RgbaUnormSrgb => 8,
            ImageFormat::BC2RgbaUnorm => 16,
            ImageFormat::BC2RgbaUnormSrgb => 16,
            ImageFormat::BC3RgbaUnorm => 16,
            ImageFormat::BC3RgbaUnormSrgb => 16,
            ImageFormat::BC4RUnorm => 8,
            ImageFormat::BC4RSnorm => 8,
            ImageFormat::BC5RgUnorm => 16,
            ImageFormat::BC5RgSnorm => 16,
            ImageFormat::BC6hRgbUfloat => 16,
            ImageFormat::BC6hRgbSfloat => 16,
            ImageFormat::BC7RgbaUnorm => 16,
            ImageFormat::BC7RgbaUnormSrgb => 16,
            ImageFormat::Bgra4Unorm => 2,
            ImageFormat::Bgr8Unorm => 3,
            ImageFormat::R16Unorm => 2,
            ImageFormat::R16Snorm => 2,
            ImageFormat::Rg16Unorm => 4,
            ImageFormat::Rg16Snorm => 4,
            ImageFormat::Rgba16Unorm => 8,
            ImageFormat::Rgba16Snorm => 8,
            ImageFormat::Rg16Float => 4,
            ImageFormat::Rg32Float => 8,
            ImageFormat::R16Float => 2,
            ImageFormat::R32Float => 4,
            ImageFormat::Rgba8Snorm => 4,
            ImageFormat::Rgb32Float => 12,
            ImageFormat::Bgr5A1Unorm => 2,
        }
    }
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

fn downsample_rgba<T: Channel>(
    new_width: usize,
    new_height: usize,
    new_depth: usize,
    width: usize,
    height: usize,
    depth: usize,
    data: &[T],
) -> Vec<T> {
    // Halve the width and height by averaging pixels.
    // This is faster than resizing using the image crate.
    let mut new_data = vec![T::ZERO; new_width * new_height * new_depth * 4];
    for z in 0..new_depth {
        for x in 0..new_width {
            for y in 0..new_height {
                let new_index = (z * new_width * new_height) + y * new_width + x;

                // Average a 2x2x2 pixel region from data into a 1x1x1 pixel region.
                // This is equivalent to a 3D convolution or pooling operation over the pixels.
                for c in 0..4 {
                    let mut sum = 0.0;
                    let mut count = 0u64;
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
                                            sum += data[index * 4 + c].to_f32();
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    new_data[new_index * 4 + c] = T::from_f32(sum / count.max(1) as f32);
                }
            }
        }
    }

    new_data
}

fn calculate_offset(
    layer: u32,
    depth_level: u32,
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

    // Each depth level adds another rounded 2D slice.
    let mip_width = mip_dimension(width, mipmap) as usize;
    let mip_height = mip_dimension(height, mipmap) as usize;
    let mip_size2d = mip_size(
        mip_width,
        mip_height,
        1,
        block_width as usize,
        block_height as usize,
        block_depth as usize,
        block_size_in_bytes,
    )?;

    // Assume mipmaps are tightly packed.
    // This is the case for DDS surface data.
    let layer_size: usize = mip_sizes.iter().sum();

    // Each layer should have the same number of mipmaps.
    let layer_offset = layer as usize * layer_size;
    let mip_offset: usize = mip_sizes.get(0..mipmap as usize)?.iter().sum();
    let depth_offset = mip_size2d * depth_level as usize;
    Some(layer_offset + mip_offset + depth_offset)
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
    width
        .div_ceil(block_width)
        .checked_mul(height.div_ceil(block_height))
        .and_then(|v| v.checked_mul(depth.div_ceil(block_depth)))
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
            downsample_rgba(2, 2, 1, 4, 4, 1, &original)
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
            downsample_rgba(1, 1, 1, 3, 3, 1, &original)
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
            downsample_rgba(1, 1, 1, 2, 2, 2, &original)
        );
    }

    #[test]
    fn downsample_rgba8_0x0() {
        assert_eq!(vec![0u8; 4], downsample_rgba(1, 1, 1, 0, 0, 1, &[]));
    }

    #[test]
    fn downsample_rgbaf32_4x4() {
        // Test that a checkerboard is averaged.
        let original: Vec<_> = std::iter::repeat([
            0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32,
        ])
        .take(4 * 4 / 2)
        .flatten()
        .collect();
        assert_eq!(
            vec![0.5; 2 * 2 * 1 * 4],
            downsample_rgba(2, 2, 1, 4, 4, 1, &original)
        );
    }

    #[test]
    fn downsample_rgbaf32_3x3() {
        // Test that a checkerboard is averaged.
        let original: Vec<_> = std::iter::repeat([
            0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32,
            0.0f32,
        ])
        .take(3 * 3 / 3)
        .flatten()
        .collect();
        assert_eq!(
            vec![0.5; 1 * 1 * 4],
            downsample_rgba(1, 1, 1, 3, 3, 1, &original)
        );
    }

    #[test]
    fn downsample_rgbaf32_2x2x2() {
        // Test that two slices of 2x2 pixels are averaged.
        let original = vec![
            0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32,
            1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32, 1.0f32,
        ];
        assert_eq!(
            vec![0.5; 1 * 1 * 1 * 4],
            downsample_rgba(1, 1, 1, 2, 2, 2, &original)
        );
    }

    #[test]
    fn downsample_rgbaf32_0x0() {
        assert_eq!(vec![0.0f32; 4], downsample_rgba(1, 1, 1, 0, 0, 1, &[]));
    }

    #[test]
    fn calculate_offset_layer0_mip0() {
        assert_eq!(
            0,
            calculate_offset(0, 0, 0, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer0_mip2() {
        // The sum of the first 2 mipmaps.
        assert_eq!(
            128 + 16,
            calculate_offset(0, 0, 2, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer2_mip0() {
        // The sum of the first 2 array layers.
        // Each mipmap must have at least a full block of data.
        assert_eq!(
            (128 + 16 + 16 + 16) * 2,
            calculate_offset(2, 0, 0, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_layer2_mip2() {
        // The sum of the first two layers and two more mipmaps.
        // Each mipmap must have at least a full block of data.
        assert_eq!(
            (128 + 16 + 16 + 16) * 2 + 128 + 16,
            calculate_offset(2, 0, 2, (8, 8, 8), (4, 4, 4), 16, 4).unwrap()
        );
    }

    #[test]
    fn calculate_offset_level2() {
        // Each 2D level is rounded up to 16x16 pixels.
        assert_eq!(
            16 * 16 * 2,
            calculate_offset(0, 2, 0, (15, 15, 15), (4, 4, 4), 16, 1).unwrap()
        );
    }

    #[test]
    fn calculate_offset_level3() {
        // Each 2D level is 16x16 pixels.
        assert_eq!(
            16 * 16 * 3 * 4,
            calculate_offset(0, 3, 0, (16, 16, 16), (1, 1, 1), 4, 1).unwrap()
        );
    }
}
