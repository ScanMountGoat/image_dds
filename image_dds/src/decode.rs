use std::ops::Range;

use crate::{
    bcn::{self, rgba_from_bcn},
    error::SurfaceError,
    mip_dimension,
    rgba::{
        rgba8_from_bgra4, rgba8_from_bgra8, rgba8_from_r8, rgba8_from_rgba8, rgba8_from_rgbaf16,
        rgba8_from_rgbaf32, rgbaf32_from_rgbaf16, rgbaf32_from_rgbaf32,
    },
    ImageFormat, Surface, SurfaceRgba32Float, SurfaceRgba8,
};
use bcn::{Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};

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
            F::BC1Unorm | F::BC1Srgb => rgba_from_bcn::<Bc1, u8>(width, height, data),
            F::BC2Unorm | F::BC2Srgb => rgba_from_bcn::<Bc2, u8>(width, height, data),
            F::BC3Unorm | F::BC3Srgb => rgba_from_bcn::<Bc3, u8>(width, height, data),
            F::BC4Unorm | F::BC4Snorm => rgba_from_bcn::<Bc4, u8>(width, height, data),
            F::BC5Unorm | F::BC5Snorm => rgba_from_bcn::<Bc5, u8>(width, height, data),
            F::BC6Ufloat | F::BC6Sfloat => rgba_from_bcn::<Bc6, u8>(width, height, data),
            F::BC7Unorm | F::BC7Srgb => rgba_from_bcn::<Bc7, u8>(width, height, data),
            F::R8Unorm => rgba8_from_r8(width, height, data),
            F::R8G8B8A8Unorm => rgba8_from_rgba8(width, height, data),
            F::R8G8B8A8Srgb => rgba8_from_rgba8(width, height, data),
            F::R16G16B16A16Float => rgba8_from_rgbaf16(width, height, data),
            F::R32G32B32A32Float => rgba8_from_rgbaf32(width, height, data),
            F::B8G8R8A8Unorm => rgba8_from_bgra8(width, height, data),
            F::B8G8R8A8Srgb => rgba8_from_bgra8(width, height, data),
            F::B4G4R4A4Unorm => rgba8_from_bgra4(width, height, data),
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
            F::BC6Ufloat | F::BC6Sfloat => rgba_from_bcn::<Bc6, f32>(width, height, data),
            F::R16G16B16A16Float => rgbaf32_from_rgbaf16(width, height, data),
            F::R32G32B32A32Float => rgbaf32_from_rgbaf32(width, height, data),
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

    #[test]
    fn decode_surface_zero_size() {
        let result = Surface {
            width: 0,
            height: 0,
            depth: 0,
            layers: 1,
            mipmaps: 1,
            image_format: ImageFormat::R8G8B8A8Srgb,
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
            image_format: ImageFormat::R8G8B8A8Srgb,
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
            image_format: ImageFormat::R8G8B8A8Srgb,
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

    // TODO: decode_layers_mipmaps_rgba8
    // TODO: decode_layers_mipmaps_rgbaf32
    #[test]
    fn decode_layers_mipmaps_rgba8_single_mipmap() {
        let rgba8 = Surface {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 3,
            image_format: ImageFormat::R8G8B8A8Srgb,
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
            image_format: ImageFormat::R8G8B8A8Srgb,
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
            image_format: ImageFormat::R8G8B8A8Srgb,
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
            image_format: ImageFormat::R8G8B8A8Srgb,
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
}
