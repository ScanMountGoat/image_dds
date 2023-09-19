#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|surface: image_dds::Surface<Vec<u8>>| {
    let _result = surface.decode_rgbaf32();
});
