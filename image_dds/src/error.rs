use thiserror::Error;

use crate::{DdsFormatInfo, ImageFormat};

/// Errors that can occur while creating a decoded image.
#[derive(Debug, Error)]
pub enum CreateImageError {
    #[error("data length {data_length} is not valid for a {width}x{height} image")]
    InvalidSurfaceDimensions {
        width: u32,
        height: u32,
        data_length: usize,
    },

    #[error("error decompressing surface: {0}")]
    DecompressSurface(#[from] SurfaceError),

    #[error("{mipmaps} mipmaps exceeds the maximum expected mipmap count of {max_mipmaps}")]
    UnexpectedMipmapCount { mipmaps: u32, max_mipmaps: u32 },
}

/// Errors that can occur while encoding or decoding a surface.
#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("surface dimensions {width} x {height} x {depth} contain no pixels")]
    ZeroSizedSurface { width: u32, height: u32, depth: u32 },

    #[error("surface pixel count {width} x {height} x {depth} would overflow")]
    PixelCountWouldOverflow { width: u32, height: u32, depth: u32 },

    #[error("surface dimensions {width} x {height} x {depth} are not divisibly by the block dimensions {block_width} x {block_height}")]
    NonIntegralDimensionsInBlocks {
        width: u32,
        height: u32,
        depth: u32,
        block_width: u32,
        block_height: u32,
    },

    #[error("expected surface to have at least {expected} bytes but found {actual}")]
    NotEnoughData { expected: usize, actual: usize },

    #[error("encoding data to format {format:?} is not supported")]
    UnsupportedEncodeFormat { format: ImageFormat },

    #[error("mipmap count {mipmaps} exceeds the maximum value of {max_total_mipmaps}")]
    InvalidMipmapCount {
        mipmaps: u32,
        height: u32,
        max_total_mipmaps: u32,
    },

    #[error("failed to get image data for layer {layer} mipmap {mipmap}")]
    MipmapDataOutOfBounds { layer: u32, mipmap: u32 },

    #[error("DDS image format {0:?} is not supported")]
    UnsupportedDdsFormat(DdsFormatInfo),

    #[error("{mipmaps} mipmaps exceeds the maximum expected mipmap count of {max_mipmaps}")]
    UnexpectedMipmapCount { mipmaps: u32, max_mipmaps: u32 },
}
