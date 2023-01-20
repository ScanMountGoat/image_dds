use bcn::{CompressSurfaceError, DecompressSurfaceError};
use ddsfile::{D3DFormat, DxgiFormat, FourCC};

// TODO: Module level documentation explaining limitations and showing basic usage.

// TODO: pub use some of the functions?
pub mod bcn;

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

// TODO: Put dds behind a feature flag.
fn image_format_from_dxgi(format: DxgiFormat) -> Option<ImageFormat> {
    match format {
        DxgiFormat::BC1_UNorm => Some(ImageFormat::BC1Unorm),
        DxgiFormat::BC1_UNorm_sRGB => Some(ImageFormat::BC1Srgb),
        DxgiFormat::BC2_UNorm => Some(ImageFormat::BC2Unorm),
        DxgiFormat::BC2_UNorm_sRGB => Some(ImageFormat::BC2Srgb),
        DxgiFormat::BC3_UNorm => Some(ImageFormat::BC3Unorm),
        DxgiFormat::BC3_UNorm_sRGB => Some(ImageFormat::BC3Srgb),
        DxgiFormat::BC4_UNorm => Some(ImageFormat::BC4Unorm),
        DxgiFormat::BC4_SNorm => Some(ImageFormat::BC4Snorm),
        DxgiFormat::BC5_UNorm => Some(ImageFormat::BC5Unorm),
        DxgiFormat::BC5_SNorm => Some(ImageFormat::BC5Snorm),
        DxgiFormat::BC6H_SF16 => Some(ImageFormat::BC6Sfloat),
        DxgiFormat::BC6H_UF16 => Some(ImageFormat::BC6Ufloat),
        DxgiFormat::BC7_UNorm => Some(ImageFormat::BC7Unorm),
        DxgiFormat::BC7_UNorm_sRGB => Some(ImageFormat::BC7Srgb),
        _ => None,
    }
}

fn image_format_from_d3d(format: D3DFormat) -> Option<ImageFormat> {
    match format {
        D3DFormat::DXT1 => Some(ImageFormat::BC1Unorm),
        D3DFormat::DXT2 => Some(ImageFormat::BC2Unorm),
        D3DFormat::DXT3 => Some(ImageFormat::BC2Unorm),
        D3DFormat::DXT4 => Some(ImageFormat::BC3Unorm),
        D3DFormat::DXT5 => Some(ImageFormat::BC3Unorm),
        _ => None,
    }
}

const BC5U: u32 = u32::from_le_bytes(*b"BC5U");
const ATI2: u32 = u32::from_le_bytes(*b"ATI2");

fn image_format_from_fourcc(fourcc: &FourCC) -> Option<ImageFormat> {
    match fourcc.0 {
        FourCC::DXT1 => Some(ImageFormat::BC1Unorm),
        FourCC::DXT2 => Some(ImageFormat::BC2Unorm),
        FourCC::DXT3 => Some(ImageFormat::BC2Unorm),
        FourCC::DXT4 => Some(ImageFormat::BC3Unorm),
        FourCC::DXT5 => Some(ImageFormat::BC3Unorm),
        FourCC::BC4_UNORM => Some(ImageFormat::BC4Unorm),
        FourCC::BC4_SNORM => Some(ImageFormat::BC4Snorm),
        ATI2 | BC5U => Some(ImageFormat::BC5Unorm),
        FourCC::BC5_SNORM => Some(ImageFormat::BC5Snorm),
        _ => None,
    }
}

impl From<ImageFormat> for DxgiFormat {
    fn from(value: ImageFormat) -> Self {
        // TODO: Differentiate between unorm and srgb.
        // TODO: Differentiate between signed and unsigned.
        match value {
            ImageFormat::BC1Unorm => Self::BC1_UNorm,
            ImageFormat::BC1Srgb => Self::BC1_UNorm_sRGB,
            ImageFormat::BC2Unorm => Self::BC2_UNorm,
            ImageFormat::BC2Srgb => Self::BC2_UNorm_sRGB,
            ImageFormat::BC3Unorm => Self::BC3_UNorm,
            ImageFormat::BC3Srgb => Self::BC3_UNorm_sRGB,
            ImageFormat::BC4Unorm => Self::BC4_UNorm,
            ImageFormat::BC4Snorm => Self::BC4_SNorm,
            ImageFormat::BC5Unorm => Self::BC5_UNorm,
            ImageFormat::BC5Snorm => Self::BC5_SNorm,
            ImageFormat::BC6Ufloat => Self::BC6H_UF16,
            ImageFormat::BC6Sfloat => Self::BC6H_SF16,
            ImageFormat::BC7Unorm => Self::BC7_UNorm,
            ImageFormat::BC7Srgb => Self::BC7_UNorm_sRGB,
        }
    }
}

fn max_mipmap_count(max_dimension: u32) -> u32 {
    // log2(x) + 1
    u32::BITS - max_dimension.leading_zeros()
}

pub fn dds_from_image(
    image: &image::RgbaImage,
    format: ImageFormat,
    quality: Quality,
    generate_mipmaps: bool,
) -> Result<ddsfile::Dds, CompressSurfaceError> {
    // TODO: Depth and array layers?
    let width = image.width();
    let height = image.height();

    let num_mipmaps = if generate_mipmaps {
        max_mipmap_count(width.max(height))
    } else {
        1
    };

    let mut dds = ddsfile::Dds::new_dxgi(ddsfile::NewDxgiParams {
        height,
        width,
        depth: None,
        format: format.into(),
        mipmap_levels: if generate_mipmaps {
            Some(num_mipmaps)
        } else {
            None
        },
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: ddsfile::D3D10ResourceDimension::Texture2D,
        alpha_mode: ddsfile::AlphaMode::Straight, // TODO: Does this matter?
    })
    .unwrap();

    // TODO: Avoid initial clone by starting from 1?
    let mut mip_image = image.clone();

    let mut surface_data = Vec::new();

    // TODO: make this a function that works on raw buffers without the image crate.
    // TODO: make this generic to also handle half precision data for BC6?
    // TODO: Initial width/height should be a multiple of 4.
    // TODO How do applications handle this?
    for i in 0..num_mipmaps {
        let mip_width = (width >> i).max(1);
        let mip_height = (height >> i).max(1);

        // TODO: Is there a faster way to write this by hand?
        // Downsample and blur using linear interpolation.
        // Nearest causes aliasing and defeats the benefits of mipmapping.
        let filter = image::imageops::FilterType::Triangle;

        // The physical size must be at least 4x4 to have enough data for a full block.
        // Applications or the GPU will use the smaller virtual size and ignore padding.
        // https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression
        mip_image =
            image::imageops::resize(&mip_image, mip_width.max(4), mip_height.max(4), filter);

        // TODO: How to handle padding for mips smaller than 1x1?
        // TODO: should width/height be a multiple of the block dimensions?
        // TODO: Avoid clone?
        let rgba_data = mip_image.as_raw();

        let mip_data = bcn::bcn_from_rgba8(
            mip_width.max(4),
            mip_height.max(4),
            rgba_data,
            format.into(),
            quality,
        )?;
        surface_data.extend_from_slice(&mip_data);
    }

    dds.data = surface_data;

    Ok(dds)
}

pub fn image_from_dds(dds: &ddsfile::Dds) -> Result<image::RgbaImage, DecompressSurfaceError> {
    // TODO: Mipmaps, depth, and array layers?
    let image_format = dds_image_format(dds).ok_or(DecompressSurfaceError::UnrecognizedFormat)?;

    let rgba = bcn::rgba8_from_bcn(
        dds.get_width(),
        dds.get_height(),
        &dds.data,
        image_format.into(),
    )?;

    // TODO: Avoid unwrap.
    let image = image::RgbaImage::from_raw(dds.get_width(), dds.get_height(), rgba).unwrap();

    Ok(image)
}

// TODO: Result?
fn dds_image_format(dds: &ddsfile::Dds) -> Option<ImageFormat> {
    // TODO: dxgi -> d3d -> fourcc -> error
    dds.get_dxgi_format()
        .and_then(image_format_from_dxgi)
        .or_else(|| dds.get_d3d_format().and_then(image_format_from_d3d))
        .or_else(|| {
            dds.header
                .spf
                .fourcc
                .as_ref()
                .and_then(image_format_from_fourcc)
        })
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
}
