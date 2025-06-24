use crate::{
    calculate_offset, error::CreateImageError, max_mipmap_count, mip_dimension, mip_size,
    ImageFormat, SurfaceError,
};

/// A surface with an image format known at runtime.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Surface<T> {
    /// The width of the surface in pixels.
    pub width: u32,
    /// The height of the surface in pixels.
    pub height: u32,
    /// The depth of the surface in pixels.
    /// This should be `1` for 2D surfaces.
    pub depth: u32,
    /// The number of array layers in the surface.
    /// This should be `1` for most surfaces and `6` for cube maps.
    pub layers: u32,
    /// The number of mipmaps in the surface.
    /// This should be `1` if the surface has only the base mip level.
    /// All array layers are assumed to have the same number of mipmaps.
    pub mipmaps: u32,
    /// The format of the bytes in [data](#structfield.data).
    pub image_format: ImageFormat,
    /// The combined image data ordered by layer and then mipmap without additional padding.
    ///
    /// A surface with L layers and M mipmaps would have the following layout:
    /// Layer 0 Mip 0, Layer 0 Mip 1,  ..., Layer L-1 Mip M-1
    pub data: T,
}

impl<T: AsRef<[u8]>> Surface<T> {
    /// Get the range of image data corresponding to the specified `layer`, `depth_level`, and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[u8]> {
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            self.image_format,
            layer,
            depth_level,
            mipmap,
        )
    }

    // TODO: Add tests for each of these cases.
    pub(crate) fn validate(&self) -> Result<(), SurfaceError> {
        if self.width == 0 || self.height == 0 || self.depth == 0 {
            return Err(SurfaceError::ZeroSizedSurface {
                width: self.width,
                height: self.height,
                depth: self.depth,
            });
        }

        let max_mipmaps = max_mipmap_count(self.width.max(self.height).max(self.depth));
        if self.mipmaps > max_mipmaps {
            return Err(SurfaceError::UnexpectedMipmapCount {
                mipmaps: self.mipmaps,
                max_mipmaps,
            });
        }

        let (block_width, block_height, block_depth) = self.image_format.block_dimensions();
        let block_size_in_bytes = self.image_format.block_size_in_bytes();
        let base_layer_size = mip_size(
            self.width as usize,
            self.height as usize,
            self.depth as usize,
            block_width as usize,
            block_height as usize,
            block_depth as usize,
            block_size_in_bytes,
        )
        .ok_or(SurfaceError::PixelCountWouldOverflow {
            width: self.width,
            height: self.height,
            depth: self.depth,
        })?;

        // TODO: validate the combined length of layers + mipmaps.
        // TODO: Calculate the correct expected size.
        if base_layer_size > self.data.as_ref().len() {
            return Err(SurfaceError::NotEnoughData {
                expected: base_layer_size,
                actual: self.data.as_ref().len(),
            });
        }

        // TODO: Return the mipmap and array offsets.
        Ok(())
    }
}

impl<T> Surface<Vec<T>> {
    /// Convert to a surface with borrowed data.
    pub fn as_ref(&self) -> Surface<&[T]> {
        Surface {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            image_format: self.image_format,
            data: self.data.as_ref(),
        }
    }
}

/// An uncompressed [ImageFormat::Rgba8Unorm] surface with 4 bytes per pixel.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceRgba8<T> {
    /// The width of the surface in pixels.
    pub width: u32,
    /// The height of the surface in pixels.
    pub height: u32,
    /// The depth of the surface in pixels.
    /// This should be `1` for 2D surfaces.
    pub depth: u32,
    /// The number of array layers in the surface.
    /// This should be `1` for most surfaces and `6` for cube maps.
    pub layers: u32,
    /// The number of mipmaps in the surface.
    /// This should be `1` if the surface has only the base mip level.
    /// All array layers are assumed to have the same number of mipmaps.
    pub mipmaps: u32,
    /// The combined image data ordered by layer and then mipmap without additional padding.
    ///
    /// A surface with L layers and M mipmaps would have the following layout:
    /// Layer 0 Mip 0, Layer 0 Mip 1,  ..., Layer L-1 Mip M-1
    pub data: T,
}

impl<T> SurfaceRgba8<Vec<T>> {
    /// Convert to a surface with borrowed data.
    pub fn as_ref(&self) -> SurfaceRgba8<&[T]> {
        SurfaceRgba8 {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            data: self.data.as_ref(),
        }
    }
}

impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    /// Get the range of 2D image data corresponding to the specified `layer`, `depth_level`, and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[u8]> {
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            ImageFormat::Rgba8Unorm,
            layer,
            depth_level,
            mipmap,
        )
    }

    /// Get the image corresponding to the specified `layer`, `depth_level`, and `mipmap`.
    ///
    /// Returns [None] if the expected range is not fully contained within the buffer.
    #[cfg(feature = "image")]
    pub fn get_image(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<image::RgbaImage> {
        self.get(layer, depth_level, mipmap).and_then(|data| {
            image::RgbaImage::from_raw(
                mip_dimension(self.width, mipmap),
                mip_dimension(self.height, mipmap),
                data.to_vec(),
            )
        })
    }

    pub(crate) fn validate(&self) -> Result<(), SurfaceError> {
        Surface {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            image_format: ImageFormat::Rgba8Unorm,
            data: self.data.as_ref(),
        }
        .validate()
    }
}

#[cfg(feature = "image")]
impl<'a> SurfaceRgba8<&'a [u8]> {
    /// Create a 2D view over the data in `image` without any copies.
    pub fn from_image(image: &'a image::RgbaImage) -> Self {
        SurfaceRgba8 {
            width: image.width(),
            height: image.height(),
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }

    /// Create a 2D view with layers over the data in `image` without any copies.
    ///
    /// Array layers should be stacked vertically in `image` with an overall height `height*layers`.
    pub fn from_image_layers(image: &'a image::RgbaImage, layers: u32) -> Self {
        SurfaceRgba8 {
            width: image.width(),
            height: image.height() / layers,
            depth: 1,
            layers,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }

    /// Create a 3D view over the data in `image` without any copies.
    ///
    /// Depth slices should be stacked vertically in `image` with an overall height `height*depth`.
    pub fn from_image_depth(image: &'a image::RgbaImage, depth: u32) -> Self {
        SurfaceRgba8 {
            width: image.width(),
            height: image.height() / depth,
            depth,
            layers: 1,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }
}

#[cfg(feature = "image")]
impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    /// Create an image for all layers and depth slices for the given `mipmap`.
    ///
    /// Array layers and depth slices are arranged vertically from top to bottom.
    pub fn to_image(&self, mipmap: u32) -> Result<image::RgbaImage, CreateImageError> {
        // Mipmaps have different dimensions.
        // A single 2D image can only represent data from a single mip level across layers.
        let mut image_data = Vec::new();
        for layer in 0..self.layers {
            for level in 0..self.depth {
                let data = self.get(layer, level, mipmap).unwrap();
                image_data.extend_from_slice(data);
            }
        }
        let data_length = image_data.len();

        // Arrange depth and array layers vertically.
        // This layout allows copyless conversions to an RGBA8 surface.
        let width = mip_dimension(self.width, mipmap);
        let height =
            mip_dimension(self.height, mipmap) * mip_dimension(self.depth, mipmap) * self.layers;

        image::RgbaImage::from_raw(width, height, image_data).ok_or(
            crate::CreateImageError::InvalidSurfaceDimensions {
                width,
                height,
                data_length,
            },
        )
    }
}

#[cfg(feature = "image")]
impl SurfaceRgba8<Vec<u8>> {
    /// Create an image for all layers and depth slices without copying.
    ///
    /// Fails if the surface has more than one mipmap.
    /// Array layers and depth slices are arranged vertically from top to bottom.
    pub fn into_image(self) -> Result<image::RgbaImage, CreateImageError> {
        // Arrange depth and array layers vertically.
        // This layout allows copyless conversions to an RGBA8 surface.
        let width = self.width;
        let height = self.height * self.depth * self.layers;

        if self.mipmaps > 1 {
            return Err(CreateImageError::UnexpectedMipmapCount {
                mipmaps: self.mipmaps,
                max_mipmaps: 1,
            });
        }

        let data_length = self.data.len();
        image::RgbaImage::from_raw(width, height, self.data).ok_or(
            crate::CreateImageError::InvalidSurfaceDimensions {
                width,
                height,
                data_length,
            },
        )
    }
}

/// An uncompressed [ImageFormat::Rgba32Float] surface with 16 bytes per pixel.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceRgba32Float<T> {
    /// The width of the surface in pixels.
    pub width: u32,
    /// The height of the surface in pixels.
    pub height: u32,
    /// The depth of the surface in pixels.
    /// This should be `1` for 2D surfaces.
    pub depth: u32,
    /// The number of array layers in the surface.
    /// This should be `1` for most surfaces and `6` for cube maps.
    pub layers: u32,
    /// The number of mipmaps in the surface.
    /// This should be `1` if the surface has only the base mip level.
    /// All array layers are assumed to have the same number of mipmaps.
    pub mipmaps: u32,
    /// The combined `f32` image data ordered by layer and then mipmap without additional padding.
    ///
    /// A surface with L layers and M mipmaps would have the following layout:
    /// Layer 0 Mip 0, Layer 0 Mip 1,  ..., Layer L-1 Mip M-1
    pub data: T,
}

impl<T> SurfaceRgba32Float<Vec<T>> {
    /// Convert to a surface with borrowed data.
    pub fn as_ref(&self) -> SurfaceRgba32Float<&[T]> {
        SurfaceRgba32Float {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            data: self.data.as_ref(),
        }
    }
}

impl<T: AsRef<[f32]>> SurfaceRgba32Float<T> {
    /// Get the range of 2D image data corresponding to the specified `layer`, `depth_level`, and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, depth_level: u32, mipmap: u32) -> Option<&[f32]> {
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            ImageFormat::Rgba32Float,
            layer,
            depth_level,
            mipmap,
        )
    }

    /// Get the image corresponding to the specified `layer`, `depth_level`, and `mipmap`.
    ///
    /// Returns [None] if the expected range is not fully contained within the buffer.
    #[cfg(feature = "image")]
    pub fn get_image(
        &self,
        layer: u32,
        depth_level: u32,
        mipmap: u32,
    ) -> Option<image::Rgba32FImage> {
        self.get(layer, depth_level, mipmap).and_then(|data| {
            image::Rgba32FImage::from_raw(
                mip_dimension(self.width, mipmap),
                mip_dimension(self.height, mipmap),
                data.to_vec(),
            )
        })
    }

    pub(crate) fn validate(&self) -> Result<(), SurfaceError> {
        Surface {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            image_format: ImageFormat::Rgba32Float,
            data: bytemuck::cast_slice(self.data.as_ref()),
        }
        .validate()
    }
}

#[cfg(feature = "image")]
impl<'a> SurfaceRgba32Float<&'a [f32]> {
    /// Create a 2D view over the data in `image` without any copies.
    pub fn from_image(image: &'a image::Rgba32FImage) -> Self {
        SurfaceRgba32Float {
            width: image.width(),
            height: image.height(),
            depth: 1,
            layers: 1,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }

    /// Create a 2D view with layers over the data in `image` without any copies.
    ///
    /// Array layers should be stacked vertically in `image` with an overall height `height*layers`.
    pub fn from_image_layers(image: &'a image::Rgba32FImage, layers: u32) -> Self {
        SurfaceRgba32Float {
            width: image.width(),
            height: image.height() / layers,
            depth: 1,
            layers,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }

    /// Create a 3D view over the data in `image` without any copies.
    ///
    /// Depth slices should be stacked vertically in `image` with an overall height `height*depth`.
    pub fn from_image_depth(image: &'a image::Rgba32FImage, depth: u32) -> Self {
        SurfaceRgba32Float {
            width: image.width(),
            height: image.height() / depth,
            depth,
            layers: 1,
            mipmaps: 1,
            data: image.as_raw(),
        }
    }
}

#[cfg(feature = "image")]
impl<T: AsRef<[f32]>> SurfaceRgba32Float<T> {
    /// Create an image for all layers and depth slices for the given `mipmap`.
    ///
    /// Array layers are arranged vertically from top to bottom.
    pub fn to_image(&self, mipmap: u32) -> Result<image::Rgba32FImage, CreateImageError> {
        // Mipmaps have different dimensions.
        // A single 2D image can only represent data from a single mip level across layers.
        let mut image_data = Vec::new();
        for layer in 0..self.layers {
            for level in 0..self.depth {
                let data = self.get(layer, level, mipmap).unwrap();
                image_data.extend_from_slice(data);
            }
        }
        let data_length = image_data.len();

        // Arrange depth slices horizontally and array layers vertically.
        let width = mip_dimension(self.width, mipmap) * mip_dimension(self.depth, mipmap);
        let height = mip_dimension(self.height, mipmap) * self.layers;

        image::Rgba32FImage::from_raw(width, height, image_data).ok_or(
            crate::CreateImageError::InvalidSurfaceDimensions {
                width,
                height,
                data_length,
            },
        )
    }
}

#[cfg(feature = "image")]
impl SurfaceRgba32Float<Vec<f32>> {
    /// Create an image for all layers and depth slices without copying.
    ///
    /// Fails if the surface has more than one mipmap.
    /// Array layers and depth slices are arranged vertically from top to bottom.
    pub fn into_image(self) -> Result<image::Rgba32FImage, CreateImageError> {
        // Arrange depth and array layers vertically.
        // This layout allows copyless conversions to an RGBA8 surface.
        let width = self.width;
        let height = self.height * self.depth * self.layers;

        if self.mipmaps > 1 {
            return Err(CreateImageError::UnexpectedMipmapCount {
                mipmaps: self.mipmaps,
                max_mipmaps: 1,
            });
        }

        let data_length = self.data.len();
        image::Rgba32FImage::from_raw(width, height, self.data).ok_or(
            crate::CreateImageError::InvalidSurfaceDimensions {
                width,
                height,
                data_length,
            },
        )
    }
}

// TODO: Add tests for this.
fn get_mipmap<T>(
    data: &[T],
    dimensions: (u32, u32, u32),
    mipmaps: u32,
    format: ImageFormat,
    layer: u32,
    depth_level: u32,
    mipmap: u32,
) -> Option<&[T]> {
    let (width, height, depth) = dimensions;

    let block_size_in_bytes = format.block_size_in_bytes();
    let block_dimensions = format.block_dimensions();

    // TODO: Create an error for failed offset calculations?
    let offset_in_bytes = calculate_offset(
        layer,
        depth_level,
        mipmap,
        (width, height, depth),
        block_dimensions,
        block_size_in_bytes,
        mipmaps,
    )?;

    // The returned slice is always 2D.
    let mip_width = mip_dimension(width, mipmap);
    let mip_height = mip_dimension(height, mipmap);

    // TODO: Create an error for overflow?
    let size_in_bytes = mip_size(
        mip_width as usize,
        mip_height as usize,
        1,
        block_dimensions.0 as usize,
        block_dimensions.1 as usize,
        block_dimensions.2 as usize,
        block_size_in_bytes,
    )?;

    let start = offset_in_bytes / std::mem::size_of::<T>();
    let count = size_in_bytes / std::mem::size_of::<T>();
    data.get(start..start + count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_as_ref() {
        assert_eq!(
            Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format: ImageFormat::BC7RgbaUnorm,
                data: &[0u8; 4 * 4][..],
            },
            Surface {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                image_format: ImageFormat::BC7RgbaUnorm,
                data: vec![0u8; 4 * 4],
            }
            .as_ref()
        );
    }

    #[test]
    fn surface_rgba8_as_ref() {
        assert_eq!(
            SurfaceRgba8 {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: &[0u8; 4 * 5 * 4][..],
            },
            SurfaceRgba8 {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 4 * 5 * 4],
            }
            .as_ref()
        );
    }

    #[test]
    fn surface_rgbaf32_as_ref() {
        assert_eq!(
            SurfaceRgba32Float {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: &[0.0; 4 * 5 * 4][..],
            },
            SurfaceRgba32Float {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 4 * 5 * 4],
            }
            .as_ref()
        );
    }

    #[test]
    fn surface_rgba8_to_image() {
        assert_eq!(
            image::RgbaImage::new(4, 5),
            SurfaceRgba8 {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 4 * 5 * 4],
            }
            .to_image(0)
            .unwrap()
        );
    }

    #[test]
    fn surface_rgba8_into_image() {
        assert_eq!(
            image::RgbaImage::new(4, 5),
            SurfaceRgba8 {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 4 * 5 * 4],
            }
            .into_image()
            .unwrap()
        );
    }

    #[test]
    fn surface_rgba8_get_image() {
        assert_eq!(
            image::RgbaImage::new(4, 5),
            SurfaceRgba8 {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0u8; 4 * 5 * 4],
            }
            .get_image(0, 0, 0)
            .unwrap()
        );
    }

    #[test]
    fn surface_rgba8_into_image_invalid_mipmaps() {
        assert_eq!(
            Err(CreateImageError::UnexpectedMipmapCount {
                mipmaps: 2,
                max_mipmaps: 1
            }),
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 2,
                data: vec![0u8; 4 * 4 * 2 * 4],
            }
            .into_image()
        );
    }

    #[test]
    fn surface_rgbaf32_to_image() {
        assert_eq!(
            image::Rgba32FImage::new(4, 5),
            SurfaceRgba32Float {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 4 * 5 * 4],
            }
            .to_image(0)
            .unwrap()
        );
    }

    #[test]
    fn surface_rgbaf32_into_image() {
        assert_eq!(
            image::Rgba32FImage::new(4, 5),
            SurfaceRgba32Float {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 4 * 5 * 4],
            }
            .into_image()
            .unwrap()
        );
    }

    #[test]
    fn surface_rgbaf32_into_image_invalid_mipmaps() {
        assert_eq!(
            Err(CreateImageError::UnexpectedMipmapCount {
                mipmaps: 2,
                max_mipmaps: 1
            }),
            SurfaceRgba32Float {
                width: 4,
                height: 4,
                depth: 1,
                layers: 1,
                mipmaps: 2,
                data: vec![0.0; 4 * 4 * 2 * 4],
            }
            .into_image()
        );
    }

    #[test]
    fn surface_rgbaf32_get_image() {
        assert_eq!(
            image::Rgba32FImage::new(4, 5),
            SurfaceRgba32Float {
                width: 4,
                height: 5,
                depth: 1,
                layers: 1,
                mipmaps: 1,
                data: vec![0.0; 4 * 5 * 4],
            }
            .get_image(0, 0, 0)
            .unwrap()
        );
    }

    #[test]
    fn surface_rgba_from_image_depth() {
        let image = image::RgbaImage::new(4, 24);
        assert_eq!(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 6,
                layers: 1,
                mipmaps: 1,
                data: &[0; 4 * 4 * 6 * 4][..],
            },
            SurfaceRgba8::from_image_depth(&image, 6)
        );
    }

    #[test]
    fn surface_rgbaf32_from_image_depth() {
        let image = image::Rgba32FImage::new(4, 24);
        assert_eq!(
            SurfaceRgba32Float {
                width: 4,
                height: 4,
                depth: 6,
                layers: 1,
                mipmaps: 1,
                data: &[0.0; 4 * 4 * 6 * 4][..],
            },
            SurfaceRgba32Float::from_image_depth(&image, 6)
        );
    }

    #[test]
    fn surface_rgba_from_image_layers() {
        let image = image::RgbaImage::new(4, 24);
        assert_eq!(
            SurfaceRgba8 {
                width: 4,
                height: 4,
                depth: 1,
                layers: 6,
                mipmaps: 1,
                data: &[0; 4 * 4 * 6 * 4][..],
            },
            SurfaceRgba8::from_image_layers(&image, 6)
        );
    }

    #[test]
    fn surface_rgbaf32_from_image_layers() {
        let image = image::Rgba32FImage::new(4, 24);
        assert_eq!(
            SurfaceRgba32Float {
                width: 4,
                height: 4,
                depth: 1,
                layers: 6,
                mipmaps: 1,
                data: &[0.0; 4 * 4 * 6 * 4][..],
            },
            SurfaceRgba32Float::from_image_layers(&image, 6)
        );
    }
}
