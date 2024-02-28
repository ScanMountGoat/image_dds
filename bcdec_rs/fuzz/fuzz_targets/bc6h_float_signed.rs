#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // 4x4 RGB f32
    // Start with non zeros to test zero filling.
    let mut expected = [f32::NAN; 4 * 4 * 3];
    unsafe {
        // The pitch is in terms of floats rather than bytes.
        bcndecode_sys::bcdec_bc6h_float(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 3, 1);
    }

    let mut actual = [f32::NAN; 4 * 4 * 3];
    bcdec_rs::bc6h_float(&data, &mut actual, 4 * 3, true);

    assert_eq!(expected, actual);
});
