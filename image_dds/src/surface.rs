use crate::{calculate_offset, mip_dimension, mip_size, ImageFormat};

// TODO: Add length validation methods that don't overflow.
/// A surface with an image format known at runtime.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
    /// The image data.
    pub data: T,
}

/// An uncompressed RGBA8 surface with 4 bytes per pixel.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
    /// The image data for the surface.
    pub data: T,
}

impl<T: AsRef<[u8]>> SurfaceRgba8<T> {
    // TODO: Add tests for this.
    // TODO: Share code and implement for Surface as well.
    /// Get the range of image data corresponding to the specified `layer` and `mipmap`.
    ///
    /// The dimensions of the image data should be calculated using [mip_dimension].
    /// Returns [None] if the expected range is not fully contained within the buffer.
    pub fn get_image_data(&self, layer: u32, mipmap: u32) -> Option<&[u8]> {
        let format = ImageFormat::R8G8B8A8Unorm;
        let block_size_in_bytes = format.block_size_in_bytes();

        // TODO: Create an error for failed offset calculations?
        let offset = calculate_offset(
            layer,
            mipmap,
            (self.width, self.height, self.depth),
            (1, 1, 1),
            block_size_in_bytes,
            self.mipmaps,
        )?;

        let mip_width = mip_dimension(self.width, mipmap);
        let mip_height = mip_dimension(self.height, mipmap);
        let mip_depth = mip_dimension(self.depth, mipmap);

        // TODO: Create an error for overflow?
        let size = mip_size(
            mip_width as usize,
            mip_height as usize,
            mip_depth as usize,
            1,
            1,
            1,
            block_size_in_bytes,
        )?;

        self.data.as_ref().get(offset..offset + size)
    }
}
