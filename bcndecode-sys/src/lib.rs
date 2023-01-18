use std::ffi::c_int;

extern "C" {
    // TODO: Pointer alignment?
    // TODO: compressed_block is cast to (unsigned long long*) or (unsigned short*).
    pub fn bcdec_bc1(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
    pub fn bcdec_bc2(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
    pub fn bcdec_bc3(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
    pub fn bcdec_bc4(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
    pub fn bcdec_bc5(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
    pub fn bcdec_bc6h_float(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
        is_signed: c_int,
    );
    pub fn bcdec_bc6h_half(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
        is_signed: c_int,
    );
    pub fn bcdec_bc7(
        compressed_block: *const u8,
        decompressed_block: *mut u8,
        destination_pitch: c_int,
    );
}

// TODO: this will be tested thoroughly by image_dds, so these tests can be deleted later.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Set the pitch for tightly packed inputs and outputs.
        unsafe {
            bcdec_bc1(
                (&[0u8; 8]).as_ptr(),
                (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(),
                4,
            );
            bcdec_bc2((&[0u8]).as_ptr(), (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(), 4);
            bcdec_bc3((&[0u8]).as_ptr(), (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(), 4);
            bcdec_bc4((&[0u8]).as_ptr(), (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(), 4);
            bcdec_bc5((&[0u8]).as_ptr(), (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(), 4);
            bcdec_bc6h_float(
                (&[0u8]).as_ptr(),
                (&mut [0u8; 4 * 4 * 12]).as_mut_ptr(),
                4,
                1,
            );
            bcdec_bc6h_half(
                (&[0u8]).as_ptr(),
                (&mut [0u8; 4 * 4 * 8]).as_mut_ptr(),
                4,
                1,
            );
            bcdec_bc7((&[0u8]).as_ptr(), (&mut [0u8; 4 * 4 * 4]).as_mut_ptr(), 4);
        }
    }
}
