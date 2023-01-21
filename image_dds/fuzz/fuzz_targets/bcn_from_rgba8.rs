#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: (u32, u32, image_dds::ImageFormat, &[u8])| {
    let (width, height, format, data) = input;
    // TODO: Is it worth testing different quality settings?
    // Use fast for now so each iteration is faster.
    let quality = image_dds::Quality::Fast;
    let _bytes = image_dds::bcn::bcn_from_rgba8(width, height, data, format.into(), quality);
});
