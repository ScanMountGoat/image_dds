use ddsfile::{D3DFormat, DxgiFormat, FourCC};
use thiserror::Error;

use crate::{
    decode_surface_rgba8, encode_surface_rgba8_generated_mipmaps, max_mipmap_count,
    CompressSurfaceError, DecompressSurfaceError, ImageFormat, Quality,
};

#[derive(Debug, Error)]
pub enum CreateDdsError {
    #[error("error creating DDS")]
    Dds(#[from] ddsfile::Error),

    #[error("error compressing surface")]
    CompressSurface(#[from] CompressSurfaceError),
}

/// Encode `image` to a DDS file with the given `format`.
///
/// Mipmaps are automatically generated when `generate_mipmaps` is `true`.
#[cfg(feature = "image")]
pub fn dds_from_image(
    image: &image::RgbaImage,
    format: ImageFormat,
    quality: Quality,
    generate_mipmaps: bool,
) -> Result<ddsfile::Dds, CreateDdsError> {
    // Assume all images are 2D for now.
    dds_from_surface_rgba8(
        image.width(),
        image.height(),
        1,
        image.as_raw(),
        format,
        quality,
        generate_mipmaps,
    )
}

/// Encode a `width` x `height`  RGBA8 surface to a DDS file with the given `format`.
///
/// Mipmaps are automatically generated when `generate_mipmaps` is `true`.
pub fn dds_from_surface_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    rgba8_data: &[u8],
    format: ImageFormat,
    quality: Quality,
    generate_mipmaps: bool,
) -> Result<ddsfile::Dds, CreateDdsError> {
    // TODO: This is also calculated in the function below.
    let num_mipmaps = max_mipmap_count(width.max(height).max(depth));

    let surface_data = encode_surface_rgba8_generated_mipmaps(
        width,
        height,
        depth,
        rgba8_data,
        format,
        quality,
        generate_mipmaps,
    )?;

    let mut dds = ddsfile::Dds::new_dxgi(ddsfile::NewDxgiParams {
        height,
        width,
        depth: if depth > 1 { Some(depth) } else { None },
        format: format.into(),
        mipmap_levels: if generate_mipmaps {
            Some(num_mipmaps)
        } else {
            None
        },
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: if depth > 1 {
            ddsfile::D3D10ResourceDimension::Texture3D
        } else {
            ddsfile::D3D10ResourceDimension::Texture2D
        },
        alpha_mode: ddsfile::AlphaMode::Straight, // TODO: Does this matter?
    })?;

    dds.data = surface_data;

    Ok(dds)
}

// TODO: Add an option to access other layers and mipmaps.
/// Decode the first array layer and mip level from `dds` to an RGBA8 surface.
pub fn decode_surface_rgba8_from_dds(
    dds: &ddsfile::Dds,
) -> Result<Vec<u8>, DecompressSurfaceError> {
    let width = dds.get_width();
    let height = dds.get_height();
    let depth = dds.get_depth();

    let image_format = dds_image_format(dds).ok_or(DecompressSurfaceError::UnrecognizedFormat)?;
    let rgba8_data = decode_surface_rgba8(width, height, depth, &dds.data, image_format)?;

    Ok(rgba8_data)
}

#[cfg(feature = "image")]
/// Decode the first array layer and mip level from `dds` to an RGBA8 image.
pub fn image_from_dds(dds: &ddsfile::Dds) -> Result<image::RgbaImage, crate::CreateImageError> {
    // Arrange depth slices horizontally from left to right.
    let width = dds.get_width() * dds.get_depth();
    let height = dds.get_height();

    let rgba8_data = decode_surface_rgba8_from_dds(dds)?;
    let data_length = rgba8_data.len();

    let image = image::RgbaImage::from_raw(width, height, rgba8_data).ok_or(
        crate::CreateImageError::InvalidSurfaceDimensions {
            width,
            height,
            data_length,
        },
    )?;

    Ok(image)
}

// TODO: Result?
fn dds_image_format(dds: &ddsfile::Dds) -> Option<ImageFormat> {
    // The format can be DXGI, D3D, or specified in the FOURCC.
    let dxgi = dds.get_dxgi_format();
    let d3d = dds.get_d3d_format();
    let fourcc = dds.header.spf.fourcc.as_ref();

    dxgi.and_then(image_format_from_dxgi)
        .or_else(|| d3d.and_then(image_format_from_d3d))
        .or_else(|| fourcc.and_then(image_format_from_fourcc))
}

fn image_format_from_dxgi(format: DxgiFormat) -> Option<ImageFormat> {
    // TODO: Support uncompressed formats.
    match format {
        DxgiFormat::R8_UNorm => Some(ImageFormat::R8Unorm),
        DxgiFormat::R8G8B8A8_UNorm => Some(ImageFormat::R8G8B8A8Unorm),
        DxgiFormat::R8G8B8A8_UNorm_sRGB => Some(ImageFormat::R8G8B8A8Srgb),
        DxgiFormat::R32G32B32A32_Float => Some(ImageFormat::R32G32B32A32Float),
        DxgiFormat::B8G8R8A8_UNorm => Some(ImageFormat::B8G8R8A8Unorm),
        DxgiFormat::B8G8R8A8_UNorm_sRGB => Some(ImageFormat::B8G8R8A8Srgb),
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
    // TODO: Support uncompressed formats.
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
            ImageFormat::R8Unorm => Self::R8_UNorm,
            ImageFormat::R8G8B8A8Unorm => Self::R8G8B8A8_UNorm,
            ImageFormat::R8G8B8A8Srgb => Self::R8G8B8A8_UNorm_sRGB,
            ImageFormat::R32G32B32A32Float => Self::R32G32B32A32_Float,
            ImageFormat::B8G8R8A8Unorm => Self::B8G8R8A8_UNorm,
            ImageFormat::B8G8R8A8Srgb => Self::B8G8R8A8_UNorm_sRGB,
        }
    }
}
