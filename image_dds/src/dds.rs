use std::ops::Range;

use ddsfile::{Caps2, D3DFormat, Dds, DxgiFormat, FourCC};
use thiserror::Error;

use crate::{
    CreateImageError, ImageFormat, Mipmaps, Quality, Surface, SurfaceError, SurfaceRgba32Float,
    SurfaceRgba8,
};

/// Errors that can occur when converting to DDS.
#[derive(Debug, Error)]
pub enum CreateDdsError {
    #[error("error creating DDS: {0}")]
    Dds(#[from] ddsfile::Error),

    #[error("error compressing surface: {0}")]
    CompressSurface(#[from] SurfaceError),
}

#[cfg(feature = "encode")]
#[cfg(feature = "image")]
/// Encode `image` to a 2D DDS file with the given `format`.
///
/// The number of mipmaps generated depends on the `mipmaps` parameter.
pub fn dds_from_image(
    image: &image::RgbaImage,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Dds, CreateDdsError> {
    // Assume all images are 2D for now.
    SurfaceRgba8::from_image(image)
        .encode(format, quality, mipmaps)?
        .to_dds()
}

#[cfg(feature = "encode")]
#[cfg(feature = "image")]
/// Encode `image` to a 2D DDS file with the given `format`.
///
/// The number of mipmaps generated depends on the `mipmaps` parameter.
pub fn dds_from_imagef32(
    image: &image::Rgba32FImage,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Dds, CreateDdsError> {
    // Assume all images are 2D for now.
    SurfaceRgba32Float::from_image(image)
        .encode(format, quality, mipmaps)?
        .to_dds()
}

#[cfg(feature = "image")]
/// Decode the given mip level from `dds` to an RGBA8 image.
/// Array layers are arranged vertically from top to bottom.
pub fn image_from_dds(dds: &Dds, mipmap: u32) -> Result<image::RgbaImage, CreateImageError> {
    let layers = array_layer_count(dds);
    SurfaceRgba8::decode_layers_mipmaps_dds(dds, 0..layers, mipmap..mipmap + 1)?.into_image()
}

#[cfg(feature = "image")]
/// Decode the given mip level from `dds` to an RGBAF32 image.
/// Array layers are arranged vertically from top to bottom.
pub fn imagef32_from_dds(dds: &Dds, mipmap: u32) -> Result<image::Rgba32FImage, CreateImageError> {
    let layers = array_layer_count(dds);
    SurfaceRgba32Float::decode_layers_mipmaps_dds(dds, 0..layers, mipmap..mipmap + 1)?.into_image()
}

impl<T: AsRef<[u8]>> Surface<T> {
    /// Create a DDS file with the same image data and format.
    pub fn to_dds(&self) -> Result<crate::ddsfile::Dds, CreateDdsError> {
        let mut dds = Dds::new_dxgi(ddsfile::NewDxgiParams {
            height: self.height,
            width: self.width,
            depth: if self.depth > 1 {
                Some(self.depth)
            } else {
                None
            },
            format: self.image_format.into(),
            mipmap_levels: (self.mipmaps > 1).then_some(self.mipmaps),
            array_layers: (self.layers > 1 && self.layers != 6).then_some(self.layers),
            caps2: (self.layers == 6).then_some(Caps2::CUBEMAP | Caps2::CUBEMAP_ALLFACES),
            is_cubemap: self.layers == 6,
            resource_dimension: if self.depth > 1 {
                ddsfile::D3D10ResourceDimension::Texture3D
            } else {
                ddsfile::D3D10ResourceDimension::Texture2D
            },
            alpha_mode: ddsfile::AlphaMode::Straight,
        })?;

        dds.data = self.data.as_ref().to_vec();

        Ok(dds)
    }
}

impl<'a> Surface<&'a [u8]> {
    /// Create a view over the data in `dds` without any copies.
    pub fn from_dds(dds: &'a crate::ddsfile::Dds) -> Result<Self, SurfaceError> {
        let width = dds.get_width();
        let height = dds.get_height();
        let depth = dds.get_depth();
        let layers = array_layer_count(dds);
        let mipmaps = dds.get_num_mipmap_levels();
        let image_format = dds_image_format(dds).map_err(SurfaceError::UnsupportedDdsFormat)?;

        Ok(Surface {
            width,
            height,
            depth,
            layers,
            mipmaps,
            image_format,
            data: &dds.data,
        })
    }
}

#[cfg(feature = "encode")]
impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    /// Encode a `width` x `height` x `depth` RGBA8 surface to a DDS file with the given `format`.
    ///
    /// The number of mipmaps generated depends on the `mipmaps` parameter.
    pub fn encode_dds(
        &self,
        format: ImageFormat,
        quality: Quality,
        mipmaps: Mipmaps,
    ) -> Result<Dds, CreateDdsError> {
        self.encode(format, quality, mipmaps)?.to_dds()
    }
}

impl SurfaceRgba8<Vec<u8>> {
    /// Decode all layers and mipmaps from `dds` to an RGBA8 surface.
    pub fn decode_dds(dds: &Dds) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        Surface::from_dds(dds)?.decode_rgba8()
    }

    /// Decode a specific range of layers and mipmaps from `dds` to an RGBA8 surface.
    pub fn decode_layers_mipmaps_dds(
        dds: &Dds,
        layers: Range<u32>,
        mipmaps: Range<u32>,
    ) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        Surface::from_dds(dds)?.decode_layers_mipmaps_rgba8(layers, mipmaps)
    }
}

impl SurfaceRgba32Float<Vec<f32>> {
    /// Decode all layers and mipmaps from `dds` to an RGBAF32 surface.
    pub fn decode_dds(dds: &Dds) -> Result<SurfaceRgba32Float<Vec<f32>>, SurfaceError> {
        Surface::from_dds(dds)?.decode_rgbaf32()
    }

    /// Decode a specific range of layers and mipmaps from `dds` to an RGBAF32 surface.
    pub fn decode_layers_mipmaps_dds(
        dds: &Dds,
        layers: Range<u32>,
        mipmaps: Range<u32>,
    ) -> Result<SurfaceRgba32Float<Vec<f32>>, SurfaceError> {
        Surface::from_dds(dds)?.decode_layers_mipmaps_rgbaf32(layers, mipmaps)
    }
}

fn array_layer_count(dds: &Dds) -> u32 {
    // Array layers for DDS are calculated differently for cube maps.
    if matches!(&dds.header10, Some(header10) if header10.misc_flag == ddsfile::MiscFlag::TEXTURECUBE)
    {
        dds.get_num_array_layers().max(1) * 6
    } else {
        dds.get_num_array_layers().max(1)
    }
}

/// Format information for all DDS variants.
#[derive(Debug)]
pub struct DdsFormatInfo {
    pub dxgi: Option<DxgiFormat>,
    pub d3d: Option<D3DFormat>,
    pub fourcc: Option<FourCC>,
}

/// Returns the format of `dds` or `None` if the format is unrecognized.
pub fn dds_image_format(dds: &Dds) -> Result<ImageFormat, DdsFormatInfo> {
    // The format can be DXGI, D3D, or specified in the FOURCC.
    let dxgi = dds.get_dxgi_format();
    let d3d = dds.get_d3d_format();
    let fourcc = dds.header.spf.fourcc.clone();

    d3d.and_then(image_format_from_d3d)
        .or_else(|| dxgi.and_then(image_format_from_dxgi))
        .or_else(|| fourcc.clone().and_then(image_format_from_fourcc))
        .ok_or(DdsFormatInfo { dxgi, d3d, fourcc })
}

fn image_format_from_dxgi(format: DxgiFormat) -> Option<ImageFormat> {
    match format {
        DxgiFormat::R8_UNorm => Some(ImageFormat::R8Unorm),
        DxgiFormat::R8G8B8A8_UNorm => Some(ImageFormat::Rgba8Unorm),
        DxgiFormat::R8G8B8A8_UNorm_sRGB => Some(ImageFormat::Rgba8UnormSrgb),
        DxgiFormat::R16G16B16A16_Float => Some(ImageFormat::Rgba16Float),
        DxgiFormat::R32G32B32A32_Float => Some(ImageFormat::Rgba32Float),
        DxgiFormat::B8G8R8A8_UNorm => Some(ImageFormat::Bgra8Unorm),
        DxgiFormat::B8G8R8A8_UNorm_sRGB => Some(ImageFormat::Bgra8UnormSrgb),
        DxgiFormat::BC1_UNorm => Some(ImageFormat::BC1RgbaUnorm),
        DxgiFormat::BC1_UNorm_sRGB => Some(ImageFormat::BC1RgbaUnormSrgb),
        DxgiFormat::BC2_UNorm => Some(ImageFormat::BC2RgbaUnorm),
        DxgiFormat::BC2_UNorm_sRGB => Some(ImageFormat::BC2RgbaUnormSrgb),
        DxgiFormat::BC3_UNorm => Some(ImageFormat::BC3RgbaUnorm),
        DxgiFormat::BC3_UNorm_sRGB => Some(ImageFormat::BC3RgbaUnormSrgb),
        DxgiFormat::BC4_UNorm => Some(ImageFormat::BC4RUnorm),
        DxgiFormat::BC4_SNorm => Some(ImageFormat::BC4RSnorm),
        DxgiFormat::BC5_UNorm => Some(ImageFormat::BC5RgUnorm),
        DxgiFormat::BC5_SNorm => Some(ImageFormat::BC5RgSnorm),
        DxgiFormat::BC6H_SF16 => Some(ImageFormat::BC6hRgbSfloat),
        DxgiFormat::BC6H_UF16 => Some(ImageFormat::BC6hRgbUfloat),
        DxgiFormat::BC7_UNorm => Some(ImageFormat::BC7RgbaUnorm),
        DxgiFormat::BC7_UNorm_sRGB => Some(ImageFormat::BC7RgbaUnormSrgb),
        DxgiFormat::B4G4R4A4_UNorm => Some(ImageFormat::Bgra4Unorm),
        _ => None,
    }
}

fn image_format_from_d3d(format: D3DFormat) -> Option<ImageFormat> {
    match format {
        D3DFormat::DXT1 => Some(ImageFormat::BC1RgbaUnorm),
        D3DFormat::DXT2 => Some(ImageFormat::BC2RgbaUnorm),
        D3DFormat::DXT3 => Some(ImageFormat::BC2RgbaUnorm),
        D3DFormat::DXT4 => Some(ImageFormat::BC3RgbaUnorm),
        D3DFormat::DXT5 => Some(ImageFormat::BC3RgbaUnorm),
        // BGRA is also ARGB depending on how we look at the bytes.
        D3DFormat::A4R4G4B4 => Some(ImageFormat::Bgra4Unorm),
        D3DFormat::A8R8G8B8 => Some(ImageFormat::Bgra8Unorm),
        _ => None,
    }
}

const BC5U: u32 = u32::from_le_bytes(*b"BC5U");
const ATI2: u32 = u32::from_le_bytes(*b"ATI2");

fn image_format_from_fourcc(fourcc: FourCC) -> Option<ImageFormat> {
    match fourcc.0 {
        FourCC::DXT1 => Some(ImageFormat::BC1RgbaUnorm),
        FourCC::DXT2 => Some(ImageFormat::BC2RgbaUnorm),
        FourCC::DXT3 => Some(ImageFormat::BC2RgbaUnorm),
        FourCC::DXT4 => Some(ImageFormat::BC3RgbaUnorm),
        FourCC::DXT5 => Some(ImageFormat::BC3RgbaUnorm),
        FourCC::BC4_UNORM => Some(ImageFormat::BC4RUnorm),
        FourCC::BC4_SNORM => Some(ImageFormat::BC4RSnorm),
        ATI2 | BC5U => Some(ImageFormat::BC5RgUnorm),
        FourCC::BC5_SNORM => Some(ImageFormat::BC5RgSnorm),
        _ => None,
    }
}

impl From<ImageFormat> for DxgiFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::BC1RgbaUnorm => Self::BC1_UNorm,
            ImageFormat::BC1RgbaUnormSrgb => Self::BC1_UNorm_sRGB,
            ImageFormat::BC2RgbaUnorm => Self::BC2_UNorm,
            ImageFormat::BC2RgbaUnormSrgb => Self::BC2_UNorm_sRGB,
            ImageFormat::BC3RgbaUnorm => Self::BC3_UNorm,
            ImageFormat::BC3RgbaUnormSrgb => Self::BC3_UNorm_sRGB,
            ImageFormat::BC4RUnorm => Self::BC4_UNorm,
            ImageFormat::BC4RSnorm => Self::BC4_SNorm,
            ImageFormat::BC5RgUnorm => Self::BC5_UNorm,
            ImageFormat::BC5RgSnorm => Self::BC5_SNorm,
            ImageFormat::BC6hRgbUfloat => Self::BC6H_UF16,
            ImageFormat::BC6hRgbSfloat => Self::BC6H_SF16,
            ImageFormat::BC7RgbaUnorm => Self::BC7_UNorm,
            ImageFormat::BC7RgbaUnormSrgb => Self::BC7_UNorm_sRGB,
            ImageFormat::R8Unorm => Self::R8_UNorm,
            ImageFormat::Rgba8Unorm => Self::R8G8B8A8_UNorm,
            ImageFormat::Rgba8UnormSrgb => Self::R8G8B8A8_UNorm_sRGB,
            ImageFormat::Rgba16Float => Self::R16G16B16A16_Float,
            ImageFormat::Rgba32Float => Self::R32G32B32A32_Float,
            ImageFormat::Bgra8Unorm => Self::B8G8R8A8_UNorm,
            ImageFormat::Bgra8UnormSrgb => Self::B8G8R8A8_UNorm_sRGB,
            ImageFormat::Bgra4Unorm => Self::B4G4R4A4_UNorm,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dds_to_from_surface() {
        let surface = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8Unorm,
            data: [0u8; 4 * 4 * 4].as_slice(),
        };
        assert_eq!(
            surface,
            Surface::from_dds(&surface.to_dds().unwrap()).unwrap()
        );
    }

    #[test]
    fn dds_to_from_surface_cube() {
        let surface = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 6,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8Unorm,
            data: [0u8; 4 * 4 * 6 * 4].as_slice(),
        };
        assert_eq!(
            surface,
            Surface::from_dds(&surface.to_dds().unwrap()).unwrap()
        );
    }
}
