#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // 4x4 RGBA f32
    // Start with non zeros to test zero filling.
    let mut expected = [f32::MAX; 4 * 4 * 4];
    unsafe {
        // The pitch is in terms of floats rather than bytes.
        bcndecode_sys::bcdec_bc5_float(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 4, 0);
    }

    let mut actual = [f32::MAX; 4 * 4 * 4];
    bcdec_rs::bc5_float(&data, &mut actual, 4 * 4, false);

    assert_eq!(expected, actual);
});
