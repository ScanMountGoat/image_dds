# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## unreleased
### Added
* Added support for `Rgba8Snorm`.
* Added support for `R16Unorm`, `R16Snorm`, `Rg16Unorm`, `Rg16Snorm`, `Rgba16Unorm`, and `Rgba16Snorm`.
* Added support for `R16Float`, `Rg16Float`, `R32Float`, `Rg32Float`, and `Rgb32Float`.

### Changed
* Improved accuracy for encoding to `Bgra4Unorm`.

## 0.7.1 - 2025-01-28
### Added
* Added `Surface::as_ref`, `SurfaceRgba8::as_ref` and `SurfaceRgba32Float::as_ref` for converting to surfaces with borrowed data.
* Added derives for `Clone` and `Copy` to `Surface`, `SurfaceRgba8`, and `SurfaceRgba32Float`.

## 0.7.0 - 2025-01-10
### Added
* Added `SurfaceRgba8::get_image` and `SurfaceRgba32Float::get_image` for more conveniently accessing mipmap data.
* Added `resize_dds` example to show some more advanced usage of surface decoding and encoding.
* Added support for `D3DFormat::R8G8B8`, `D3DFormat::A8B8G8R8`, `D3DFormat::A16B16G16R16F`, and `D3DFormat::A32B32G32R32F`.
* Added support for encoding BC2 compressed surfaces.
* Defined `SurfaceRgba8::to_image` and `SurfaceRgba32Float::to_image` for `AsRef<[T]>` instead of just `Vec<T>`.
* Added support for `R8Snorm`, `Rg8Unorm`, and `Rg8Snorm` formats.

### Changed
* Improved accuracy of `u8` and `f32` decoding for BC4 and BC5.

### Fixed
* Fixed an issue where the result of `Surface::from_dds` would contain `0` layers instead of the expected value of `1`.

## 0.6.2 - 2024-11-23
### Fixed
* Fixed a compile error when building without the `ddsfile` feature flag.

## 0.6.1 - 2024-10-28
### Fixed
* Fixed an issue where mipmap generation did not work properly for 3D textures.

## 0.6.0 - 2024-07-29
### Changed
* Changed the rounding behavior of BC1, BC2, and BC3 decoding to be more precise, which more closely matches the floating point arithmetic used in DirectXTex. Differences in pixel RGB values compared to the previous decoder will be at most 1.

## 0.5.1 - 2024-04-15
### Fixed
* Fixed an issue where `dds_from_imagef32` would panic due to internal alignment mismatches when encoding to `ImageFormat::Rgba16Float` and `ImageFormat::Rgba32Float`.

## 0.5.0 - 2024-03-01
### Changed
* Renamed `ImageFormat` variants to be more descriptive.
* Changed BCN decoding implementation to bcdec_rs for better safety and easier compilation.

### Removed
* Removed the `"decode"` feature from image_dds. Decoding is now implemented in pure Rust and always enabled.

## 0.4.0 - 2023-12-23
### Added
* Added support for `D3DFormat::A8R8G8B8`.

### Changed
* Marked `ImageFormat` as `#[non_exhaustive]` to limit future breaking changes.
* Adjusted DDS unsupported format error to include relevant DDS format type information.

## 0.3.0 - 2023-11-22
### Added
* Added `Surface::decode_layers_mipmaps_rgba8` and `Surface::decode_layers_mipmaps_rgbaf32`.
* Added support for the `B4G4R4A4Unorm` format.

### Changed
* Improved performance for `image_from_dds` by reducing copies.

## 0.2.0 - 2023-10-29
### Added
* Added optional serde support for enums and surfaces.
* Added optional strum support for enums.
* Added encode/decode support for the `R16G16B16A16Float` format.
* Added support for HDR floating point images like EXR.
* Added support for creating a 3D surface from a vertically stacked image.
* Added support for creating a cube map surface from a vertically stacked image.

### Changed
* Relaxed validation and implemented padding to allow encoding BCN surfaces with non integral dimensions in blocks.
* Combined `CompressSurfaceError` and `DecompressSurfaceError` into `SurfaceError`.
* Changed surface conversion functions to be methods instead of functions to support chaining.

## 0.1.1 - 2023-03-21
### Fixed
* Fixed an issue decoding images with non integral dimensions in blocks.

## 0.1.0 - 2023-03-13
* First public release!