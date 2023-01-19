use bcn::{CompressSurfaceError, DecompressSurfaceError};
use ddsfile::DxgiFormat;

// TODO: Module level documentation explaining limitations and showing basic usage.

// TODO: pub use some of the functions?
pub mod bcn;

/// The conversion quality when converting to compressed formats.
///
/// Higher quality settings run significantly slower.
/// Block compressed formats like BC7 use a fixed compression ratio,
/// so lower quality settings do not use less space than slower ones.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    /// Faster exports with slightly lower quality.
    Fast,
    /// Normal export speed and quality.
    Normal,
    /// Slower exports for slightly higher quality.
    Slow,
}

// TODO: Move this into the BCN module instead?
// TODO: Document that not all DDS formats are supported.
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CompressionFormat {
    Bc1,
    Bc2,
    Bc3,
    Bc4,
    Bc5,
    Bc6,
    Bc7,
}

// TODO: Put this behind a feature flag.
impl TryFrom<DxgiFormat> for CompressionFormat {
    type Error = String;

    fn try_from(value: DxgiFormat) -> Result<Self, Self::Error> {
        match value {
            DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB => Ok(CompressionFormat::Bc1),
            DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB => Ok(CompressionFormat::Bc2),
            DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB => Ok(CompressionFormat::Bc3),
            DxgiFormat::BC4_UNorm | DxgiFormat::BC4_SNorm => Ok(CompressionFormat::Bc4), // TODO: signed variants?
            DxgiFormat::BC5_UNorm | DxgiFormat::BC5_SNorm => Ok(CompressionFormat::Bc5), // TODO: signed variants?
            DxgiFormat::BC6H_SF16 | DxgiFormat::BC6H_UF16 => Ok(CompressionFormat::Bc6), // TODO: signed variants?
            DxgiFormat::BC7_UNorm | DxgiFormat::BC7_UNorm_sRGB => Ok(CompressionFormat::Bc7),
            _ => Err(format!("Unsupported format {value:?}")),
        }
    }
}

impl From<CompressionFormat> for DxgiFormat {
    fn from(value: CompressionFormat) -> Self {
        // TODO: Differentiate between unorm and srgb.
        // TODO: Differentiate between signed and unsigned.
        match value {
            CompressionFormat::Bc1 => Self::BC1_UNorm,
            CompressionFormat::Bc2 => Self::BC2_UNorm,
            CompressionFormat::Bc3 => Self::BC3_UNorm,
            CompressionFormat::Bc4 => Self::BC4_UNorm,
            CompressionFormat::Bc5 => Self::BC5_UNorm,
            CompressionFormat::Bc6 => Self::BC6H_UF16,
            CompressionFormat::Bc7 => Self::BC7_UNorm,
        }
    }
}

pub fn dds_from_image(
    image: &image::RgbaImage,
    format: CompressionFormat,
    quality: Quality,
) -> Result<ddsfile::Dds, CompressSurfaceError> {
    // TODO: Mipmaps, depth, and array layers?
    let mut dds = ddsfile::Dds::new_dxgi(ddsfile::NewDxgiParams {
        height: image.height(),
        width: image.width(),
        depth: None,
        format: format.into(),
        mipmap_levels: None,
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: ddsfile::D3D10ResourceDimension::Texture2D,
        alpha_mode: ddsfile::AlphaMode::Straight, // TODO: Does this matter?
    })
    .unwrap();

    dds.data = bcn::bcn_from_rgba8(
        image.width(),
        image.height(),
        image.as_raw(),
        format,
        quality,
    )?;

    Ok(dds)
}

pub fn image_from_dds(dds: &ddsfile::Dds) -> Result<image::RgbaImage, DecompressSurfaceError> {
    // TODO: Mipmaps, depth, and array layers?
    let rgba = bcn::rgba8_from_bcn(
        dds.get_width(),
        dds.get_height(),
        &dds.data,
        dds.get_dxgi_format().unwrap().try_into().unwrap(),
    )?;

    // TODO: Avoid unwrap.
    let image = image::RgbaImage::from_raw(dds.get_width(), dds.get_height(), rgba).unwrap();

    Ok(image)
}

fn div_round_up(x: usize, d: usize) -> usize {
    (x + d - 1) / d
}
