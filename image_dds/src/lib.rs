//! # Introduction
//! DDS can store the vast majority of both compressed and uncompressed GPU texture data.
//! This includes uncompressed formats like [ImageFormat::Rgba8Unorm].
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
//! image_dds supports decoding surfaces to RGBA `u8` or `f32` for
//! better compatibility at the cost of increased memory usage.
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
//! # Features
//! Despite the name, neither the `ddsfile` nor `image` crates are required
//! and can be disabled in the Cargo.toml by setting `default-features = false`.
//! The `"ddsfile"` and `"image"` features can then be enabled individually.
//! The `"encode"` feature is enabled by default but can be disabled
//! to resolve compilation errors on some targets if not needed.
//!
//! # Limitations
//! Not all targets will compile by default due to intel-tex-rs-2 using the Intel ISPC compiler
//! and lacking precompiled kernels for all targets.

mod bcn;
mod rgba;
mod surface;

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

// TODO: Is this the best way to handle this?
trait Pixel: Default + Copy {
    fn from_f32(f: f32) -> Self;
    fn to_f32(&self) -> f32;
}

impl Pixel for u8 {
    fn from_f32(f: f32) -> Self {
        f as Self
    }

    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl Pixel for f32 {
    fn from_f32(f: f32) -> Self {
        f
    }

    fn to_f32(&self) -> f32 {
        *self
    }
}

fn downsample_rgba<T: Pixel>(
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
    let mut new_data = vec![T::default(); new_width * new_height * new_depth * 4];
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
    div_round_up(width, block_width)
        .checked_mul(div_round_up(height, block_height))
        .and_then(|v| v.checked_mul(div_round_up(depth, block_depth)))
        .and_then(|v| v.checked_mul(block_size_in_bytes))
}

fn snorm8_to_unorm8(x: u8) -> u8 {
    // Validated against decoding R8Snorm DDS with GPU and paint.net (DirectXTex).
    if x < 128 {
        x + 128
    } else if x == 128 {
        0
    } else {
        x - 129
    }
}

fn unorm8_to_snorm8(x: u8) -> u8 {
    // Inverse of snorm_to_unorm.
    if x >= 128 {
        x - 128
    } else if x == 127 {
        0
    } else {
        x + 129
    }
}

fn snorm8_to_float(x: u8) -> f32 {
    ((x as i8) as f32 / 127.0).max(-1.0)
}

fn float_to_snorm8(x: f32) -> i8 {
    ((x.clamp(-1.0, 1.0)) * 127.0).round() as i8
}

fn snorm16_to_float(x: u16) -> f32 {
    ((x as i16) as f32 / 32767.0).max(-1.0)
}

fn float_to_snorm16(x: f32) -> i16 {
    ((x.clamp(-1.0, 1.0)) * 32767.0).round() as i16
}

// https://rundevelopment.github.io/blog/fast-unorm-conversions
fn unorm4_to_unorm8(x: u8) -> u8 {
    x * 17
}

fn unorm8_to_unorm4(x: u8) -> u8 {
    ((x as u16 * 15 + 135) >> 8) as u8
}

fn unorm16_to_unorm8(x: u16) -> u8 {
    ((x as u32 * 255 + 32895) >> 16) as u8
}

fn unorm8_to_unorm16(x: u8) -> u16 {
    x as u16 * 257
}

// TODO: Find an efficient way to do this and add tests.
fn snorm16_to_unorm8(x: u16) -> u8 {
    // Remap [-1, 1] to [0, 1] to fit in an unsigned integer.
    ((snorm16_to_float(x) * 0.5 + 0.5) * 255.0).round() as u8
}

fn unorm8_to_snorm16(x: u8) -> i16 {
    // Remap [0, 1] to [-1, 1] to fit in a signed integer.
    (((x as f32 / 255.0) * 2.0 - 1.0) * 32767.0).round() as i16
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

    fn snorm8_to_unorm8_reference(x: u8) -> u8 {
        // Remap [-1, 1] to [0, 1] to fit in an unsigned integer.
        ((snorm8_to_float(x) * 0.5 + 0.5) * 255.0).round() as u8
    }

    fn unorm8_to_snorm8_reference(x: u8) -> i8 {
        // Remap [0, 1] to [-1, 1] to fit in a signed integer.
        (((x as f32 / 255.0) * 2.0 - 1.0) * 127.0).round() as i8
    }

    #[test]
    fn convert_snorm8_to_unorm8() {
        // 128, ..., 255, 0, ..., 126
        for i in 0..=255 {
            assert_eq!(snorm8_to_unorm8(i), snorm8_to_unorm8_reference(i));
        }
    }

    #[test]
    fn convert_unorm8_to_snorm8() {
        // 129, ..., 255, 0, ..., 127
        for i in 0..=255 {
            assert_eq!(unorm8_to_snorm8(i) as i8, unorm8_to_snorm8_reference(i));
        }
    }

    #[test]
    fn snorm8_unorm8_inverse() {
        for i in 0..=255 {
            if i != 128 {
                assert_eq!(unorm8_to_snorm8(snorm8_to_unorm8(i)), i);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(unorm8_to_snorm8(128), 0);
    }

    #[test]
    fn snorm8_unorm8_float() {
        for i in 0..=255 {
            if i != 128 {
                assert_eq!(float_to_snorm8(snorm8_to_float(i)), i as i8);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(snorm8_to_float(128), -1.0);
    }

    fn unorm4_to_unorm8_reference(x: u8) -> u8 {
        (x as f32 / 15.0 * 255.0).round() as u8
    }

    fn unorm8_to_unorm4_reference(x: u8) -> u8 {
        (x as f32 / 255.0 * 15.0).round() as u8
    }

    #[test]
    fn convert_unorm8_to_unorm4() {
        for i in 0..=255 {
            assert_eq!(unorm8_to_unorm4(i), unorm8_to_unorm4_reference(i));
        }
    }

    #[test]
    fn convert_unorm4_to_unorm8() {
        for i in 0..=15 {
            assert_eq!(unorm4_to_unorm8(i), unorm4_to_unorm8_reference(i));
        }
    }

    fn unorm16_to_unorm8_reference(x: u16) -> u8 {
        (x as f32 / 65535.0 * 255.0).round() as u8
    }

    fn unorm8_to_unorm16_reference(x: u8) -> u16 {
        (x as f32 / 255.0 * 65535.0).round() as u16
    }

    #[test]
    fn convert_unorm8_to_unorm16() {
        for i in 0..=255 {
            assert_eq!(unorm8_to_unorm16(i), unorm8_to_unorm16_reference(i));
        }
    }

    #[test]
    fn convert_unorm16_to_unorm8() {
        for i in 0..=65535 {
            assert_eq!(unorm16_to_unorm8(i), unorm16_to_unorm8_reference(i));
        }
    }

    #[test]
    fn snorm16_unorm16_float() {
        for i in 0..=65535 {
            if i != 32768 {
                assert_eq!(float_to_snorm16(snorm16_to_float(i)), i as i16);
            }
        }
        // Explictly test the value with no true inverse.
        assert_eq!(snorm16_to_float(32768), -1.0);
    }
}
