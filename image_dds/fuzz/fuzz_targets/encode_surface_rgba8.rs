#![no_main]

use libfuzzer_sys::fuzz_target;

type Input = (
    image_dds::SurfaceRgba8<Vec<u8>>,
    image_dds::ImageFormat,
    image_dds::Quality,
    image_dds::Mipmaps,
);

fuzz_target!(|input: Input| {
    let (surface, format, quality, mipmaps) = input;
    let _result = image_dds::encode_surface_rgba8(surface, format, quality, mipmaps);
});
