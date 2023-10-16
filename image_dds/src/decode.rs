use crate::{
    bcn::{self, rgba_from_bcn},
    error::SurfaceError,
    mip_dimension,
    rgba::{
        rgba8_from_bgra8, rgba8_from_r8, rgba8_from_rgba8, rgba8_from_rgbaf16, rgba8_from_rgbaf32,
        rgbaf32_from_rgbaf16, rgbaf32_from_rgbaf32,
    },
    ImageFormat, Surface, SurfaceRgba32Float, SurfaceRgba8,
};
use bcn::{Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};

impl<T: AsRef<[u8]>> Surface<T> {
    /// Decode all layers and mipmaps from `surface` to RGBA8.
    pub fn decode_rgba8(&self) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        self.validate()?;

        let data = decode_surface(self)?;

        Ok(SurfaceRgba8 {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            data,
        })
    }

    /// Decode all layers and mipmaps from `surface` to RGBAF32.
    ///
    /// Non floating point formats are normalized to the range `0.0` to `1.0`.
    pub fn decode_rgbaf32(&self) -> Result<SurfaceRgba32Float<Vec<f32>>, SurfaceError> {
        self.validate()?;

        let data = decode_surface(self)?;

        Ok(SurfaceRgba32Float {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            data,
        })
    }
}

fn decode_surface<T, P>(surface: &Surface<T>) -> Result<Vec<P>, SurfaceError>
where
    T: AsRef<[u8]>,
    P: Decode + Copy,
{
    let mut combined_surface_data = Vec::new();
    for layer in 0..surface.layers {
        for mipmap in 0..surface.mipmaps {
            let data = surface
                .get(layer, mipmap)
                .ok_or(SurfaceError::MipmapDataOutOfBounds { layer, mipmap })?;

            // The mipmap index is already validated by get above.
            let width = mip_dimension(surface.width, mipmap);
            let height = mip_dimension(surface.height, mipmap);
            let depth = mip_dimension(surface.depth, mipmap);

            // TODO: Avoid additional copies?
            let data = P::decode(width, height, depth, surface.image_format, data)?;
            combined_surface_data.extend_from_slice(&data);
        }
    }

    Ok(combined_surface_data)
}

trait Decode: Sized {
    fn decode(
        width: u32,
        height: u32,
        depth: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError>;
}

impl Decode for u8 {
    fn decode(
        width: u32,
        height: u32,
        depth: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError> {
        use ImageFormat as F;
        match image_format {
            F::BC1Unorm | F::BC1Srgb => rgba_from_bcn::<Bc1, u8>(width, height, depth, data),
            F::BC2Unorm | F::BC2Srgb => rgba_from_bcn::<Bc2, u8>(width, height, depth, data),
            F::BC3Unorm | F::BC3Srgb => rgba_from_bcn::<Bc3, u8>(width, height, depth, data),
            F::BC4Unorm | F::BC4Snorm => rgba_from_bcn::<Bc4, u8>(width, height, depth, data),
            F::BC5Unorm | F::BC5Snorm => rgba_from_bcn::<Bc5, u8>(width, height, depth, data),
            F::BC6Ufloat | F::BC6Sfloat => rgba_from_bcn::<Bc6, u8>(width, height, depth, data),
            F::BC7Unorm | F::BC7Srgb => rgba_from_bcn::<Bc7, u8>(width, height, depth, data),
            F::R8Unorm => rgba8_from_r8(width, height, depth, data),
            F::R8G8B8A8Unorm => rgba8_from_rgba8(width, height, depth, data),
            F::R8G8B8A8Srgb => rgba8_from_rgba8(width, height, depth, data),
            F::R16G16B16A16Float => rgba8_from_rgbaf16(width, height, depth, data),
            F::R32G32B32A32Float => rgba8_from_rgbaf32(width, height, depth, data),
            F::B8G8R8A8Unorm => rgba8_from_bgra8(width, height, depth, data),
            F::B8G8R8A8Srgb => rgba8_from_bgra8(width, height, depth, data),
        }
    }
}

impl Decode for f32 {
    fn decode(
        width: u32,
        height: u32,
        depth: u32,
        image_format: ImageFormat,
        data: &[u8],
    ) -> Result<Vec<Self>, SurfaceError> {
        use ImageFormat as F;
        match image_format {
            F::BC6Ufloat | F::BC6Sfloat => rgba_from_bcn::<Bc6, f32>(width, height, depth, data),
            F::R16G16B16A16Float => rgbaf32_from_rgbaf16(width, height, depth, data),
            F::R32G32B32A32Float => rgbaf32_from_rgbaf32(width, height, depth, data),
            _ => {
                // Use existing decoding for formats that don't store floating point data.
                let rgba8 = u8::decode(width, height, depth, image_format, data)?;
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
}
