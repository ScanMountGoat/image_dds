use ddsfile::{D3DFormat, Dds, DxgiFormat, FourCC};
use thiserror::Error;

use crate::{
    decode_surface_rgba8, encode_surface_rgba8, mip_dimension, CompressSurfaceError,
    CreateImageError, DecompressSurfaceError, ImageFormat, Mipmaps, Quality, Surface, SurfaceRgba8,
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
/// The number of mipmaps generated depends on the `mipmaps` parameter.
#[cfg(feature = "image")]
pub fn dds_from_image(
    image: &image::RgbaImage,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Dds, CreateDdsError> {
    // Assume all images are 2D for now.
    // TODO: 3d and cube map support?
    dds_from_surface_rgba8(
        SurfaceRgba8 {
            width: image.width(),
            height: image.height(),
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: image.as_raw(),
        },
        format,
        quality,
        mipmaps,
    )
}

/// Encode a `width` x `height` x `depth` RGBA8 surface to a DDS file with the given `format`.
///
/// The number of mipmaps generated depends on the `mipmaps` parameter.
pub fn dds_from_surface_rgba8<T: AsRef<[u8]>>(
    surface: SurfaceRgba8<T>,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Dds, CreateDdsError> {
    let Surface {
        width,
        height,
        depth,
        layers,
        mipmaps,
        image_format,
        data,
    } = encode_surface_rgba8(surface, format, quality, mipmaps)?;

    let mut dds = Dds::new_dxgi(ddsfile::NewDxgiParams {
        height,
        width,
        depth: if depth > 1 { Some(depth) } else { None },
        format: image_format.into(),
        mipmap_levels: if mipmaps > 1 { Some(mipmaps) } else { None },
        array_layers: if layers > 1 { Some(layers) } else { None },
        caps2: None,
        is_cubemap: false,
        resource_dimension: if depth > 1 {
            ddsfile::D3D10ResourceDimension::Texture3D
        } else {
            ddsfile::D3D10ResourceDimension::Texture2D
        },
        alpha_mode: ddsfile::AlphaMode::Straight, // TODO: Does this matter?
    })?;

    dds.data = data;

    Ok(dds)
}

/// Decode all layers and mipmaps from `dds` to an RGBA8 surface.
pub fn decode_surface_rgba8_from_dds(
    dds: &Dds,
) -> Result<SurfaceRgba8<Vec<u8>>, DecompressSurfaceError> {
    let width = dds.get_width();
    let height = dds.get_height();
    let depth = dds.get_depth();
    let layers = array_layer_count(dds);
    let mipmaps = dds.get_num_mipmap_levels();

    let image_format = dds_image_format(dds).ok_or(DecompressSurfaceError::UnrecognizedFormat)?;
    decode_surface_rgba8(Surface {
        width,
        height,
        depth,
        layers,
        mipmaps,
        image_format,
        data: &dds.data,
    })
}

#[cfg(feature = "image")]
/// Decode the given mip level from `dds` to an RGBA8 image.
/// Array layers are arranged vertically from top to bottom.
pub fn image_from_dds(dds: &Dds, mipmap: u32) -> Result<image::RgbaImage, CreateImageError> {
    let surface = decode_surface_rgba8_from_dds(dds)?;

    // Mipmaps have different dimensions.
    // A single 2D image can only represent data from a single mip level across layers.
    let image_data: Vec<_> = (0..surface.layers)
        .flat_map(|layer| surface.get(layer, mipmap).unwrap())
        .copied()
        .collect();
    let data_length = image_data.len();

    // Arrange depth slices horizontally and array layers vertically.
    let width = mip_dimension(surface.width, mipmap) * mip_dimension(surface.depth, mipmap);
    let height = mip_dimension(surface.height, mipmap) * surface.layers;

    let image = image::RgbaImage::from_raw(width, height, image_data).ok_or(
        crate::CreateImageError::InvalidSurfaceDimensions {
            width,
            height,
            data_length,
        },
    )?;

    Ok(image)
}

fn array_layer_count(dds: &Dds) -> u32 {
    // Array layers for DDS are calculated differently for cube maps.
    if matches!(&dds.header10, Some(header10) if header10.misc_flag == ddsfile::MiscFlag::TEXTURECUBE)
    {
        dds.get_num_array_layers() * 6
    } else {
        dds.get_num_array_layers()
    }
}

fn dds_image_format(dds: &Dds) -> Option<ImageFormat> {
    // The format can be DXGI, D3D, or specified in the FOURCC.
    let dxgi = dds.get_dxgi_format();
    let d3d = dds.get_d3d_format();
    let fourcc = dds.header.spf.fourcc.as_ref();

    dxgi.and_then(image_format_from_dxgi)
        .or_else(|| d3d.and_then(image_format_from_d3d))
        .or_else(|| fourcc.and_then(image_format_from_fourcc))
}

fn image_format_from_dxgi(format: DxgiFormat) -> Option<ImageFormat> {
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
