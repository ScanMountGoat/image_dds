#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // 4x4 RGB f32
    // Start with non zeros to test zero filling.
    let mut expected = [255u8; 4 * 4 * 12];
    unsafe {
        // The pitch is in terms of floats rather than bytes.
        bcndecode_sys::bcdec_bc6h_float(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 3, 0);
    }

    let mut actual = [255u8; 4 * 4 * 12];
    bcdec_rs::bc6h_float(&data, &mut actual, 4 * 3, false);

    assert_eq!(expected, actual);
});
