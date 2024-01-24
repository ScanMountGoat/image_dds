#![no_main]

extern crate libfuzzer_sys;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 16]| {
    // 4x4 RGBA u8
    let mut expected = [0u8; 4 * 4 * 4];
    unsafe {
        bcndecode_sys::bcdec_bc2(data.as_ptr(), expected.as_mut_ptr() as _, 4*4);
    }

    let mut actual = [0u8; 4*4 * 4];
    bcdec_rs::bc2(&data, &mut actual, 4*4);

    assert_eq!(expected, actual);
});
