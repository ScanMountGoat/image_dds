use ddsfile::{D3DFormat, DxgiFormat, FourCC};
use thiserror::Error;

use crate::{
    bcn::{CompressSurfaceError, DecompressSurfaceError},
    encode_surface_rgba8_generated_mipmaps, max_mipmap_count, ImageFormat, Quality, decode_surface_rgba8,
};

#[derive(Debug, Error)]
pub enum CreateDdsError {
    #[error("error creating DDS")]
    Dds(#[from] ddsfile::Error),

    #[error("error compressing surface")]
    CompressSurface(#[from] CompressSurfaceError),
}

// TODO: Add variants that don't require the image crate.
pub fn dds_from_image(
    image: &image::RgbaImage,
    format: ImageFormat,
    quality: Quality,
    generate_mipmaps: bool,
) -> Result<ddsfile::Dds, CreateDdsError> {
    let width = image.width();
    let height = image.height();

    // TODO: This is also calculated in the function below.
    let num_mipmaps = max_mipmap_count(width.max(height));

    let surface_data = encode_surface_rgba8_generated_mipmaps(
        width,
        height,
        image.as_raw(),
        format,
        quality,
        generate_mipmaps,
    )?;

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
        resource_dimension: ddsfile::D3D10ResourceDimension::Texture2D, // TODO: Support 3D
        alpha_mode: ddsfile::AlphaMode::Straight,                       // TODO: Does this matter?
    })?;

    dds.data = surface_data;

    Ok(dds)
}

pub fn image_from_dds(dds: &ddsfile::Dds) -> Result<image::RgbaImage, crate::CreateImageError> {
    // TODO: Mipmaps, depth, and array layers?
    let image_format = dds_image_format(dds).ok_or(DecompressSurfaceError::UnrecognizedFormat)?;

    let width = dds.get_width();
    let height = dds.get_height();

    // TODO: Create a function decode_rgba8?
    let rgba8_data = decode_surface_rgba8(width, height, &dds.data, image_format.into())?;
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

fn image_format_from_dxgi(format: DxgiFormat) -> Option<ImageFormat> {
    // TODO: Support uncompressed formats.
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
