# image_dds
A library for converting uncompressed image data to and from compressed formats.

## Examples
The provided example projects demonstrate basic usage of the conversion functions. 
The library also provides functions for working directly with the raw bytes of a surface instead of a dds or image file.

`cargo run --release --example img2dds image.png out.dds bc3`  
`cargo run --release --example dds2img out.dds out.tiff`  

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
Build the projects using `cargo build --release` with a newer version of the Rust toolchain installed. Builds support Windows, Linux, and MacOS. Some targets may not build properly due to a lack of precompiled ISP kernels in intel-tex-rs-2.