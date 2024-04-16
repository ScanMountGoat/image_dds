#![no_main]

use libfuzzer_sys::fuzz_target;

type Input = (
    image_dds::SurfaceRgba32Float<Vec<f32>>,
    image_dds::ImageFormat,
    image_dds::Quality,
    image_dds::Mipmaps,
);

fuzz_target!(|input: Input| {
    let (surface, format, quality, mipmaps) = input;
    let _result = surface.encode(format, quality, mipmaps);
});
