#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // 4x4 RGB f16
    // Start with non zeros to test zero filling.
    let mut expected = [255u8; 4 * 4 * 6];
    unsafe {
        // The pitch is in terms of half floats rather than bytes.
        bcndecode_sys::bcdec_bc6h_half(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 3, 1);
    }

    let mut actual = [255u8; 4 * 4 * 6];
    bcdec_rs::bc6h_half(&data, &mut actual, 4 * 3, true);

    assert_eq!(expected, actual);
});
