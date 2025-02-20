use std::ops::Range;

use crate::{
    bcn::{self, decode_bcn},
    error::SurfaceError,
    mip_dimension,
    rgba::{
        decode_rgba, Bgr5A1, Bgr8, Bgra4, Bgra8, R16Snorm, R8Snorm, Rf16, Rf32, Rg16, Rg16Snorm,
        Rg8, Rg8Snorm, Rgba16, Rgba16Snorm, Rgba8, Rgba8Snorm, Rgbaf16, Rgbaf32, Rgbf32, Rgf16,
        Rgf32, R16, R8,
    },
    ImageFormat, Surface, SurfaceRgba32Float, SurfaceRgba8,
};
use bcn::{Bc1, Bc2, Bc3, Bc4, Bc4S, Bc5, Bc5S, Bc6, Bc7};

impl<T: AsRef<[u8]>> Surface<T> {
    /// Decode all layers and mipmaps from `surface` to RGBA8.
    pub fn decode_rgba8(&self) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        self.decode_layers_mipmaps_rgba8(0..self.layers, 0..self.mipmaps)
    }

    /// Decode a specific range of layers and mipmaps from `surface` to RGBA8.
    pub fn decode_layers_mipmaps_rgba8(
        &self,
        layers: Range<u32>,
        mipmaps: Range<u32>,
    ) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        self.validate()?;

        let data = decode_surface(self, layers.clone(), mipmaps.clone())?;

        Ok(SurfaceRgba8 {
            width: mip_dimension(self.width, mipmaps.start),
            height: mip_dimension(self.height, mipmaps.start),
            depth: mip_dimension(self.depth, mipmaps.start),
            layers: (layers.end - layers.start).max(1),
            mipmaps: (mipmaps.end - mipmaps.start).max(1),
            data,
        })
    }

    /// Decode all layers and mipmaps from `surface` to RGBAF32.
    ///
    /// Non floating point formats are normalized to the range `0.0` to `1.0`.
    pub fn decode_rgbaf32(&self) -> Result<SurfaceRgba32Float<Vec<f32>>, SurfaceError> {
        self.decode_layers_mipmaps_rgbaf32(0..self.layers, 0..self.mipmaps)
    }

    /// Decode a specific range of layers and mipmaps from `surface` to RGBAF32.
    ///
    /// Non floating point formats are normalized to the range `0.0` to `1.0`.
    pub fn decode_layers_mipmaps_rgbaf32(
        &self,
        layers: Range<u32>,
        mipmaps: Range<u32>,
    ) -> Result<SurfaceRgba32Float<Vec<f32>>, SurfaceError> {
        self.validate()?;

        let data = decode_surface(self, layers.clone(), mipmaps.clone())?;

        Ok(SurfaceRgba32Float {
            width: mip_dimension(self.width, mipmaps.start),
            height: mip_dimension(self.height, mipmaps.start),
            depth: mip_dimension(self.depth, mipmaps.start),
            layers: (layers.end - layers.start).max(1),
            mipmaps: (mipmaps.end - mipmaps.start).max(1),
            data,
        })
    }
}

fn decode_surface<T, P>(
    surface: &Surface<T>,
    layers: Range<u32>,
    mipmaps: Range<u32>,
) -> Result<Vec<P>, SurfaceError>
where
    T: AsRef<[u8]>,
    P: Decode + Copy,
{
    let mut combined_surface_data = Vec::new();
    for layer in layers {
        for level in 0..surface.depth {
            for mipmap in mipmaps.clone() {
                let data = surface
                    .get(layer, level, mipmap)
                    .ok_or(SurfaceError::MipmapDataOutOfBounds { layer, mipmap })?;

                // The mipmap index is already validated by get above.
                let width = mip_dimension(surface.width, mipmap);
                let height = mip_dimension(surface.height, mipmap);

                // TODO: Avoid additional copies?
                let data = P::decode(width, height, surface.image_format, data)?;

                combined_surface_data.extend_from_slice(&data);
            }
        }
    }

    Ok(combined_surface_data)
}

// Decoding only works on 2D surfaces.
trait Decode: Sized {
    fn decode(
        width: u32,
        height: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError>;
}

impl Decode for u8 {
    fn decode(
        width: u32,
        height: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError> {
        use ImageFormat as F;
        match image_format {
            F::BC1RgbaUnorm | F::BC1RgbaUnormSrgb => decode_bcn::<Bc1, u8>(width, height, data),
            F::BC2RgbaUnorm | F::BC2RgbaUnormSrgb => decode_bcn::<Bc2, u8>(width, height, data),
            F::BC3RgbaUnorm | F::BC3RgbaUnormSrgb => decode_bcn::<Bc3, u8>(width, height, data),
            F::BC4RUnorm => decode_bcn::<Bc4, u8>(width, height, data),
            F::BC4RSnorm => decode_bcn::<Bc4S, u8>(width, height, data),
            F::BC5RgUnorm => decode_bcn::<Bc5, u8>(width, height, data),
            F::BC5RgSnorm => decode_bcn::<Bc5S, u8>(width, height, data),
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => decode_bcn::<Bc6, u8>(width, height, data),
            F::BC7RgbaUnorm | F::BC7RgbaUnormSrgb => decode_bcn::<Bc7, u8>(width, height, data),
            F::R8Unorm => decode_rgba::<R8, u8>(width, height, data),
            F::R8Snorm => decode_rgba::<R8Snorm, u8>(width, height, data),
            F::Rg8Unorm => decode_rgba::<Rg8, u8>(width, height, data),
            F::Rg8Snorm => decode_rgba::<Rg8Snorm, u8>(width, height, data),
            F::Rgba8Unorm | F::Rgba8UnormSrgb => decode_rgba::<Rgba8, u8>(width, height, data),
            F::Rgba16Float => decode_rgba::<Rgbaf16, u8>(width, height, data),
            F::Rgba32Float => decode_rgba::<Rgbaf32, u8>(width, height, data),
            F::Bgra8Unorm | F::Bgra8UnormSrgb => decode_rgba::<Bgra8, u8>(width, height, data),
            F::Rgba8Snorm => decode_rgba::<Rgba8Snorm, u8>(width, height, data),
            F::Bgra4Unorm => decode_rgba::<Bgra4, u8>(width, height, data),
            F::Bgr8Unorm => decode_rgba::<Bgr8, u8>(width, height, data),
            F::R16Unorm => decode_rgba::<R16, u8>(width, height, data),
            F::R16Snorm => decode_rgba::<R16Snorm, u8>(width, height, data),
            F::Rg16Unorm => decode_rgba::<Rg16, u8>(width, height, data),
            F::Rg16Snorm => decode_rgba::<Rg16Snorm, u8>(width, height, data),
            F::Rgba16Unorm => decode_rgba::<Rgba16, u8>(width, height, data),
            F::Rgba16Snorm => decode_rgba::<Rgba16Snorm, u8>(width, height, data),
            F::Rg16Float => decode_rgba::<Rgf16, u8>(width, height, data),
            F::Rg32Float => decode_rgba::<Rgf32, u8>(width, height, data),
            F::R16Float => decode_rgba::<Rf16, u8>(width, height, data),
            F::R32Float => decode_rgba::<Rf32, u8>(width, height, data),
            F::Rgb32Float => decode_rgba::<Rgbf32, u8>(width, height, data),
            F::Bgr5A1Unorm => decode_rgba::<Bgr5A1, u8>(width, height, data),
        }
    }
}

impl Decode for f32 {
    fn decode(
        width: u32,
        height: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError> {
        use ImageFormat as F;
        match image_format {
            F::R8Snorm => decode_rgba::<R8Snorm, f32>(width, height, data),
            F::Rg8Snorm => decode_rgba::<Rg8Snorm, f32>(width, height, data),
            F::Rgba8Snorm => decode_rgba::<Rgba8Snorm, f32>(width, height, data),
            F::BC4RSnorm => decode_bcn::<Bc4S, f32>(width, height, data),
            F::BC5RgSnorm => decode_bcn::<Bc5S, f32>(width, height, data),
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => decode_bcn::<Bc6, f32>(width, height, data),
            F::R16Float => decode_rgba::<Rf16, f32>(width, height, data),
            F::Rg16Float => decode_rgba::<Rgf16, f32>(width, height, data),
            F::Rgba16Float => decode_rgba::<Rgbaf16, f32>(width, height, data),
            F::R32Float => decode_rgba::<Rf32, f32>(width, height, data),
            F::Rg32Float => decode_rgba::<Rgf32, f32>(width, height, data),
            F::Rgb32Float => decode_rgba::<Rgbf32, f32>(width, height, data),
            F::Rgba32Float => decode_rgba::<Rgbaf32, f32>(width, height, data),
            F::R16Unorm => decode_rgba::<R16, f32>(width, height, data),
            F::Rg16Unorm => decode_rgba::<Rg16, f32>(width, height, data),
            F::Rgba16Unorm => decode_rgba::<Rgba16, f32>(width, height, data),
            F::R16Snorm => decode_rgba::<R16Snorm, f32>(width, height, data),
            F::Rg16Snorm => decode_rgba::<Rg16Snorm, f32>(width, height, data),
            F::Rgba16Snorm => decode_rgba::<Rgba16Snorm, f32>(width, height, data),
            _ => {
                // Use existing decoding for formats that don't store floating point data.
                let rgba8 = u8::decode(width, height, image_format, data)?;
                Ok(rgba8.into_iter().map(|u| u as f32 / 255.0).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn decode_surface_zero_size() {
        let result = Surface {
            width: 0,
            height: 0,
            depth: 0,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 0],
        }
        .decode_rgba8();

        assert!(matches!(
            result,
            Err(SurfaceError::ZeroSizedSurface {
                width: 0,
                height: 0,
                depth: 0,
            })
        ));
    }

    #[test]
    fn decode_surface_dimensions_overflow() {
        let result = Surface {
            width: u32::MAX,
            height: u32::MAX,
            depth: u32::MAX,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 0],
        }
        .decode_rgba8();

        assert!(matches!(
            result,
            Err(SurfaceError::PixelCountWouldOverflow {
                width: u32::MAX,
                height: u32::MAX,
                depth: u32::MAX,
            })
        ));
    }

    #[test]
    fn decode_surface_too_many_mipmaps() {
        let result = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 10,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 4 * 4 * 4],
        }
        .decode_rgba8();

        assert!(matches!(
            result,
            Err(SurfaceError::UnexpectedMipmapCount {
                mipmaps: 10,
                max_mipmaps: 3
            })
        ));
    }

    #[test]
    fn decode_layers_mipmaps_rgba8_single_mipmap() {
        let rgba8 = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 3,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 512],
        }
        .decode_layers_mipmaps_rgba8(0..1, 1..2)
        .unwrap();

        assert_eq!(
            SurfaceRgba8 {
                width: 2,
                height: 2,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 2 * 2 * 4]
            },
            rgba8
        );
    }

    #[test]
    fn decode_layers_mipmaps_rgba8_no_mipmaps() {
        // TODO: How to handle this?
        let rgba8 = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 4 * 4 * 4],
        }
        .decode_layers_mipmaps_rgba8(0..1, 0..0)
        .unwrap();

        assert_eq!(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: Vec::new()
            },
            rgba8
        );
    }

    #[test]
    fn decode_layers_mipmaps_rgbaf32_single_mipmap() {
        let rgbaf32 = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 3,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 512],
        }
        .decode_layers_mipmaps_rgbaf32(0..1, 1..2)
        .unwrap();

        assert_eq!(
            SurfaceRgba32Float {
                width: 2,
                height: 2,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 2 * 2 * 4]
            },
            rgbaf32
        );
    }

    #[test]
    fn decode_layers_mipmaps_rgbaf32_no_mipmaps() {
        // TODO: How to handle this?
        let rgbaf32 = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::Rgba8UnormSrgb,
            data: &[0u8; 4 * 4 * 4],
        }
        .decode_layers_mipmaps_rgbaf32(0..1, 0..0)
        .unwrap();

        assert_eq!(
            SurfaceRgba32Float {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: Vec::new()
            },
            rgbaf32
        );
    }

    #[test]
    fn decode_all_u8() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 4 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            surface.decode_rgba8().unwrap();
        }
    }

    #[test]
    fn decode_all_f32() {
        for image_format in ImageFormat::iter() {
            let data = vec![0u8; 4 * 4 * image_format.block_size_in_bytes()];
            let surface = Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format,
                data: data.as_slice(),
            };
            surface.decode_rgbaf32().unwrap();
        }
    }
}
