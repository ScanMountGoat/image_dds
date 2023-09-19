use crate::bcn::{bcn_from_rgba, Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};
use crate::rgba::{
    bgra8_from_rgba8, r8_from_rgba8, rgba8_from_rgba8, rgbaf16_from_rgba8, rgbaf32_from_rgba8,
};
use crate::{
    downsample_rgba8, error::SurfaceError, max_mipmap_count, mip_dimension, round_up, ImageFormat,
    Mipmaps, Quality, Surface, SurfaceRgba8,
};

impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    // TODO: Add documentation showing how to use this.
    /// Encode an RGBA8 surface to the given `format`.
    ///
    /// The number of mipmaps generated depends on the `mipmaps` parameter.
    /// The `rgba8_data` only needs to contain enough data for the base mip level of `width` x `height` pixels.
    pub fn encode(
        &self,
        format: ImageFormat,
        quality: Quality,
        mipmaps: Mipmaps,
    ) -> Result<Surface<Vec<u8>>, SurfaceError> {
        let width = self.width;
        let height = self.height;
        let depth = self.depth;
        let layers = self.layers;

        self.validate()?;

        // TODO: Encode the correct number of array layers.
        let num_mipmaps = match mipmaps {
            Mipmaps::Disabled => 1,
            Mipmaps::FromSurface => self.mipmaps,
            Mipmaps::GeneratedExact(count) => count,
            Mipmaps::GeneratedAutomatic => max_mipmap_count(width.max(height).max(depth)),
        };

        let use_surface = mipmaps == Mipmaps::FromSurface;

        // TODO: Does this work if the base mip level is smaller than 4x4?
        let mut surface_data = Vec::new();

        for layer in 0..layers {
            encode_mipmaps_rgba8(
                &mut surface_data,
                self,
                format,
                quality,
                num_mipmaps,
                use_surface,
                layer,
            )?;
        }

        Ok(Surface {
            width,
            height,
            depth,
            layers,
            mipmaps: num_mipmaps,
            image_format: format,
            data: surface_data,
        })
    }
}

// TODO: Find a way to simplify this.
// TODO: Also support f32.
fn encode_mipmaps_rgba8<T: AsRef<[u8]>>(
    encoded_data: &mut Vec<u8>,
    surface: &SurfaceRgba8<T>,
    format: ImageFormat,
    quality: Quality,
    num_mipmaps: u32,
    use_surface: bool,
    layer: u32,
) -> Result<(), SurfaceError> {
    let block_dimensions = format.block_dimensions();

    // Track the previous image data and dimensions.
    // This enables generating mipmaps from a single base layer.
    let mut mip_data = get_mipmap_data(surface, layer, 0, format)?;

    let encoded = mip_data.encode(format, quality)?;
    encoded_data.extend_from_slice(&encoded);

    // TODO: Error if surface does not have the appropriate number of mipmaps?
    for mipmap in 1..num_mipmaps {
        mip_data = if use_surface {
            get_mipmap_data(surface, layer, mipmap, format)?
        } else {
            mip_data.downsample(
                surface.width,
                surface.height,
                surface.depth,
                block_dimensions,
                mipmap,
            )
        };

        let encoded = mip_data.encode(format, quality)?;
        encoded_data.extend_from_slice(&encoded);
    }
    Ok(())
}

struct MipData {
    width: usize,
    height: usize,
    depth: usize,
    data: Vec<u8>,
}

impl MipData {
    fn downsample(
        &self,
        base_width: u32,
        base_height: u32,
        base_depth: u32,
        block_dimensions: (u32, u32, u32),
        mipmap: u32,
    ) -> MipData {
        // Mip dimensions are the padded virtual size of the mipmap.
        // Padding the physical size of the previous mip produces incorrect results.
        let (width, height, depth) = physical_dimensions(
            mip_dimension(base_width, mipmap),
            mip_dimension(base_height, mipmap),
            mip_dimension(base_depth, mipmap),
            block_dimensions,
        );

        // Assume the data is already padded.
        let data = downsample_rgba8(
            width,
            height,
            depth,
            self.width,
            self.height,
            self.depth,
            &self.data,
        );

        MipData {
            width,
            height,
            depth,
            data,
        }
    }

    fn encode(&self, format: ImageFormat, quality: Quality) -> Result<Vec<u8>, SurfaceError> {
        encode_rgba8(
            self.width as u32,
            self.height as u32,
            self.depth as u32,
            &self.data,
            format,
            quality,
        )
    }
}

fn get_mipmap_data<T: AsRef<[u8]>>(
    surface: &SurfaceRgba8<T>,
    layer: u32,
    mipmap: u32,
    format: ImageFormat,
) -> Result<MipData, SurfaceError> {
    let block_dimensions = format.block_dimensions();

    let mip_width = mip_dimension(surface.width, mipmap);
    let mip_height = mip_dimension(surface.height, mipmap);
    let mip_depth = mip_dimension(surface.depth, mipmap);

    let data = surface.get(layer, mipmap).unwrap();

    let (width, height, depth) =
        physical_dimensions(mip_width, mip_height, mip_depth, block_dimensions);

    // TODO: Just take the block dimensions instead?
    let data = pad_mipmap_rgba8(
        mip_width as usize,
        mip_height as usize,
        mip_depth as usize,
        width,
        height,
        depth,
        data,
    );

    Ok(MipData {
        width,
        height,
        depth,
        data,
    })
}

fn physical_dimensions(
    width: u32,
    height: u32,
    depth: u32,
    block_dimensions: (u32, u32, u32),
) -> (usize, usize, usize) {
    // The physical size must have integral dimensions in blocks.
    // Applications or the GPU will use the smaller virtual size and ignore padding.
    // For example, a 1x1 BCN block still requires 4x4 pixels of data.
    // https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression
    let (block_width, block_height, block_depth) = block_dimensions;
    (
        round_up(width as usize, block_width as usize),
        round_up(height as usize, block_height as usize),
        round_up(depth as usize, block_depth as usize),
    )
}

fn pad_mipmap_rgba8(
    width: usize,
    height: usize,
    depth: usize,
    new_width: usize,
    new_height: usize,
    new_depth: usize,
    data: &[u8],
) -> Vec<u8> {
    let new_size = new_width * new_height * new_depth * 4;

    if data.len() < new_size {
        // Zero pad the data to the appropriate size.
        let mut padded_data = vec![0u8; new_size];
        // Copy the original data row by row.
        for z in 0..depth {
            for y in 0..height {
                // Assume padded dimensions are larger than the dimensions.
                let in_base = ((z * width * height) + y * width) * 4;
                let out_base = ((z * new_width * new_height) + y * new_width) * 4;
                padded_data[out_base..out_base + width * 4]
                    .copy_from_slice(&data[in_base..in_base + width * 4]);
            }
        }

        padded_data
    } else {
        data.to_vec()
    }
}

fn encode_rgba8(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
    format: ImageFormat,
    quality: Quality,
) -> Result<Vec<u8>, SurfaceError> {
    // Unorm and srgb only affect how the data is read.
    // Use the same conversion code for both.
    use ImageFormat as F;
    match format {
        F::BC1Unorm | F::BC1Srgb => bcn_from_rgba::<Bc1, u8>(width, height, depth, data, quality),
        F::BC2Unorm | F::BC2Srgb => bcn_from_rgba::<Bc2, u8>(width, height, depth, data, quality),
        F::BC3Unorm | F::BC3Srgb => bcn_from_rgba::<Bc3, u8>(width, height, depth, data, quality),
        F::BC4Unorm | F::BC4Snorm => bcn_from_rgba::<Bc4, u8>(width, height, depth, data, quality),
        F::BC5Unorm | F::BC5Snorm => bcn_from_rgba::<Bc5, u8>(width, height, depth, data, quality),
        F::BC6Ufloat | F::BC6Sfloat => {
            bcn_from_rgba::<Bc6, u8>(width, height, depth, data, quality)
        }
        F::BC7Unorm | F::BC7Srgb => bcn_from_rgba::<Bc7, u8>(width, height, depth, data, quality),
        F::R8Unorm => r8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Unorm => rgba8_from_rgba8(width, height, depth, data),
        F::R8G8B8A8Srgb => rgba8_from_rgba8(width, height, depth, data),
        F::R16G16B16A16Float => rgbaf16_from_rgba8(width, height, depth, data),
        F::R32G32B32A32Float => rgbaf32_from_rgba8(width, height, depth, data),
        F::B8G8R8A8Unorm => bgra8_from_rgba8(width, height, depth, data),
        F::B8G8R8A8Srgb => bgra8_from_rgba8(width, height, depth, data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_surface_integral_dimensions() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = SurfaceRgba8 {
            width: 12,
            height: 12,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: &[0u8; 12 * 12 * 4],
        }
        .encode(
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(12, surface.width);
        assert_eq!(12, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(4, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!((9 + 4 + 1 + 1) * 16, surface.data.len());
    }

    #[test]
    fn encode_surface_cube_mipmaps() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = SurfaceRgba8 {
            width: 4,
            height: 4,
            depth: 1,
            layers: 6,
            mipmaps: 3,
            data: &[0u8; (4 * 4 + 2 * 2 + 1 * 1) * 6 * 4],
        }
        .encode(
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(6, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!(3 * 16 * 6, surface.data.len());
    }

    #[test]
    fn encode_surface_disabled_mipmaps() {
        let surface = SurfaceRgba8 {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 3,
            data: &[0u8; 64 + 16 + 4],
        }
        .encode(ImageFormat::BC7Srgb, Quality::Fast, Mipmaps::Disabled)
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(1, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        assert_eq!(16, surface.data.len());
    }

    #[test]
    fn encode_surface_mipmaps_from_surface() {
        let surface = SurfaceRgba8 {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 2,
            data: &[0u8; 64 + 16],
        }
        .encode(ImageFormat::BC7Srgb, Quality::Fast, Mipmaps::FromSurface)
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(2, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        assert_eq!(16 * 2, surface.data.len());
    }

    #[test]
    fn encode_surface_non_integral_dimensions() {
        // This should succeed with appropriate padding.
        let surface = SurfaceRgba8 {
            width: 3,
            height: 5,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: &[0u8; 256],
        }
        .encode(
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(3, surface.width);
        assert_eq!(5, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7Srgb, surface.image_format);
        // Each mipmap must have an integral size in blocks.
        assert_eq!((2 + 2) * 16, surface.data.len());
    }

    #[test]
    fn encode_surface_zero_size() {
        let result = SurfaceRgba8 {
            width: 0,
            height: 0,
            depth: 0,
            layers: 1,
            mipmaps: 1,
            data: &[0u8; 0],
        }
        .encode(
            ImageFormat::BC7Srgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        );
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
    fn pad_1x1_to_2x2() {
        assert_eq!(
            vec![1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            pad_mipmap_rgba8(1, 1, 1, 2, 2, 1, &[1, 2, 3, 4])
        );
    }

    #[test]
    fn pad_2x2_to_3x3() {
        assert_eq!(
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            pad_mipmap_rgba8(
                2,
                2,
                1,
                3,
                3,
                1,
                &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
            )
        );
    }

    #[test]
    fn physical_dimensions_padding() {
        assert_eq!((4, 5, 6), physical_dimensions(2, 3, 1, (4, 5, 6)));
    }

    #[test]
    fn physical_dimensions_mipmaps() {
        assert_eq!((8, 8, 1), physical_dimensions(8, 8, 1, (4, 4, 1)));
        assert_eq!((4, 4, 1), physical_dimensions(4, 4, 1, (4, 4, 1)));
        assert_eq!((4, 4, 1), physical_dimensions(2, 2, 1, (4, 4, 1)));
        assert_eq!((4, 4, 1), physical_dimensions(1, 1, 1, (4, 4, 1)));
    }
}
