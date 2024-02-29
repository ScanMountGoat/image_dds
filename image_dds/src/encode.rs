use std::borrow::Cow;

use crate::bcn::{bcn_from_rgba, Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};
use crate::rgba::{
    bgra4_from_rgba8, bgra8_from_rgba8, r8_from_rgba8, rgba8_from_rgba8, rgbaf16_from_rgba8,
    rgbaf16_from_rgbaf32, rgbaf32_from_rgba8, rgbaf32_from_rgbaf32,
};
use crate::{
    downsample_rgba, error::SurfaceError, max_mipmap_count, mip_dimension, round_up, ImageFormat,
    Mipmaps, Quality, Surface, SurfaceRgba8,
};
use crate::{Pixel, SurfaceRgba32Float};

impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    /// Encode an RGBA8 surface to the given `format`.
    ///
    /// The number of mipmaps generated depends on the `mipmaps` parameter.
    pub fn encode(
        &self,
        format: ImageFormat,
        quality: Quality,
        mipmaps: Mipmaps,
    ) -> Result<Surface<Vec<u8>>, SurfaceError> {
        self.validate()?;
        encode_surface(self, format, quality, mipmaps)
    }
}

// TODO: Tests for this?
impl<T: AsRef<[f32]>> SurfaceRgba32Float<T> {
    /// Encode an RGBAF32 surface to the given `format`.
    ///
    /// The number of mipmaps generated depends on the `mipmaps` parameter.
    pub fn encode(
        &self,
        format: ImageFormat,
        quality: Quality,
        mipmaps: Mipmaps,
    ) -> Result<Surface<Vec<u8>>, SurfaceError> {
        self.validate()?;
        encode_surface(self, format, quality, mipmaps)
    }
}

fn encode_surface<S, P>(
    surface: &S,
    format: ImageFormat,
    quality: Quality,
    mipmaps: Mipmaps,
) -> Result<Surface<Vec<u8>>, SurfaceError>
where
    S: GetMipmap<P>,
    P: Default + Copy + Encode + Pixel,
{
    // TODO: Encode the correct number of array layers.
    let num_mipmaps = match mipmaps {
        Mipmaps::Disabled => 1,
        Mipmaps::FromSurface => surface.mipmaps(),
        Mipmaps::GeneratedExact(count) => count,
        Mipmaps::GeneratedAutomatic => {
            max_mipmap_count(surface.width().max(surface.height()).max(surface.depth()))
        }
    };

    let use_surface = mipmaps == Mipmaps::FromSurface;

    // TODO: Does this work if the base mip level is smaller than 4x4?
    let mut surface_data = Vec::new();

    for layer in 0..surface.layers() {
        // Encode 2D or 3D data for this layer.
        encode_mipmaps_rgba(
            &mut surface_data,
            surface,
            format,
            quality,
            num_mipmaps,
            use_surface,
            layer,
        )?;
    }

    Ok(Surface {
        width: surface.width(),
        height: surface.height(),
        depth: surface.depth(),
        layers: surface.layers(),
        mipmaps: num_mipmaps,
        image_format: format,
        data: surface_data,
    })
}

// TODO: Find a way to simplify this.
fn encode_mipmaps_rgba<S, P>(
    surface_data: &mut Vec<u8>,
    surface: &S,
    format: ImageFormat,
    quality: Quality,
    num_mipmaps: u32,
    use_surface: bool,
    layer: u32,
) -> Result<(), SurfaceError>
where
    S: GetMipmap<P>,
    P: Default + Copy + Encode + Pixel,
{
    let block_dimensions = format.block_dimensions();

    for level in 0..surface.depth() {
        // Track the previous image data and dimensions.
        // This enables generating mipmaps from a single base layer.
        let mut mip_data = get_mipmap_data(surface, layer, level, 0, block_dimensions)?;

        let encoded = mip_data.encode(format, quality)?;
        surface_data.extend_from_slice(&encoded);

        for mipmap in 1..num_mipmaps {
            mip_data = if use_surface {
                // TODO: Error if surface does not have the appropriate number of mipmaps?
                get_mipmap_data(surface, layer, level, mipmap, block_dimensions)?
            } else {
                mip_data.downsample(surface.width(), surface.height(), block_dimensions, mipmap)
            };

            let encoded = mip_data.encode(format, quality)?;
            surface_data.extend_from_slice(&encoded);
        }
    }

    Ok(())
}

struct MipData<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T: Pixel> MipData<T> {
    fn downsample(
        &self,
        base_width: u32,
        base_height: u32,
        block_dimensions: (u32, u32, u32),
        mipmap: u32,
    ) -> MipData<T> {
        // Mip dimensions are the padded virtual size of the mipmap.
        // Padding the physical size of the previous mip produces incorrect results.
        let (width, height, depth) = physical_dimensions(
            mip_dimension(base_width, mipmap),
            mip_dimension(base_height, mipmap),
            1,
            block_dimensions,
        );

        // Assume the data is already padded.
        let data = downsample_rgba(width, height, depth, self.width, self.height, 1, &self.data);

        MipData {
            width,
            height,
            data,
        }
    }
}

impl<T> MipData<T>
where
    T: Encode,
{
    fn encode(&self, format: ImageFormat, quality: Quality) -> Result<Vec<u8>, SurfaceError> {
        T::encode(
            self.width as u32,
            self.height as u32,
            &self.data,
            format,
            quality,
        )
    }
}

trait GetMipmap<P> {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
    fn layers(&self) -> u32;
    fn mipmaps(&self) -> u32;
    fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[P]>;
}

impl<T> GetMipmap<u8> for SurfaceRgba8<T>
where
    T: AsRef<[u8]>,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn depth(&self) -> u32 {
        self.depth
    }

    fn layers(&self) -> u32 {
        self.layers
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[u8]> {
        self.get(layer, depth_level, mipmap)
    }
}

impl<T> GetMipmap<f32> for SurfaceRgba32Float<T>
where
    T: AsRef<[f32]>,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn depth(&self) -> u32 {
        self.depth
    }

    fn layers(&self) -> u32 {
        self.layers
    }

    fn mipmaps(&self) -> u32 {
        self.mipmaps
    }

    fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[f32]> {
        self.get(layer, depth_level, mipmap)
    }
}

fn get_mipmap_data<S, P>(
    surface: &S,
    layer: u32,
    depth_level: u32,
    mipmap: u32,
    block_dimensions: (u32, u32, u32),
) -> Result<MipData<P>, SurfaceError>
where
    S: GetMipmap<P>,
    P: Default + Copy,
{
    let mip_width = mip_dimension(surface.width(), mipmap);
    let mip_height = mip_dimension(surface.height(), mipmap);

    let data = surface.get(layer, depth_level, mipmap).unwrap();

    let (width, height, _) = physical_dimensions(mip_width, mip_height, 1, block_dimensions);

    let data = pad_mipmap_rgba(
        mip_width as usize,
        mip_height as usize,
        1,
        width,
        height,
        1,
        data,
    )
    .to_vec();

    Ok(MipData {
        width,
        height,
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

fn pad_mipmap_rgba<T>(
    width: usize,
    height: usize,
    depth: usize,
    new_width: usize,
    new_height: usize,
    new_depth: usize,
    data: &[T],
) -> Cow<[T]>
where
    T: Default + Copy,
{
    let channels = 4;
    let new_size = new_width * new_height * new_depth * channels;

    if data.len() < new_size {
        // Zero pad the data to the appropriate size.
        let mut padded_data = vec![T::default(); new_size];
        // Copy the original data row by row.
        for z in 0..depth {
            for y in 0..height {
                // Assume padded dimensions are larger than the dimensions.
                let in_base = ((z * width * height) + y * width) * channels;
                let out_base = ((z * new_width * new_height) + y * new_width) * channels;
                padded_data[out_base..out_base + width * channels]
                    .copy_from_slice(&data[in_base..in_base + width * channels]);
            }
        }

        Cow::Owned(padded_data)
    } else {
        Cow::Borrowed(data)
    }
}

// Encoding only works on 2D surfaces.
trait Encode: Sized {
    fn encode(
        width: u32,
        height: u32,
        data: &[Self],
        format: ImageFormat,
        quality: Quality,
    ) -> Result<Vec<u8>, SurfaceError>;
}

impl Encode for u8 {
    fn encode(
        width: u32,
        height: u32,
        data: &[Self],
        format: ImageFormat,
        quality: Quality,
    ) -> Result<Vec<u8>, SurfaceError> {
        // Unorm and srgb only affect how the data is read.
        // Use the same conversion code for both.
        use ImageFormat as F;
        match format {
            F::BC1RgbaUnorm | F::BC1RgbaUnormSrgb => {
                bcn_from_rgba::<Bc1, u8>(width, height, data, quality)
            }
            F::BC2RgbaUnorm | F::BC2RgbaUnormSrgb => {
                bcn_from_rgba::<Bc2, u8>(width, height, data, quality)
            }
            F::BC3RgbaUnorm | F::BC3RgbaUnormSrgb => {
                bcn_from_rgba::<Bc3, u8>(width, height, data, quality)
            }
            F::BC4RUnorm | F::BC4RSnorm => bcn_from_rgba::<Bc4, u8>(width, height, data, quality),
            F::BC5RgUnorm | F::BC5RgSnorm => bcn_from_rgba::<Bc5, u8>(width, height, data, quality),
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => {
                bcn_from_rgba::<Bc6, u8>(width, height, data, quality)
            }
            F::BC7RgbaUnorm | F::BC7RgbaUnormSrgb => {
                bcn_from_rgba::<Bc7, u8>(width, height, data, quality)
            }
            F::R8Unorm => r8_from_rgba8(width, height, data),
            F::Rgba8Unorm => rgba8_from_rgba8(width, height, data),
            F::Rgba8UnormSrgb => rgba8_from_rgba8(width, height, data),
            F::Rgba16Float => rgbaf16_from_rgba8(width, height, data),
            F::Rgba32Float => rgbaf32_from_rgba8(width, height, data),
            F::Bgra8Unorm => bgra8_from_rgba8(width, height, data),
            F::Bgra8UnormSrgb => bgra8_from_rgba8(width, height, data),
            F::Bgra4Unorm => bgra4_from_rgba8(width, height, data),
        }
    }
}

impl Encode for f32 {
    fn encode(
        width: u32,
        height: u32,
        data: &[Self],
        format: ImageFormat,
        quality: Quality,
    ) -> Result<Vec<u8>, SurfaceError> {
        // Unorm and srgb only affect how the data is read.
        // Use the same conversion code for both.
        use ImageFormat as F;
        match format {
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => {
                bcn_from_rgba::<Bc6, f32>(width, height, data, quality)
            }
            F::Rgba16Float => {
                // TODO: Create conversion functions that don't require a cast?
                rgbaf16_from_rgbaf32(width, height, bytemuck::cast_slice(data))
                    .map(bytemuck::cast_vec)
            }
            F::Rgba32Float => rgbaf32_from_rgbaf32(width, height, bytemuck::cast_slice(data))
                .map(bytemuck::cast_vec),
            _ => {
                let rgba8: Vec<_> = data.iter().map(|f| (f * 255.0) as u8).collect();
                u8::encode(width, height, &rgba8, format, quality)
            }
        }
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
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(12, surface.width);
        assert_eq!(12, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(4, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
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
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(6, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
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
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::Disabled,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(1, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
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
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::FromSurface,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(2, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
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
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(3, surface.width);
        assert_eq!(5, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
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
            ImageFormat::BC7RgbaUnormSrgb,
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
    fn encode_surface_float32_integral_dimensions() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = SurfaceRgba32Float {
            width: 12,
            height: 12,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: &[0.0; 12 * 12 * 4],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(12, surface.width);
        assert_eq!(12, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(4, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!((9 + 4 + 1 + 1) * 16, surface.data.len());
    }

    #[test]
    fn encode_surface_float32_cube_mipmaps() {
        // It's ok for mipmaps to not be divisible by the block width.
        let surface = SurfaceRgba32Float {
            width: 4,
            height: 4,
            depth: 1,
            layers: 6,
            mipmaps: 3,
            data: &[0.0; (4 * 4 + 2 * 2 + 1 * 1) * 6 * 4],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(6, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
        // Each mipmap must be at least 1 block in size.
        assert_eq!(3 * 16 * 6, surface.data.len());
    }

    #[test]
    fn encode_surface_float32_disabled_mipmaps() {
        let surface = SurfaceRgba32Float {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 3,
            data: &[0.0; 64 + 16 + 4],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::Disabled,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(1, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
        assert_eq!(16, surface.data.len());
    }

    #[test]
    fn encode_surface_float32_mipmaps_from_surface() {
        let surface = SurfaceRgba32Float {
            width: 4,
            height: 4,
            depth: 1,
            layers: 1,
            mipmaps: 2,
            data: &[0.0; 64 + 16],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::FromSurface,
        )
        .unwrap();

        assert_eq!(4, surface.width);
        assert_eq!(4, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(2, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
        assert_eq!(16 * 2, surface.data.len());
    }

    #[test]
    fn encode_surface_float32_non_integral_dimensions() {
        // This should succeed with appropriate padding.
        let surface = SurfaceRgba32Float {
            width: 3,
            height: 5,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: &[0.0; 256],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(3, surface.width);
        assert_eq!(5, surface.height);
        assert_eq!(1, surface.depth);
        assert_eq!(1, surface.layers);
        assert_eq!(3, surface.mipmaps);
        assert_eq!(ImageFormat::BC7RgbaUnormSrgb, surface.image_format);
        // Each mipmap must have an integral size in blocks.
        assert_eq!((2 + 2) * 16, surface.data.len());
    }

    #[test]
    fn encode_surface_float32_zero_size() {
        let result = SurfaceRgba32Float {
            width: 0,
            height: 0,
            depth: 0,
            layers: 1,
            mipmaps: 1,
            data: &[0.0; 0],
        }
        .encode(
            ImageFormat::BC7RgbaUnormSrgb,
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
    fn pad_1x1_to_1x1() {
        assert_eq!(
            Cow::<[u8]>::Borrowed(&[1, 2, 3, 4]),
            pad_mipmap_rgba(1, 1, 1, 1, 1, 1, &[1, 2, 3, 4])
        );
    }

    #[test]
    fn pad_1x1_to_2x2() {
        assert_eq!(
            Cow::<[u8]>::Owned(vec![1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            pad_mipmap_rgba(1, 1, 1, 2, 2, 1, &[1, 2, 3, 4])
        );
    }

    #[test]
    fn pad_2x2_to_3x3() {
        assert_eq!(
            Cow::<[u8]>::Owned(vec![
                1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]),
            pad_mipmap_rgba(
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
