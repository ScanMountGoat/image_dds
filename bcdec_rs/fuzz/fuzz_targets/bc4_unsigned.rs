#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
    let mut expected = [255u8; 4 * 4 * 1];
    unsafe {
        bcndecode_sys::bcdec_bc4(data.as_ptr(), expected.as_mut_ptr() as _, 4 * 1, 0);
    }

    let mut actual = [255u8; 4 * 4 * 1];
    bcdec_rs::bc4(&data, &mut actual, 4);

    assert_eq!(expected, actual);
});
