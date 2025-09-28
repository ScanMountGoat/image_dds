use std::borrow::Cow;

use crate::bcn::{encode_bcn, Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7};
use crate::rgba::{
    encode_rgba, Bgr5A1, Bgr8, Bgra4, Bgra8, R16Snorm, R8Snorm, Rf16, Rf32, Rg16, Rg16Snorm, Rg8,
    Rg8Snorm, Rgba16, Rgba16Snorm, Rgba8, Rgba8Snorm, Rgbaf16, Rgbaf32, Rgbf32, Rgf16, Rgf32, R16,
    R8,
};
use crate::{
    downsample_rgba, error::SurfaceError, max_mipmap_count, mip_dimension, ImageFormat, Mipmaps,
    Quality, Surface, SurfaceRgba8,
};
use crate::{
    rgba::convert::{float_to_snorm8, Channel},
    SurfaceRgba32Float,
};

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
    P: Encode + Channel + Default,
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
    P: Default + Encode + Channel,
{
    let block_dimensions = format.block_dimensions();

    // Track the previous image data and dimensions.
    // This enables generating mipmaps from a single base layer.
    let mut mip_data = get_mipmap_data(surface, layer, 0, block_dimensions)?;

    let encoded = mip_data.encode(format, quality)?;
    surface_data.extend_from_slice(&encoded);

    for mipmap in 1..num_mipmaps {
        mip_data = if use_surface {
            // TODO: Error if surface does not have the appropriate number of mipmaps?
            get_mipmap_data(surface, layer, mipmap, block_dimensions)?
        } else {
            mip_data.downsample(
                surface.width(),
                surface.height(),
                surface.depth(),
                block_dimensions,
                mipmap,
            )
        };

        let encoded = mip_data.encode(format, quality)?;
        surface_data.extend_from_slice(&encoded);
    }

    Ok(())
}

struct MipData<T> {
    width: usize,
    height: usize,
    depth: usize,
    data: Vec<T>,
}

impl<T: Channel> MipData<T> {
    fn downsample(
        &self,
        base_width: u32,
        base_height: u32,
        base_depth: u32,
        block_dimensions: (u32, u32, u32),
        mipmap: u32,
    ) -> MipData<T> {
        // Mip dimensions are the padded virtual size of the mipmap.
        // Padding the physical size of the previous mip produces incorrect results.
        let (width, height, depth) = physical_dimensions(
            mip_dimension(base_width, mipmap),
            mip_dimension(base_height, mipmap),
            mip_dimension(base_depth, mipmap),
            block_dimensions,
        );

        // Assume the data is already padded.
        let data = downsample_rgba(
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
}

impl<T> MipData<T>
where
    T: Encode,
{
    fn encode(&self, format: ImageFormat, quality: Quality) -> Result<Vec<u8>, SurfaceError> {
        T::encode(
            self.width as u32,
            self.height as u32 * self.depth as u32,
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
    mipmap: u32,
    block_dimensions: (u32, u32, u32),
) -> Result<MipData<P>, SurfaceError>
where
    S: GetMipmap<P>,
    P: Default + Copy,
{
    let mip_width = mip_dimension(surface.width(), mipmap);
    let mip_height = mip_dimension(surface.height(), mipmap);
    let mip_depth = mip_dimension(surface.depth(), mipmap);

    // TODO: This should be for all depth levels.
    // TODO: This can be optimized to avoid copies?
    let mut data = Vec::new();
    for level in 0..surface.depth() {
        let new_data = surface.get(layer, level, mipmap).unwrap();
        data.extend_from_slice(new_data);
    }

    let (width, height, depth) =
        physical_dimensions(mip_width, mip_height, mip_depth, block_dimensions);

    let data = pad_mipmap_rgba(
        mip_width as usize,
        mip_height as usize,
        mip_depth as usize,
        width,
        height,
        depth,
        &data,
    )
    .to_vec();

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
        width.next_multiple_of(block_width) as usize,
        height.next_multiple_of(block_height) as usize,
        depth.next_multiple_of(block_depth) as usize,
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
) -> Cow<'_, [T]>
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
                encode_bcn::<Bc1, u8>(width, height, data, quality)
            }
            F::BC2RgbaUnorm | F::BC2RgbaUnormSrgb => {
                encode_bcn::<Bc2, u8>(width, height, data, quality)
            }
            F::BC3RgbaUnorm | F::BC3RgbaUnormSrgb => {
                encode_bcn::<Bc3, u8>(width, height, data, quality)
            }
            F::BC4RUnorm | F::BC4RSnorm => encode_bcn::<Bc4, u8>(width, height, data, quality),
            F::BC5RgUnorm | F::BC5RgSnorm => encode_bcn::<Bc5, u8>(width, height, data, quality),
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => {
                encode_bcn::<Bc6, u8>(width, height, data, quality)
            }
            F::BC7RgbaUnorm | F::BC7RgbaUnormSrgb => {
                encode_bcn::<Bc7, u8>(width, height, data, quality)
            }
            F::R8Unorm => encode_rgba::<R8, u8>(width, height, data),
            F::R8Snorm => encode_rgba::<R8Snorm, u8>(width, height, data),
            F::Rg8Unorm => encode_rgba::<Rg8, u8>(width, height, data),
            F::Rg8Snorm => encode_rgba::<Rg8Snorm, u8>(width, height, data),
            F::Rgba8Unorm | F::Rgba8UnormSrgb => encode_rgba::<Rgba8, u8>(width, height, data),
            F::Rgba8Snorm => encode_rgba::<Rgba8Snorm, u8>(width, height, data),
            F::Bgra8Unorm | F::Bgra8UnormSrgb => encode_rgba::<Bgra8, u8>(width, height, data),
            F::Bgra4Unorm => encode_rgba::<Bgra4, u8>(width, height, data),
            F::Bgr8Unorm => encode_rgba::<Bgr8, u8>(width, height, data),
            F::R16Unorm => encode_rgba::<R16, u8>(width, height, data),
            F::R16Snorm => encode_rgba::<R16Snorm, u8>(width, height, data),
            F::Rg16Unorm => encode_rgba::<Rg16, u8>(width, height, data),
            F::Rg16Snorm => encode_rgba::<Rg16Snorm, u8>(width, height, data),
            F::Rgba16Unorm => encode_rgba::<Rgba16, u8>(width, height, data),
            F::Rgba16Snorm => encode_rgba::<Rgba16Snorm, u8>(width, height, data),
            F::R16Float => encode_rgba::<Rf32, u8>(width, height, data),
            F::Rg16Float => encode_rgba::<Rgf16, u8>(width, height, data),
            F::Rgba16Float => encode_rgba::<Rgbaf16, u8>(width, height, data),
            F::R32Float => encode_rgba::<Rf32, u8>(width, height, data),
            F::Rg32Float => encode_rgba::<Rgf32, u8>(width, height, data),
            F::Rgb32Float => encode_rgba::<Rgbf32, u8>(width, height, data),
            F::Rgba32Float => encode_rgba::<Rgbaf32, u8>(width, height, data),
            F::Bgr5A1Unorm => encode_rgba::<Bgr5A1, u8>(width, height, data),
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
            F::R8Snorm => encode_rgba::<R8Snorm, f32>(width, height, data),
            F::Rg8Snorm => encode_rgba::<Rg8Snorm, f32>(width, height, data),
            F::Rgba8Snorm => encode_rgba::<Rgba8Snorm, f32>(width, height, data),
            F::BC4RSnorm | F::BC5RgSnorm => {
                // intel_tex doesn't have a dedicated encoder for snorm formats.
                let rgba8: Vec<_> = data.iter().map(|f| float_to_snorm8(*f) as u8).collect();
                u8::encode(width, height, &rgba8, format, quality)
            }
            F::BC6hRgbUfloat | F::BC6hRgbSfloat => {
                encode_bcn::<Bc6, f32>(width, height, data, quality)
            }
            F::R16Float => encode_rgba::<Rf16, f32>(width, height, data),
            F::Rg16Float => encode_rgba::<Rgf16, f32>(width, height, data),
            F::Rgba16Float => encode_rgba::<Rgbaf16, f32>(width, height, data),
            F::R32Float => encode_rgba::<Rf32, f32>(width, height, data),
            F::Rg32Float => encode_rgba::<Rgf32, f32>(width, height, data),
            F::Rgba32Float => encode_rgba::<Rgbaf32, f32>(width, height, data),
            F::R16Unorm => encode_rgba::<R16, f32>(width, height, data),
            F::Rg16Unorm => encode_rgba::<Rg16, f32>(width, height, data),
            F::Rgba16Unorm => encode_rgba::<Rgba16, f32>(width, height, data),
            F::R16Snorm => encode_rgba::<R16Snorm, f32>(width, height, data),
            F::Rg16Snorm => encode_rgba::<Rg16Snorm, f32>(width, height, data),
            F::Rgba16Snorm => encode_rgba::<Rgba16Snorm, f32>(width, height, data),
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

    use strum::IntoEnumIterator;

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
    fn encode_surface_float32_cube_mipmaps_length() {
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
    fn encode_surface_float32_2d_mipmaps() {
        let surface = SurfaceRgba32Float {
            width: 3,
            height: 3,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: &(0..36).map(|i| i as f32).collect::<Vec<_>>(),
        }
        .encode(
            ImageFormat::Rgba32Float,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(
            Surface {
                width: 3,
                height: 3,
                depth: 1,
                layers: 1,
                mipmaps: 2,
                image_format: ImageFormat::Rgba32Float,
                data: bytemuck::cast_slice::<[f32; 4], u8>(&[
                    [0.0, 1.0, 2.0, 3.0],
                    [4.0, 5.0, 6.0, 7.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [12.0, 13.0, 14.0, 15.0],
                    [16.0, 17.0, 18.0, 19.0],
                    [20.0, 21.0, 22.0, 23.0],
                    [24.0, 25.0, 26.0, 27.0],
                    [28.0, 29.0, 30.0, 31.0],
                    [32.0, 33.0, 34.0, 35.0],
                    [8.0, 9.0, 10.0, 11.0],
                ])
                .to_vec()
            },
            surface
        );
    }

    #[test]
    fn encode_surface_float32_3d_mipmaps() {
        let surface = SurfaceRgba32Float {
            width: 3,
            height: 3,
            depth: 3,
            layers: 1,
            mipmaps: 1,
            data: &(0..108).map(|i| i as f32).collect::<Vec<_>>(),
        }
        .encode(
            ImageFormat::Rgba32Float,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        dbg!(bytemuck::cast_slice::<_, [f32; 4]>(&surface.data));

        assert_eq!(
            Surface {
                width: 3,
                height: 3,
                depth: 3,
                layers: 1,
                mipmaps: 2,
                image_format: ImageFormat::Rgba32Float,
                data: bytemuck::cast_slice::<[f32; 4], u8>(&[
                    [0.0, 1.0, 2.0, 3.0],
                    [4.0, 5.0, 6.0, 7.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [12.0, 13.0, 14.0, 15.0],
                    [16.0, 17.0, 18.0, 19.0],
                    [20.0, 21.0, 22.0, 23.0],
                    [24.0, 25.0, 26.0, 27.0],
                    [28.0, 29.0, 30.0, 31.0],
                    [32.0, 33.0, 34.0, 35.0],
                    [36.0, 37.0, 38.0, 39.0],
                    [40.0, 41.0, 42.0, 43.0],
                    [44.0, 45.0, 46.0, 47.0],
                    [48.0, 49.0, 50.0, 51.0],
                    [52.0, 53.0, 54.0, 55.0],
                    [56.0, 57.0, 58.0, 59.0],
                    [60.0, 61.0, 62.0, 63.0],
                    [64.0, 65.0, 66.0, 67.0],
                    [68.0, 69.0, 70.0, 71.0],
                    [72.0, 73.0, 74.0, 75.0],
                    [76.0, 77.0, 78.0, 79.0],
                    [80.0, 81.0, 82.0, 83.0],
                    [84.0, 85.0, 86.0, 87.0],
                    [88.0, 89.0, 90.0, 91.0],
                    [92.0, 93.0, 94.0, 95.0],
                    [96.0, 97.0, 98.0, 99.0],
                    [100.0, 101.0, 102.0, 103.0],
                    [104.0, 105.0, 106.0, 107.0],
                    [26.0, 27.0, 28.0, 29.0],
                ])
                .to_vec()
            },
            surface
        );
    }

    #[test]
    fn encode_surface_float32_cube_mipmaps() {
        let surface = SurfaceRgba32Float {
            width: 3,
            height: 3,
            depth: 1,
            layers: 6,
            mipmaps: 1,
            data: &(0..216).map(|i| i as f32).collect::<Vec<_>>(),
        }
        .encode(
            ImageFormat::Rgba32Float,
            Quality::Fast,
            Mipmaps::GeneratedAutomatic,
        )
        .unwrap();

        assert_eq!(
            Surface {
                width: 3,
                height: 3,
                depth: 1,
                layers: 6,
                mipmaps: 2,
                image_format: ImageFormat::Rgba32Float,
                data: bytemuck::cast_slice::<[f32; 4], u8>(&[
                    [0.0, 1.0, 2.0, 3.0],
                    [4.0, 5.0, 6.0, 7.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [12.0, 13.0, 14.0, 15.0],
                    [16.0, 17.0, 18.0, 19.0],
                    [20.0, 21.0, 22.0, 23.0],
                    [24.0, 25.0, 26.0, 27.0],
                    [28.0, 29.0, 30.0, 31.0],
                    [32.0, 33.0, 34.0, 35.0],
                    [8.0, 9.0, 10.0, 11.0],
                    [36.0, 37.0, 38.0, 39.0],
                    [40.0, 41.0, 42.0, 43.0],
                    [44.0, 45.0, 46.0, 47.0],
                    [48.0, 49.0, 50.0, 51.0],
                    [52.0, 53.0, 54.0, 55.0],
                    [56.0, 57.0, 58.0, 59.0],
                    [60.0, 61.0, 62.0, 63.0],
                    [64.0, 65.0, 66.0, 67.0],
                    [68.0, 69.0, 70.0, 71.0],
                    [44.0, 45.0, 46.0, 47.0],
                    [72.0, 73.0, 74.0, 75.0],
                    [76.0, 77.0, 78.0, 79.0],
                    [80.0, 81.0, 82.0, 83.0],
                    [84.0, 85.0, 86.0, 87.0],
                    [88.0, 89.0, 90.0, 91.0],
                    [92.0, 93.0, 94.0, 95.0],
                    [96.0, 97.0, 98.0, 99.0],
                    [100.0, 101.0, 102.0, 103.0],
                    [104.0, 105.0, 106.0, 107.0],
                    [80.0, 81.0, 82.0, 83.0],
                    [108.0, 109.0, 110.0, 111.0],
                    [112.0, 113.0, 114.0, 115.0],
                    [116.0, 117.0, 118.0, 119.0],
                    [120.0, 121.0, 122.0, 123.0],
                    [124.0, 125.0, 126.0, 127.0],
                    [128.0, 129.0, 130.0, 131.0],
                    [132.0, 133.0, 134.0, 135.0],
                    [136.0, 137.0, 138.0, 139.0],
                    [140.0, 141.0, 142.0, 143.0],
                    [116.0, 117.0, 118.0, 119.0],
                    [144.0, 145.0, 146.0, 147.0],
                    [148.0, 149.0, 150.0, 151.0],
                    [152.0, 153.0, 154.0, 155.0],
                    [156.0, 157.0, 158.0, 159.0],
                    [160.0, 161.0, 162.0, 163.0],
                    [164.0, 165.0, 166.0, 167.0],
                    [168.0, 169.0, 170.0, 171.0],
                    [172.0, 173.0, 174.0, 175.0],
                    [176.0, 177.0, 178.0, 179.0],
                    [152.0, 153.0, 154.0, 155.0],
                    [180.0, 181.0, 182.0, 183.0],
                    [184.0, 185.0, 186.0, 187.0],
                    [188.0, 189.0, 190.0, 191.0],
                    [192.0, 193.0, 194.0, 195.0],
                    [196.0, 197.0, 198.0, 199.0],
                    [200.0, 201.0, 202.0, 203.0],
                    [204.0, 205.0, 206.0, 207.0],
                    [208.0, 209.0, 210.0, 211.0],
                    [212.0, 213.0, 214.0, 215.0],
                    [188.0, 189.0, 190.0, 191.0],
                ])
                .to_vec()
            },
            surface
        );
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

    #[test]
    fn encode_all_u8() {
        for image_format in ImageFormat::iter() {
            let surface = SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 4 * 4 * 4],
            };
            surface
                .encode(image_format, Quality::Normal, Mipmaps::GeneratedAutomatic)
                .unwrap();
        }
    }

    #[test]
    fn encode_all_f32() {
        for image_format in ImageFormat::iter() {
            let surface = SurfaceRgba32Float {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 4 * 4 * 4],
            };
            surface
                .encode(image_format, Quality::Normal, Mipmaps::GeneratedAutomatic)
                .unwrap();
        }
    }
}
