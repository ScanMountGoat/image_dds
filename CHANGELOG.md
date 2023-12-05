# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### unreleased
* Marked `ImageFormat` as `#[non_exhaustive]` to limit future breaking changes.

## 0.3.0 - 2023-11-22
### Added
* Added `Surface::decode_layers_mipmaps_rgba8` and `Surface::decode_layers_mipmaps_rgbaf32`.
* Added support for the `B4G4R4A4Unorm` format.
* Added support for `D3DFormat::A8R8G8B8`.

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