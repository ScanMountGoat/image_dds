#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // BC4 stores grayscale data, so each decompressed pixel is 1 float.
    // Start with non zeros to test zero filling.
    let mut expected = [f32::MAX; 4 * 4 * 1];
    unsafe {
        // The pitch is in terms of floats rather than bytes.
        bcndecode_sys::bcdec_bc4_float(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 1, 1);
    }

    let mut actual = [f32::MAX; 4 * 4 * 1];
    bcdec_rs::bc4_float(&data, &mut actual, 4, true);

    assert_eq!(expected, actual);
});
