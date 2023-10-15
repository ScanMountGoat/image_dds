use crate::{
    calculate_offset, error::CreateImageError, max_mipmap_count, mip_dimension, mip_size,
    ImageFormat, SurfaceError,
};

/// A surface with an image format known at runtime.
#[derive(Debug, PartialEq)]
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
    /// Get the range of image data corresponding to the specified `layer` and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, mipmap: u32) -> Option<&[u8]> {
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            self.image_format,
            layer,
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

/// An uncompressed [ImageFormat::R8G8B8A8Unorm] surface with 4 bytes per pixel.
#[derive(Debug, PartialEq)]
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

impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    /// Get the range of image data corresponding to the specified `layer` and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, mipmap: u32) -> Option<&[u8]> {
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            ImageFormat::R8G8B8A8Unorm,
            layer,
            mipmap,
        )
    }

    pub(crate) fn validate(&self) -> Result<(), SurfaceError> {
        Surface {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            image_format: ImageFormat::R8G8B8A8Unorm,
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
}

#[cfg(feature = "image")]
impl SurfaceRgba8<Vec<u8>> {
    // TODO: Allow configuring the output like cross, horizontal, etc?
    /// Create an image for all layers and depth slices for the given `mipmap`.
    ///
    /// Array layers are arranged vertically from top to bottom.
    pub fn to_image(&self, mipmap: u32) -> Result<image::RgbaImage, CreateImageError> {
        // Mipmaps have different dimensions.
        // A single 2D image can only represent data from a single mip level across layers.
        let image_data: Vec<_> = (0..self.layers)
            .flat_map(|layer| self.get(layer, mipmap).unwrap())
            .copied()
            .collect();
        let data_length = image_data.len();

        // Arrange depth slices horizontally and array layers vertically.
        let width = mip_dimension(self.width, mipmap) * mip_dimension(self.depth, mipmap);
        let height = mip_dimension(self.height, mipmap) * self.layers;

        image::RgbaImage::from_raw(width, height, image_data).ok_or(
            crate::CreateImageError::InvalidSurfaceDimensions {
                width,
                height,
                data_length,
            },
        )
    }
}

/// An uncompressed [ImageFormat::R32G32B32A32Float] surface with 16 bytes per pixel.
#[derive(Debug, PartialEq)]
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

impl<T: AsRef<[f32]>> SurfaceRgba32Float<T> {
    /// Get the range of image data corresponding to the specified `layer` and `mipmap`.
    ///
    /// The dimensions of the returned data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get(&self, layer: u32, mipmap: u32) -> Option<&[f32]> {
        // TODO: Is it safe to cast like this?
        get_mipmap(
            self.data.as_ref(),
            (self.width, self.height, self.depth),
            self.mipmaps,
            ImageFormat::R32G32B32A32Float,
            layer,
            mipmap,
        )
        .map(bytemuck::cast_slice)
    }

    pub(crate) fn validate(&self) -> Result<(), SurfaceError> {
        Surface {
            width: self.width,
            height: self.height,
            depth: self.depth,
            layers: self.layers,
            mipmaps: self.mipmaps,
            image_format: ImageFormat::R32G32B32A32Float,
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
}

#[cfg(feature = "image")]
impl SurfaceRgba32Float<Vec<f32>> {
    // TODO: Allow configuring the output like cross, horizontal, etc?
    /// Create an image for all layers and depth slices for the given `mipmap`.
    ///
    /// Array layers are arranged vertically from top to bottom.
    pub fn to_image(&self, mipmap: u32) -> Result<image::Rgba32FImage, CreateImageError> {
        // Mipmaps have different dimensions.
        // A single 2D image can only represent data from a single mip level across layers.
        let image_data: Vec<_> = (0..self.layers)
            .flat_map(|layer| self.get(layer, mipmap).unwrap())
            .copied()
            .collect();
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

// TODO: Add tests for this.
fn get_mipmap<T>(
    data: &[T],
    dimensions: (u32, u32, u32),
    mipmaps: u32,
    format: ImageFormat,
    layer: u32,
    mipmap: u32,
) -> Option<&[T]> {
    let (width, height, depth) = dimensions;

    let block_size_in_bytes = format.block_size_in_bytes();
    let block_dimensions = format.block_dimensions();

    // TODO: Create an error for failed offset calculations?
    let offset_in_bytes = calculate_offset(
        layer,
        mipmap,
        (width, height, depth),
        block_dimensions,
        block_size_in_bytes,
        mipmaps,
    )?;

    let mip_width = mip_dimension(width, mipmap);
    let mip_height = mip_dimension(height, mipmap);
    let mip_depth = mip_dimension(depth, mipmap);

    // TODO: Create an error for overflow?
    let size_in_bytes = mip_size(
        mip_width as usize,
        mip_height as usize,
        mip_depth as usize,
        block_dimensions.0 as usize,
        block_dimensions.1 as usize,
        block_dimensions.2 as usize,
        block_size_in_bytes,
    )?;

    let start = offset_in_bytes / std::mem::size_of::<T>();
    let count = size_in_bytes / std::mem::size_of::<T>();
    data.get(start..start + count)
}
