# image_dds
A library for converting uncompressed image data to and from compressed formats.

This facilitates creating tooling for working with compressed GPU textures. Compressed BC1 or BC3 textures can be converted to uncompressed RGBA8 data to work with devices or contexts that do not support compressed textures like the browser. Uncompressed RGBA8 image data can be compressed to create DDS files or save space for GPU textures on supported hardware.

## Supported Formats
Currently only BCN formats are supported at this time since these are the formats commonly used by DDS files and compressed GPU textures. This library current does not support other compressed formats used for GPU textures like ETC1.  Compression is handled using [intel-tex-rs-2](https://github.com/Traverse-Research/intel-tex-rs-2) for bindings to Intel's ISPC texture compressor in C++. Decompression is handled using bindings to the [bcdec](https://github.com/iOrange/bcdec) library in C.

| Format | Compress | Decompress |
| --- | --- | --- |
| BC1 | :heavy_check_mark: | :heavy_check_mark: |
| BC2 | :x: | :heavy_check_mark: |
| BC3 | :heavy_check_mark: | :heavy_check_mark: |
| BC4 | :heavy_check_mark: | :heavy_check_mark: |
| BC5 | :heavy_check_mark: | :heavy_check_mark: |
| BC6 | :heavy_check_mark: | :heavy_check_mark: |
| BC7 | :heavy_check_mark: | :heavy_check_mark: |

## Features
Helper functions for working with the files from the [image](https://crates.io/crates/image) and [ddsfile](https://crates.io/crates/ddsfile) crates are supported under feature flags and enabled by default.

## Building
Build the projects using `cargo build --release` with a newer version of the Rust toolchain installed. Builds support Windows, Linux, and MacOS. MacOS arm64 builds currently don't work due to configuration settings in intel-tex-rs-2.