#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u32, u32, image_dds::ImageFormat, &[u8])| {
    let (width, height, format, data) = input;
    let _bytes = image_dds::bcn::rgba8_from_bcn(width, height, data, format.into());
});
