use std::ffi::c_int;

extern "C" {
    // TODO: Pointer alignment?
    // TODO: compressed_block is cast to (unsigned long long*) or (unsigned short*).
    // TODO: Use the type cast internally rather than *mut u8
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
