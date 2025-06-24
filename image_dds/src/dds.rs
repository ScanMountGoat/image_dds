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
    ///
    /// Creates a DXGI DDS for most formats and D3D DDS for some legacy formats.
    pub fn to_dds(&self) -> Result<crate::ddsfile::Dds, CreateDdsError> {
        let mut dds = dxgi_from_image_format(self.image_format)
            .map(|format| {
                Dds::new_dxgi(ddsfile::NewDxgiParams {
                    height: self.height,
                    width: self.width,
                    depth: if self.depth > 1 {
                        Some(self.depth)
                    } else {
                        None
                    },
                    format,
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
                })
            })
            .or_else(|| {
                // Not all surface formats are supported by DXGI.
                d3d_from_image_format(self.image_format).map(|format| {
                    Dds::new_d3d(ddsfile::NewD3dParams {
                        height: self.height,
                        width: self.width,
                        depth: if self.depth > 1 {
                            Some(self.depth)
                        } else {
                            None
                        },
                        format,
                        mipmap_levels: (self.mipmaps > 1).then_some(self.mipmaps),
                        caps2: (self.layers == 6)
                            .then_some(Caps2::CUBEMAP | Caps2::CUBEMAP_ALLFACES),
                    })
                })
            })
            .unwrap()?;

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
#[derive(Debug, PartialEq)]
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
        DxgiFormat::R8_SNorm => Some(ImageFormat::R8Snorm),
        DxgiFormat::R8G8_UNorm => Some(ImageFormat::Rg8Unorm),
        DxgiFormat::R8G8_SNorm => Some(ImageFormat::Rg8Snorm),
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
        DxgiFormat::R16G16B16A16_UNorm => Some(ImageFormat::Rgba16Unorm),
        DxgiFormat::R16G16B16A16_SNorm => Some(ImageFormat::Rgba16Snorm),
        DxgiFormat::R16G16_UNorm => Some(ImageFormat::Rg16Unorm),
        DxgiFormat::R16G16_SNorm => Some(ImageFormat::Rg16Snorm),
        DxgiFormat::R16_UNorm => Some(ImageFormat::R16Unorm),
        DxgiFormat::R16_SNorm => Some(ImageFormat::R16Snorm),
        DxgiFormat::R16_Float => Some(ImageFormat::R16Float),
        DxgiFormat::R16G16_Float => Some(ImageFormat::Rg16Float),
        DxgiFormat::R32_Float => Some(ImageFormat::R32Float),
        DxgiFormat::R32G32_Float => Some(ImageFormat::Rg32Float),
        DxgiFormat::R8G8B8A8_SNorm => Some(ImageFormat::Rgba8Snorm),
        DxgiFormat::R32G32B32_Float => Some(ImageFormat::Rgb32Float),
        DxgiFormat::B5G5R5A1_UNorm => Some(ImageFormat::Bgr5A1Unorm),
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
        // BGRA can also be written ARGB depending on how we look at the bytes.
        D3DFormat::A4R4G4B4 => Some(ImageFormat::Bgra4Unorm),
        D3DFormat::A8R8G8B8 => Some(ImageFormat::Bgra8Unorm),
        D3DFormat::R8G8B8 => Some(ImageFormat::Bgr8Unorm),
        D3DFormat::A8B8G8R8 => Some(ImageFormat::Rgba8Unorm),
        D3DFormat::G16R16F => Some(ImageFormat::Rg16Float),
        D3DFormat::A16B16G16R16F => Some(ImageFormat::Rgba16Float),
        D3DFormat::G32R32F => Some(ImageFormat::Rg32Float),
        D3DFormat::A32B32G32R32F => Some(ImageFormat::Rgba32Float),
        D3DFormat::G16R16 => Some(ImageFormat::Rg16Unorm),
        D3DFormat::A16B16G16R16 => Some(ImageFormat::Rgba16Unorm),
        D3DFormat::A1R5G5B5 => Some(ImageFormat::Bgr5A1Unorm),
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

fn d3d_from_image_format(value: ImageFormat) -> Option<D3DFormat> {
    // bc4 and bc5 are handled by fourcc.
    match value {
        ImageFormat::BC1RgbaUnorm => Some(D3DFormat::DXT1),
        ImageFormat::BC1RgbaUnormSrgb => Some(D3DFormat::DXT1),
        ImageFormat::BC2RgbaUnorm => Some(D3DFormat::DXT2),
        ImageFormat::BC2RgbaUnormSrgb => Some(D3DFormat::DXT2),
        ImageFormat::BC3RgbaUnorm => Some(D3DFormat::DXT5),
        ImageFormat::BC3RgbaUnormSrgb => Some(D3DFormat::DXT5),
        ImageFormat::BC4RUnorm => None,
        ImageFormat::BC4RSnorm => None,
        ImageFormat::BC5RgUnorm => None,
        ImageFormat::BC5RgSnorm => None,
        ImageFormat::BC6hRgbUfloat => None,
        ImageFormat::BC6hRgbSfloat => None,
        ImageFormat::BC7RgbaUnorm => None,
        ImageFormat::BC7RgbaUnormSrgb => None,
        ImageFormat::R8Unorm => None,
        ImageFormat::R8Snorm => None,
        ImageFormat::Rg8Unorm => None,
        ImageFormat::Rg8Snorm => None,
        ImageFormat::Rgba8Unorm => Some(D3DFormat::A8B8G8R8),
        ImageFormat::Rgba8UnormSrgb => Some(D3DFormat::A8B8G8R8),
        ImageFormat::Rgba16Float => Some(D3DFormat::A16B16G16R16F),
        ImageFormat::Rgba32Float => Some(D3DFormat::A32B32G32R32F),
        ImageFormat::Bgra8Unorm => Some(D3DFormat::A8R8G8B8),
        ImageFormat::Bgra8UnormSrgb => Some(D3DFormat::A8R8G8B8),
        ImageFormat::Bgra4Unorm => Some(D3DFormat::A4R4G4B4),
        ImageFormat::Bgr8Unorm => Some(D3DFormat::R8G8B8),
        ImageFormat::R16Unorm => None,
        ImageFormat::R16Snorm => None,
        ImageFormat::Rg16Unorm => Some(D3DFormat::G16R16),
        ImageFormat::Rg16Snorm => None,
        ImageFormat::Rgba16Unorm => Some(D3DFormat::A16B16G16R16),
        ImageFormat::Rgba16Snorm => None,
        ImageFormat::Rg16Float => Some(D3DFormat::G16R16F),
        ImageFormat::Rg32Float => Some(D3DFormat::G32R32F),
        ImageFormat::R16Float => Some(D3DFormat::R16F),
        ImageFormat::R32Float => Some(D3DFormat::R32F),
        ImageFormat::Rgba8Snorm => None,
        ImageFormat::Rgb32Float => None,
        ImageFormat::Bgr5A1Unorm => Some(D3DFormat::A1R5G5B5),
    }
}

fn dxgi_from_image_format(value: ImageFormat) -> Option<DxgiFormat> {
    match value {
        ImageFormat::BC1RgbaUnorm => Some(DxgiFormat::BC1_UNorm),
        ImageFormat::BC1RgbaUnormSrgb => Some(DxgiFormat::BC1_UNorm_sRGB),
        ImageFormat::BC2RgbaUnorm => Some(DxgiFormat::BC2_UNorm),
        ImageFormat::BC2RgbaUnormSrgb => Some(DxgiFormat::BC2_UNorm_sRGB),
        ImageFormat::BC3RgbaUnorm => Some(DxgiFormat::BC3_UNorm),
        ImageFormat::BC3RgbaUnormSrgb => Some(DxgiFormat::BC3_UNorm_sRGB),
        ImageFormat::BC4RUnorm => Some(DxgiFormat::BC4_UNorm),
        ImageFormat::BC4RSnorm => Some(DxgiFormat::BC4_SNorm),
        ImageFormat::BC5RgUnorm => Some(DxgiFormat::BC5_UNorm),
        ImageFormat::BC5RgSnorm => Some(DxgiFormat::BC5_SNorm),
        ImageFormat::BC6hRgbUfloat => Some(DxgiFormat::BC6H_UF16),
        ImageFormat::BC6hRgbSfloat => Some(DxgiFormat::BC6H_SF16),
        ImageFormat::BC7RgbaUnorm => Some(DxgiFormat::BC7_UNorm),
        ImageFormat::BC7RgbaUnormSrgb => Some(DxgiFormat::BC7_UNorm_sRGB),
        ImageFormat::R8Unorm => Some(DxgiFormat::R8_UNorm),
        ImageFormat::R8Snorm => Some(DxgiFormat::R8_SNorm),
        ImageFormat::Rg8Unorm => Some(DxgiFormat::R8G8_UNorm),
        ImageFormat::Rg8Snorm => Some(DxgiFormat::R8G8_SNorm),
        ImageFormat::Rgba8Unorm => Some(DxgiFormat::R8G8B8A8_UNorm),
        ImageFormat::Rgba8UnormSrgb => Some(DxgiFormat::R8G8B8A8_UNorm_sRGB),
        ImageFormat::Rgba16Float => Some(DxgiFormat::R16G16B16A16_Float),
        ImageFormat::Rgba32Float => Some(DxgiFormat::R32G32B32A32_Float),
        ImageFormat::Bgra8Unorm => Some(DxgiFormat::B8G8R8A8_UNorm),
        ImageFormat::Bgra8UnormSrgb => Some(DxgiFormat::B8G8R8A8_UNorm_sRGB),
        ImageFormat::Bgra4Unorm => Some(DxgiFormat::B4G4R4A4_UNorm),
        ImageFormat::Bgr8Unorm => None,
        ImageFormat::R16Unorm => Some(DxgiFormat::R16_UNorm),
        ImageFormat::R16Snorm => Some(DxgiFormat::R16_SNorm),
        ImageFormat::Rg16Unorm => Some(DxgiFormat::R16G16_UNorm),
        ImageFormat::Rg16Snorm => Some(DxgiFormat::R16G16_SNorm),
        ImageFormat::Rgba16Unorm => Some(DxgiFormat::R16G16B16A16_UNorm),
        ImageFormat::Rgba16Snorm => Some(DxgiFormat::R16G16B16A16_SNorm),
        ImageFormat::Rg16Float => Some(DxgiFormat::R16G16_Float),
        ImageFormat::Rg32Float => Some(DxgiFormat::R32G32_Float),
        ImageFormat::R16Float => Some(DxgiFormat::R16_Float),
        ImageFormat::R32Float => Some(DxgiFormat::R32_Float),
        ImageFormat::Rgba8Snorm => Some(DxgiFormat::R8G8B8A8_SNorm),
        ImageFormat::Rgb32Float => Some(DxgiFormat::R32G32B32_Float),
        ImageFormat::Bgr5A1Unorm => Some(DxgiFormat::B5G5R5A1_UNorm),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn dds_to_from_surface() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 4 * 6 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            assert_eq!(
                surface,
                Surface::from_dds(&surface.to_dds().unwrap()).unwrap()
            );
        }
    }

    #[test]
    fn dds_to_from_surface_cube() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 4 * 6 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 6,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            assert_eq!(
                surface,
                Surface::from_dds(&surface.to_dds().unwrap()).unwrap()
            );
        }
    }

    #[test]
    fn dds_to_from_surface_3d() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 4 * 4 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 4,
                depth: 4,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            assert_eq!(
                surface,
                Surface::from_dds(&surface.to_dds().unwrap()).unwrap()
            );
        }
    }

    #[test]
    fn dds_from_image_formats() {
        let image = image::RgbaImage::new(4, 8);
        for image_format in ImageFormat::iter() {
            let dds = dds_from_image(
                &image,
                image_format,
                Quality::Fast,
                Mipmaps::GeneratedAutomatic,
            )
            .unwrap();
            assert_eq!(4, dds.get_width());
            assert_eq!(8, dds.get_height());
        }
    }

    #[test]
    fn image_from_dds_formats() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 8 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 8,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            let dds = surface.to_dds().unwrap();

            let image = image_from_dds(&dds, 0).unwrap();
            assert_eq!((4, 8), image.dimensions());
        }
    }

    #[test]
    fn dds_from_imagef32_formats() {
        let image = image::Rgba32FImage::new(4, 8);
        for image_format in ImageFormat::iter() {
            let dds = dds_from_imagef32(
                &image,
                image_format,
                Quality::Fast,
                Mipmaps::GeneratedAutomatic,
            )
            .unwrap();
            assert_eq!(4, dds.get_width());
            assert_eq!(8, dds.get_height());
        }
    }

    #[test]
    fn imagef32_from_dds_formats() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 8 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 8,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            let dds = surface.to_dds().unwrap();

            let image = imagef32_from_dds(&dds, 0).unwrap();
            assert_eq!((4, 8), image.dimensions());
        }
    }
}
