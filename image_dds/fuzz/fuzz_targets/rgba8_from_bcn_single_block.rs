#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (image_dds::ImageFormat, &[u8])| {
    let (format, data) = input;

    // The largest BCN compressed block is 16 bytes.
    // Each format uses 4x4 pixel blocks.
    if data.len() >= 16 {
        let _bytes = image_dds::bcn::rgba8_from_bcn(4, 4, data, format.into());
    }
});
