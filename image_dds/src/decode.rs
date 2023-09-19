use crate::{
    bcn,
    error::SurfaceError,
    mip_dimension,
    rgba::{
        decode_rgba8_from_rgba8, rgba8_from_bgra8, rgba8_from_r8, rgba8_from_rgbaf16,
        rgba8_from_rgbaf32,
    },
    ImageFormat, Surface, SurfaceRgba8,
};
use bcn::{Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};

impl<T: AsRef<[u8]>> Surface<T> {
    /// Decode all layers and mipmaps from `surface` to RGBA8.
    pub fn decode_rgba8(&self) -> Result<SurfaceRgba8<Vec<u8>>, SurfaceError> {
        self.validate()?;

        let mut combined_surface_data = Vec::new();
        for layer in 0..self.layers {
            for mipmap in 0..self.mipmaps {
                let data = self
                    .get(layer, mipmap)
                    .ok_or(SurfaceError::MipmapDataOutOfBounds { layer, mipmap })?;

                // The mipmap index is already validated by get above.
                let width = mip_dimension(self.width, mipmap);
                let height = mip_dimension(self.height, mipmap);
                let depth = mip_dimension(self.depth, mipmap);

                // TODO: Avoid additional copies?
                let data = decode_data_rgba8(width, height, depth, self.image_format, data)?;
                combined_surface_data.extend_from_slice(&data);
            }
        }

        Ok(SurfaceRgba8 {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            data: combined_surface_data,
        })
    }
}

fn decode_data_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    image_format: ImageFormat,
    data: &[u8],
) -> Result<Vec<u8>, SurfaceError> {
    use ImageFormat as F;
    let data = match image_format {
        F::BC1Unorm | F::BC1Srgb => bcn::rgba8_from_bcn::<Bc1>(width, height, depth, data),
        F::BC2Unorm | F::BC2Srgb => bcn::rgba8_from_bcn::<Bc2>(width, height, depth, data),
        F::BC3Unorm | F::BC3Srgb => bcn::rgba8_from_bcn::<Bc3>(width, height, depth, data),
        F::BC4Unorm | F::BC4Snorm => bcn::rgba8_from_bcn::<Bc4>(width, height, depth, data),
        F::BC5Unorm | F::BC5Snorm => bcn::rgba8_from_bcn::<Bc5>(width, height, depth, data),
        F::BC6Ufloat | F::BC6Sfloat => bcn::rgba8_from_bcn::<Bc6>(width, height, depth, data),
        F::BC7Unorm | F::BC7Srgb => bcn::rgba8_from_bcn::<Bc7>(width, height, depth, data),
        F::R8Unorm => rgba8_from_r8(width, height, depth, data),
        F::R8G8B8A8Unorm => decode_rgba8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Srgb => decode_rgba8_from_rgba8(width, height, depth, data),
        F::R16G16B16A16Float => rgba8_from_rgbaf16(width, height, depth, data),
        F::R32G32B32A32Float => rgba8_from_rgbaf32(width, height, depth, data),
        F::B8G8R8A8Unorm => rgba8_from_bgra8(width, height, depth, data),
        F::B8G8R8A8Srgb => rgba8_from_bgra8(width, height, depth, data),
    }?;
    Ok(data)
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
