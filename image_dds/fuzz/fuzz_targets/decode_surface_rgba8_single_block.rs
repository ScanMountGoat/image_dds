#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (image_dds::ImageFormat, &[u8])| {
    let (image_format, data) = input;

    // The largest BCN compressed block is 16 bytes.
    // Each format uses 4x4 pixel blocks.
    let surface = image_dds::Surface {
        width: 4,
        height: 4,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format,
        data,
    };

    let _result = surface.decode_rgba8();
});
