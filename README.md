# image_dds

[![Latest Version](https://img.shields.io/crates/v/image_dds.svg)](https://crates.io/crates/image_dds) [![docs.rs](https://docs.rs/image_dds/badge.svg)](https://docs.rs/image_dds)

A library for converting uncompressed image data to and from compressed formats.

## Examples
The provided example projects demonstrate basic usage of the conversion functions. 

The library also provides functions for working directly with the raw bytes of a surface instead of a dds or image file. Floating point data like EXR files or BC6 DDS files is also supported.

`cargo run --release --example img2dds image.png out.dds BC3RgbaUnorm`  
`cargo run --release --example dds2img out.dds out.tiff`  

`cargo run --release --example img2ddsf32 image.exr out.dds`  
`cargo run --release --example dds2imgf32 out.dds out.exr`  

Array layers and depth slices in images should be stacked vertically. 
This enables surface creation to avoid making additional copies since the RGBA data is already in the desired layout.

`cargo run --release --example img2dds 3d.dds 3d.png`  
`cargo run --release --example img2dds 3d.png out.dds Rgba8Unorm depth`  

`cargo run --release --example dds2imgf32 cube.dds cube.exr`  
`cargo run --release --example img2ddsf32 cube.exr out.dds BC6hRgbUfloat layers`  

## Supported Formats
The only compressed formats supported at this time are BCN formats since these are the formats commonly used by DDS files and compressed GPU textures. This library does not support other compressed formats used for GPU textures like ETC1. Compression is handled using [intel-tex-rs-2](https://github.com/Traverse-Research/intel-tex-rs-2) for bindings to Intel's ISPC texture compressor in C++. Decompression is handled using a safe Rust port of the [bcdec](https://github.com/iOrange/bcdec) library in C.

Some uncompressed formats are also supported. These formats are supported by DDS but are rarely used with DDS files in practice. Uncompressed formats are often used for small textures or textures used for window surfaces and UI elements.

See the [documentation](https://docs.rs/image_dds/latest/image_dds/enum.ImageFormat.html) for all supported formats.

## Features
Helper functions for working with the files from the [image](https://crates.io/crates/image) and [ddsfile](https://crates.io/crates/ddsfile) crates are supported under feature flags and enabled by default. The `encoding` feature is enabled by default but can be disabled to resolve compilation issues on certain targets if not needed. The default features of the image crate are disabled by default. Features are additive, so simply add a reference to the appropriate version of image in the `Cargo.toml` to enable all the default features.

## Building
Build the projects using `cargo build --release` with a newer version of the Rust toolchain installed. Builds support Windows, Linux, and MacOS. Some targets may not build properly due to a lack of precompiled ISP kernels in intel-tex-rs-2.
