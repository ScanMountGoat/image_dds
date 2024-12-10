#![no_std]
//! A safe, no_std, pure Rust port of [bcdec](https://github.com/iOrange/bcdec).

// A mostly 1:1 translation of the code and comments found here:
// https://github.com/iOrange/bcdec/blob/main/bcdec.h
// Names are shortened and pointer arithmetic is converted to more idiomatic Rust.

// Used information sources:
// https://docs.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression
// https://docs.microsoft.com/en-us/windows/win32/direct3d11/bc6h-format
// https://docs.microsoft.com/en-us/windows/win32/direct3d11/bc7-format
// https://docs.microsoft.com/en-us/windows/win32/direct3d11/bc7-format-mode-reference
//
// ! WARNING ! Khronos's BPTC partitions tables contain mistakes, do not use them!
// https://www.khronos.org/registry/DataFormat/specs/1.1/dataformat.1.1.html#BPTC
//
// ! Use tables from here instead !
// https://www.khronos.org/registry/OpenGL/extensions/ARB/ARB_texture_compression_bptc.txt
//
// Leaving it here as it's a nice read
// https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
//
// Fast half to float function from here
// https://gist.github.com/rygorous/2144712

/// Decode 8 bytes from `compressed_block` to RGBA8
/// with `destination_pitch` many bytes  per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 8];
/// let mut decompressed_block = [0u8; 4 * 4 * 4];
/// bcdec_rs::bc1(&compressed_block, &mut decompressed_block, 4 * 4);
/// ```
pub fn bc1(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    color_block(
        compressed_block,
        decompressed_block,
        destination_pitch,
        false,
    )
}

/// Decode 16 bytes from `compressed_block` to RGBA8
/// with `destination_pitch` many bytes per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0u8; 4 * 4 * 4];
/// bcdec_rs::bc2(&compressed_block, &mut decompressed_block, 4 * 4);
/// ```
pub fn bc2(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    color_block(
        &compressed_block[8..],
        decompressed_block,
        destination_pitch,
        true,
    );
    sharp_alpha_block(compressed_block, decompressed_block, destination_pitch);
}

/// Decode 16 bytes from `compressed_block` to RGBA8
/// with `destination_pitch` many bytes per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0u8; 4 * 4 * 4];
/// bcdec_rs::bc3(&compressed_block, &mut decompressed_block, 4 * 4);
/// ```
pub fn bc3(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    color_block(
        &compressed_block[8..],
        decompressed_block,
        destination_pitch,
        true,
    );
    smooth_alpha_block(
        compressed_block,
        &mut decompressed_block[3..],
        destination_pitch,
        4,
    );
}

/// Decode 8 bytes from `compressed_block` to R8
/// with `destination_pitch` many bytes per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 8];
/// let mut decompressed_block = [0u8; 4 * 4];
/// bcdec_rs::bc4(&compressed_block, &mut decompressed_block, 4, false);
/// ```
pub fn bc4(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    is_signed: bool,
) {
    if is_signed {
        smooth_alpha_block_signed(compressed_block, decompressed_block, destination_pitch, 1);
    } else {
        smooth_alpha_block(compressed_block, decompressed_block, destination_pitch, 1);
    }
}

/// Decode 16 bytes from `compressed_block` to RG8
/// with `destination_pitch` many bytes per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0u8; 4 * 4 * 2];
/// bcdec_rs::bc5(&compressed_block, &mut decompressed_block, 4 * 2, false);
/// ```
pub fn bc5(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    is_signed: bool,
) {
    if is_signed {
        smooth_alpha_block_signed(compressed_block, decompressed_block, destination_pitch, 2);
        smooth_alpha_block_signed(
            &compressed_block[8..],
            &mut decompressed_block[1..],
            destination_pitch,
            2,
        );
    } else {
        smooth_alpha_block(compressed_block, decompressed_block, destination_pitch, 2);
        smooth_alpha_block(
            &compressed_block[8..],
            &mut decompressed_block[1..],
            destination_pitch,
            2,
        );
    }
}

/// Decode 16 bytes from `compressed_block` to RGBFloat16
/// with `destination_pitch` many half floats per output row.
///
/// The `u16` values contain the bits of a half-precision floats.
/// For a crate for working with these values, see [half](https://crates.io/crates/half).
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0u16; 4 * 4 * 3];
/// bcdec_rs::bc6h_half(&compressed_block, &mut decompressed_block, 4 * 3, false);
/// ```
pub fn bc6h_half(
    compressed_block: &[u8],
    decompressed_block: &mut [u16],
    destination_pitch: usize,
    is_signed: bool,
) {
    static ACTUAL_BITS_COUNT: [[u8; 14]; 4] = [
        [10, 7, 11, 11, 11, 9, 8, 8, 8, 6, 10, 11, 12, 16], //  W
        [5, 6, 5, 4, 4, 5, 6, 5, 5, 6, 10, 9, 8, 4],        // dR
        [5, 6, 4, 5, 4, 5, 5, 6, 5, 6, 10, 9, 8, 4],        // dG
        [5, 6, 4, 4, 5, 5, 5, 5, 6, 6, 10, 9, 8, 4],        // dB
    ];

    // There are 32 possible partition sets for a two-region tile.
    // Each 4x4 block represents a single shape.
    // Here also every fix-up index has MSB bit set.
    static PARTITION_SETS: [[[u8; 4]; 4]; 32] = [
        [[128, 0, 1, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 0, 1, 129]], //  0
        [[128, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 129]], //  1
        [[128, 1, 1, 1], [0, 1, 1, 1], [0, 1, 1, 1], [0, 1, 1, 129]], //  2
        [[128, 0, 0, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 1, 1, 129]], //  3
        [[128, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 1, 129]], //  4
        [[128, 0, 1, 1], [0, 1, 1, 1], [0, 1, 1, 1], [1, 1, 1, 129]], //  5
        [[128, 0, 0, 1], [0, 0, 1, 1], [0, 1, 1, 1], [1, 1, 1, 129]], //  6
        [[128, 0, 0, 0], [0, 0, 0, 1], [0, 0, 1, 1], [0, 1, 1, 129]], //  7
        [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 1, 129]], //  8
        [[128, 0, 1, 1], [0, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], //  9
        [[128, 0, 0, 0], [0, 0, 0, 1], [0, 1, 1, 1], [1, 1, 1, 129]], // 10
        [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 1], [0, 1, 1, 129]], // 11
        [[128, 0, 0, 1], [0, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], // 12
        [[128, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 1], [1, 1, 1, 129]], // 13
        [[128, 0, 0, 0], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], // 14
        [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 129]], // 15
        [[128, 0, 0, 0], [1, 0, 0, 0], [1, 1, 1, 0], [1, 1, 1, 129]], // 16
        [[128, 1, 129, 1], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]], // 17
        [[128, 0, 0, 0], [0, 0, 0, 0], [129, 0, 0, 0], [1, 1, 1, 0]], // 18
        [[128, 1, 129, 1], [0, 0, 1, 1], [0, 0, 0, 1], [0, 0, 0, 0]], // 19
        [[128, 0, 129, 1], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]], // 20
        [[128, 0, 0, 0], [1, 0, 0, 0], [129, 1, 0, 0], [1, 1, 1, 0]], // 21
        [[128, 0, 0, 0], [0, 0, 0, 0], [129, 0, 0, 0], [1, 1, 0, 0]], // 22
        [[128, 1, 1, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 0, 0, 129]], // 23
        [[128, 0, 129, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 0]], // 24
        [[128, 0, 0, 0], [1, 0, 0, 0], [129, 0, 0, 0], [1, 1, 0, 0]], // 25
        [[128, 1, 129, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 1, 1, 0]], // 26
        [[128, 0, 129, 1], [0, 1, 1, 0], [0, 1, 1, 0], [1, 1, 0, 0]], // 27
        [[128, 0, 0, 1], [0, 1, 1, 1], [129, 1, 1, 0], [1, 0, 0, 0]], // 28
        [[128, 0, 0, 0], [1, 1, 1, 1], [129, 1, 1, 1], [0, 0, 0, 0]], // 29
        [[128, 1, 129, 1], [0, 0, 0, 1], [1, 0, 0, 0], [1, 1, 1, 0]], // 30
        [[128, 0, 129, 1], [1, 0, 0, 1], [1, 0, 0, 1], [1, 1, 0, 0]], // 31
    ];

    let a_weight3 = [0, 9, 18, 27, 37, 46, 55, 64];
    let a_weight4 = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

    let mut bstream = Bitstream {
        low: u64::from_le_bytes(compressed_block[0..8].try_into().unwrap()),
        high: u64::from_le_bytes(compressed_block[8..16].try_into().unwrap()),
    };

    let mut r = [0i32; 4]; // wxyz
    let mut g = [0i32; 4]; // wxyz
    let mut b = [0i32; 4]; // wxyz

    let mut mode = bstream.read_bits(2);
    if mode > 1 {
        mode |= bstream.read_bits(3) << 2;
    }

    // modes >= 11 (10 in my code) are using 0 one, others will read it from the bitstream
    let mut partition = 0;

    match mode {
        // mode 1
        0b00 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 75 bits (10.555, 10.555, 10.555)
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(5); // rx[4:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(5); // gx[4:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(5); // bx[4:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(5); // ry[4:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(5); // rz[4:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 0;
        }

        // mode 2
        0b01 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 75 bits (7666, 7666, 7666)
            g[2] |= bstream.read_bit_i32() << 5; // gy[5]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[3] |= bstream.read_bit_i32() << 5; // gz[5]
            r[0] |= bstream.read_bits_i32(7); // rw[6:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(7); // gw[6:0]
            b[2] |= bstream.read_bit_i32() << 5; // by[5]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(7); // bw[6:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            b[3] |= bstream.read_bit_i32() << 5; // bz[5]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(6); // rx[5:0]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(6); // gx[5:0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(6); // bx[5:0]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(6); // ry[5:0]
            r[3] |= bstream.read_bits_i32(6); // rz[5:0]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 1;
        }

        // mode 3
        0b00010 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (11.555, 11.444, 11.444)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(5); // rx[4:0]
            r[0] |= bstream.read_bit_i32() << 10; // rw[10]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(4); // gx[3:0]
            g[0] |= bstream.read_bit_i32() << 10; // gw[10]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(4); // bx[3:0]
            b[0] |= bstream.read_bit_i32() << 10; // bw[10]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(5); // ry[4:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(5); // rz[4:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 2;
        }
        // mode 4
        0b00110 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (11.444, 11.555, 11.444)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(4); // rx[3:0]
            r[0] |= bstream.read_bit_i32() << 10; // rw[10]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(5); // gx[4:0]
            g[0] |= bstream.read_bit_i32() << 10; // gw[10]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(4); // bx[3:0]
            b[0] |= bstream.read_bit_i32() << 10; // bw[10]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(4); // ry[3:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(4); // rz[3:0]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 3;
        }
        // mode 5
        0b01010 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (11.444, 11.444, 11.555)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(4); // rx[3:0]
            r[0] |= bstream.read_bit_i32() << 10; // rw[10]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(4); // gx[3:0]
            g[0] |= bstream.read_bit_i32() << 10; // gw[10]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(5); // bx[4:0]
            b[0] |= bstream.read_bit_i32() << 10; // bw[10]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(4); // ry[3:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(4); // rz[3:0]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 4;
        }
        // mode 6
        0b01110 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (9555, 9555, 9555)
            r[0] |= bstream.read_bits_i32(9); // rw[8:0]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(9); // gw[8:0]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(9); // bw[8:0]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(5); // rx[4:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(5); // gx[4:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gx[3:0]
            b[1] |= bstream.read_bits_i32(5); // bx[4:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(5); // ry[4:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(5); // rz[4:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 5;
        }
        // mode 7
        0b10010 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (8666, 8555, 8555)
            r[0] |= bstream.read_bits_i32(8); // rw[7:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(8); // gw[7:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(8); // bw[7:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(6); // rx[5:0]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(5); // gx[4:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(5); // bx[4:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(6); // ry[5:0]
            r[3] |= bstream.read_bits_i32(6); // rz[5:0]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 6;
        }
        // mode 8
        0b10110 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (8555, 8666, 8555)
            r[0] |= bstream.read_bits_i32(8); // rw[7:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(8); // gw[7:0]
            g[2] |= bstream.read_bit_i32() << 5; // gy[5]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(8); // bw[7:0]
            g[3] |= bstream.read_bit_i32() << 5; // gz[5]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(5); // rx[4:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(6); // gx[5:0]
            g[3] |= bstream.read_bits_i32(4); // zx[3:0]
            b[1] |= bstream.read_bits_i32(5); // bx[4:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(5); // ry[4:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(5); // rz[4:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 7;
        }
        // mode 9
        0b11010 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (8555, 8555, 8666)
            r[0] |= bstream.read_bits_i32(8); // rw[7:0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(8); // gw[7:0]
            b[2] |= bstream.read_bit_i32() << 5; // by[5]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(8); // bw[7:0]
            b[3] |= bstream.read_bit_i32() << 5; // bz[5]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(5); // bw[4:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(5); // gx[4:0]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(6); // bx[5:0]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(5); // ry[4:0]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            r[3] |= bstream.read_bits_i32(5); // rz[4:0]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 8;
        }
        // mode 10
        0b11110 => {
            // Partitition indices: 46 bits
            // Partition: 5 bits
            // Color Endpoints: 72 bits (6666, 6666, 6666)
            r[0] |= bstream.read_bits_i32(6); // rw[5:0]
            g[3] |= bstream.read_bit_i32() << 4; // gz[4]
            b[3] |= bstream.read_bit_i32(); // bz[0]
            b[3] |= bstream.read_bit_i32() << 1; // bz[1]
            b[2] |= bstream.read_bit_i32() << 4; // by[4]
            g[0] |= bstream.read_bits_i32(6); // gw[5:0]
            g[2] |= bstream.read_bit_i32() << 5; // gy[5]
            b[2] |= bstream.read_bit_i32() << 5; // by[5]
            b[3] |= bstream.read_bit_i32() << 2; // bz[2]
            g[2] |= bstream.read_bit_i32() << 4; // gy[4]
            b[0] |= bstream.read_bits_i32(6); // bw[5:0]
            g[3] |= bstream.read_bit_i32() << 5; // gz[5]
            b[3] |= bstream.read_bit_i32() << 3; // bz[3]
            b[3] |= bstream.read_bit_i32() << 5; // bz[5]
            b[3] |= bstream.read_bit_i32() << 4; // bz[4]
            r[1] |= bstream.read_bits_i32(6); // rx[5:0]
            g[2] |= bstream.read_bits_i32(4); // gy[3:0]
            g[1] |= bstream.read_bits_i32(6); // gx[5:0]
            g[3] |= bstream.read_bits_i32(4); // gz[3:0]
            b[1] |= bstream.read_bits_i32(6); // bx[5:0]
            b[2] |= bstream.read_bits_i32(4); // by[3:0]
            r[2] |= bstream.read_bits_i32(6); // ry[5:0]
            r[3] |= bstream.read_bits_i32(6); // rz[5:0]
            partition = bstream.read_bits_i32(5); // d[4:0]
            mode = 9;
        }
        // mode 11
        0b00011 => {
            // Partitition indices: 63 bits
            // Partition: 0 bits
            // Color Endpoints: 60 bits (10.10, 10.10, 10.10)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(10); // rx[9:0]
            g[1] |= bstream.read_bits_i32(10); // gx[9:0]
            b[1] |= bstream.read_bits_i32(10); // bx[9:0]
            mode = 10;
        }
        // mode 12
        0b00111 => {
            // Partitition indices: 63 bits
            // Partition: 0 bits
            // Color Endpoints: 60 bits (11.9, 11.9, 11.9)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(9); // rx[8:0]
            r[0] |= bstream.read_bit_i32() << 10; // rw[10]
            g[1] |= bstream.read_bits_i32(9); // gx[8:0]
            g[0] |= bstream.read_bit_i32() << 10; // gw[10]
            b[1] |= bstream.read_bits_i32(9); // bx[8:0]
            b[0] |= bstream.read_bit_i32() << 10; // bw[10]
            mode = 11;
        }
        // mode 13
        0b01011 => {
            // Partitition indices: 63 bits
            // Partition: 0 bits
            // Color Endpoints: 60 bits (12.8, 12.8, 12.8)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(8); // rx[7:0]
            r[0] |= bstream.read_bits_r(2) << 10; // rx[10:11]
            g[1] |= bstream.read_bits_i32(8); // gx[7:0]
            g[0] |= bstream.read_bits_r(2) << 10; // gx[10:11]
            b[1] |= bstream.read_bits_i32(8); // bx[7:0]
            b[0] |= bstream.read_bits_r(2) << 10; // bx[10:11]
            mode = 12;
        }
        // mode 14
        0b01111 => {
            // Partitition indices: 63 bits
            // Partition: 0 bits
            // Color Endpoints: 60 bits (16.4, 16.4, 16.4)
            r[0] |= bstream.read_bits_i32(10); // rw[9:0]
            g[0] |= bstream.read_bits_i32(10); // gw[9:0]
            b[0] |= bstream.read_bits_i32(10); // bw[9:0]
            r[1] |= bstream.read_bits_i32(4); // rx[3:0]
            r[0] |= bstream.read_bits_r(6) << 10; // rw[10:15]
            g[1] |= bstream.read_bits_i32(4); // gx[3:0]
            g[0] |= bstream.read_bits_r(6) << 10; // gw[10:15]
            b[1] |= bstream.read_bits_i32(4); // bx[3:0]
            b[0] |= bstream.read_bits_r(6) << 10; // bw[10:15]
            mode = 13;
        }
        _ => {
            // Modes 10011, 10111, 11011, and 11111 (not shown) are reserved.
            // Do not use these in your encoder. If the hardware is passed blocks
            // with one of these modes specified, the resulting decompressed block
            // must contain all zeroes in all channels except for the alpha channel.
            for i in 0..4 {
                // TODO: function for indexing?
                let index = i * destination_pitch;
                decompressed_block[index..index + 4 * 3].fill(0);
            }

            return;
        }
    }

    let num_partitions = if mode >= 10 { 0 } else { 1 };

    let actual_bits0_mode = ACTUAL_BITS_COUNT[0][mode as usize] as i32;
    if is_signed {
        r[0] = extend_sign(r[0], actual_bits0_mode);
        g[0] = extend_sign(g[0], actual_bits0_mode);
        b[0] = extend_sign(b[0], actual_bits0_mode);
    }
    // Mode 11 (like Mode 10) does not use delta compression,
    // and instead stores both color endpoints explicitly.
    if mode != 9 && mode != 10 || is_signed {
        for i in 1..(num_partitions + 1) * 2 {
            r[i] = extend_sign(r[i], ACTUAL_BITS_COUNT[1][mode as usize] as i32);
            g[i] = extend_sign(g[i], ACTUAL_BITS_COUNT[2][mode as usize] as i32);
            b[i] = extend_sign(b[i], ACTUAL_BITS_COUNT[3][mode as usize] as i32);
        }
    }

    if mode != 9 && mode != 10 {
        for i in 1..(num_partitions + 1) * 2 {
            r[i] = transform_inverse(r[i], r[0], actual_bits0_mode, is_signed);
            g[i] = transform_inverse(g[i], g[0], actual_bits0_mode, is_signed);
            b[i] = transform_inverse(b[i], b[0], actual_bits0_mode, is_signed);
        }
    }

    for i in 0..(num_partitions + 1) * 2 {
        r[i] = unquantize(r[i], actual_bits0_mode, is_signed);
        g[i] = unquantize(g[i], actual_bits0_mode, is_signed);
        b[i] = unquantize(b[i], actual_bits0_mode, is_signed);
    }

    let weights = if mode >= 10 {
        &a_weight4[..]
    } else {
        &a_weight3[..]
    };
    for i in 0..4 {
        for j in 0..4 {
            let mut partition_set = if mode >= 10 {
                if i | j != 0 {
                    0usize
                } else {
                    128usize
                }
            } else {
                PARTITION_SETS[partition as usize][i][j] as usize
            };

            let mut index_bits = if mode >= 10 { 4 } else { 3 };
            // fix-up index is specified with one less bit
            // The fix-up index for subset 0 is always index 0
            if (partition_set & 0x80) != 0 {
                index_bits -= 1;
            }
            partition_set &= 0x01;

            let index = bstream.read_bits(index_bits);
            let weight = weights[index as usize];

            let ep_i = partition_set * 2;

            // TODO: function for indexing?
            let out = i * destination_pitch + j * 3;
            decompressed_block[out..out + 3].copy_from_slice(&[
                finish_unquantize(interpolate_i32(r[ep_i], r[ep_i + 1], weight), is_signed),
                finish_unquantize(interpolate_i32(g[ep_i], g[ep_i + 1], weight), is_signed),
                finish_unquantize(interpolate_i32(b[ep_i], b[ep_i + 1], weight), is_signed),
            ]);
        }
    }
}

/// Decode 16 bytes from `compressed_block` to RGBFloat32
/// with `destination_pitch` many floats per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0.0f32; 4 * 4 * 3];
/// bcdec_rs::bc6h_float(&compressed_block, &mut decompressed_block, 4 * 3, false);
/// ```
pub fn bc6h_float(
    compressed_block: &[u8],
    decompressed_block: &mut [f32],
    destination_pitch: usize,
    is_signed: bool,
) {
    let input_pitch = 4 * 3;
    let mut block = [0u16; 4 * 4 * 3];
    bc6h_half(compressed_block, &mut block, input_pitch, is_signed);

    for i in 0..4 {
        for j in 0..4 {
            // TODO: function for indexing?
            // The input is f16 but the output is f32.
            let in_index = i * input_pitch + j * 3;
            let out_index = i * destination_pitch + j * 3;
            decompressed_block[out_index..out_index + 3].copy_from_slice(&[
                half_to_float_quick(block[in_index]),
                half_to_float_quick(block[in_index + 1]),
                half_to_float_quick(block[in_index + 2]),
            ]);
        }
    }
}

/// Decode 16 bytes from `compressed_block` to RGBA8
/// with `destination_pitch` many bytes per output row.
///
/// # Examples
///
/// ```rust
/// // Decode a single 4x4 pixel block.
/// let compressed_block = [0u8; 16];
/// let mut decompressed_block = [0u8; 4 * 4 * 4];
/// bcdec_rs::bc7(&compressed_block, &mut decompressed_block, 4 * 4);
/// ```
pub fn bc7(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    static ACTUAL_BITS_COUNT: [[u8; 8]; 2] = [
        [4, 6, 5, 7, 5, 7, 7, 5], // RGBA
        [0, 0, 0, 0, 6, 8, 7, 5], // Alpha
    ];

    // There are 64 possible partition sets for a two-region tile.
    // Each 4x4 block represents a single shape.
    // Here also every fix-up index has MSB bit set.
    static PARTITION_SETS: [[[[u8; 4]; 4]; 64]; 2] = [
        [
            // Partition table for 2-subset BPTC
            [[128, 0, 1, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 0, 1, 129]], //  0
            [[128, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 129]], //  1
            [[128, 1, 1, 1], [0, 1, 1, 1], [0, 1, 1, 1], [0, 1, 1, 129]], //  2
            [[128, 0, 0, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 1, 1, 129]], //  3
            [[128, 0, 0, 0], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 1, 129]], //  4
            [[128, 0, 1, 1], [0, 1, 1, 1], [0, 1, 1, 1], [1, 1, 1, 129]], //  5
            [[128, 0, 0, 1], [0, 0, 1, 1], [0, 1, 1, 1], [1, 1, 1, 129]], //  6
            [[128, 0, 0, 0], [0, 0, 0, 1], [0, 0, 1, 1], [0, 1, 1, 129]], //  7
            [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 1], [0, 0, 1, 129]], //  8
            [[128, 0, 1, 1], [0, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], //  9
            [[128, 0, 0, 0], [0, 0, 0, 1], [0, 1, 1, 1], [1, 1, 1, 129]], // 10
            [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 1], [0, 1, 1, 129]], // 11
            [[128, 0, 0, 1], [0, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], // 12
            [[128, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 1], [1, 1, 1, 129]], // 13
            [[128, 0, 0, 0], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 129]], // 14
            [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 129]], // 15
            [[128, 0, 0, 0], [1, 0, 0, 0], [1, 1, 1, 0], [1, 1, 1, 129]], // 16
            [[128, 1, 129, 1], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]], // 17
            [[128, 0, 0, 0], [0, 0, 0, 0], [129, 0, 0, 0], [1, 1, 1, 0]], // 18
            [[128, 1, 129, 1], [0, 0, 1, 1], [0, 0, 0, 1], [0, 0, 0, 0]], // 19
            [[128, 0, 129, 1], [0, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0]], // 20
            [[128, 0, 0, 0], [1, 0, 0, 0], [129, 1, 0, 0], [1, 1, 1, 0]], // 21
            [[128, 0, 0, 0], [0, 0, 0, 0], [129, 0, 0, 0], [1, 1, 0, 0]], // 22
            [[128, 1, 1, 1], [0, 0, 1, 1], [0, 0, 1, 1], [0, 0, 0, 129]], // 23
            [[128, 0, 129, 1], [0, 0, 0, 1], [0, 0, 0, 1], [0, 0, 0, 0]], // 24
            [[128, 0, 0, 0], [1, 0, 0, 0], [129, 0, 0, 0], [1, 1, 0, 0]], // 25
            [[128, 1, 129, 0], [0, 1, 1, 0], [0, 1, 1, 0], [0, 1, 1, 0]], // 26
            [[128, 0, 129, 1], [0, 1, 1, 0], [0, 1, 1, 0], [1, 1, 0, 0]], // 27
            [[128, 0, 0, 1], [0, 1, 1, 1], [129, 1, 1, 0], [1, 0, 0, 0]], // 28
            [[128, 0, 0, 0], [1, 1, 1, 1], [129, 1, 1, 1], [0, 0, 0, 0]], // 29
            [[128, 1, 129, 1], [0, 0, 0, 1], [1, 0, 0, 0], [1, 1, 1, 0]], // 30
            [[128, 0, 129, 1], [1, 0, 0, 1], [1, 0, 0, 1], [1, 1, 0, 0]], // 31
            [[128, 1, 0, 1], [0, 1, 0, 1], [0, 1, 0, 1], [0, 1, 0, 129]], // 32
            [[128, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0], [1, 1, 1, 129]], // 33
            [[128, 1, 0, 1], [1, 0, 129, 0], [0, 1, 0, 1], [1, 0, 1, 0]], // 34
            [[128, 0, 1, 1], [0, 0, 1, 1], [129, 1, 0, 0], [1, 1, 0, 0]], // 35
            [[128, 0, 129, 1], [1, 1, 0, 0], [0, 0, 1, 1], [1, 1, 0, 0]], // 36
            [[128, 1, 0, 1], [0, 1, 0, 1], [129, 0, 1, 0], [1, 0, 1, 0]], // 37
            [[128, 1, 1, 0], [1, 0, 0, 1], [0, 1, 1, 0], [1, 0, 0, 129]], // 38
            [[128, 1, 0, 1], [1, 0, 1, 0], [1, 0, 1, 0], [0, 1, 0, 129]], // 39
            [[128, 1, 129, 1], [0, 0, 1, 1], [1, 1, 0, 0], [1, 1, 1, 0]], // 40
            [[128, 0, 0, 1], [0, 0, 1, 1], [129, 1, 0, 0], [1, 0, 0, 0]], // 41
            [[128, 0, 129, 1], [0, 0, 1, 0], [0, 1, 0, 0], [1, 1, 0, 0]], // 42
            [[128, 0, 129, 1], [1, 0, 1, 1], [1, 1, 0, 1], [1, 1, 0, 0]], // 43
            [[128, 1, 129, 0], [1, 0, 0, 1], [1, 0, 0, 1], [0, 1, 1, 0]], // 44
            [[128, 0, 1, 1], [1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 1, 129]], // 45
            [[128, 1, 1, 0], [0, 1, 1, 0], [1, 0, 0, 1], [1, 0, 0, 129]], // 46
            [[128, 0, 0, 0], [0, 1, 129, 0], [0, 1, 1, 0], [0, 0, 0, 0]], // 47
            [[128, 1, 0, 0], [1, 1, 129, 0], [0, 1, 0, 0], [0, 0, 0, 0]], // 48
            [[128, 0, 129, 0], [0, 1, 1, 1], [0, 0, 1, 0], [0, 0, 0, 0]], // 49
            [[128, 0, 0, 0], [0, 0, 129, 0], [0, 1, 1, 1], [0, 0, 1, 0]], // 50
            [[128, 0, 0, 0], [0, 1, 0, 0], [129, 1, 1, 0], [0, 1, 0, 0]], // 51
            [[128, 1, 1, 0], [1, 1, 0, 0], [1, 0, 0, 1], [0, 0, 1, 129]], // 52
            [[128, 0, 1, 1], [0, 1, 1, 0], [1, 1, 0, 0], [1, 0, 0, 129]], // 53
            [[128, 1, 129, 0], [0, 0, 1, 1], [1, 0, 0, 1], [1, 1, 0, 0]], // 54
            [[128, 0, 129, 1], [1, 0, 0, 1], [1, 1, 0, 0], [0, 1, 1, 0]], // 55
            [[128, 1, 1, 0], [1, 1, 0, 0], [1, 1, 0, 0], [1, 0, 0, 129]], // 56
            [[128, 1, 1, 0], [0, 0, 1, 1], [0, 0, 1, 1], [1, 0, 0, 129]], // 57
            [[128, 1, 1, 1], [1, 1, 1, 0], [1, 0, 0, 0], [0, 0, 0, 129]], // 58
            [[128, 0, 0, 1], [1, 0, 0, 0], [1, 1, 1, 0], [0, 1, 1, 129]], // 59
            [[128, 0, 0, 0], [1, 1, 1, 1], [0, 0, 1, 1], [0, 0, 1, 129]], // 60
            [[128, 0, 129, 1], [0, 0, 1, 1], [1, 1, 1, 1], [0, 0, 0, 0]], // 61
            [[128, 0, 129, 0], [0, 0, 1, 0], [1, 1, 1, 0], [1, 1, 1, 0]], // 62
            [[128, 1, 0, 0], [0, 1, 0, 0], [0, 1, 1, 1], [0, 1, 1, 129]], // 63
        ],
        [
            // Partition table for 3-subset BPTC
            [[128, 0, 1, 129], [0, 0, 1, 1], [0, 2, 2, 1], [2, 2, 2, 130]], //  0
            [[128, 0, 0, 129], [0, 0, 1, 1], [130, 2, 1, 1], [2, 2, 2, 1]], //  1
            [[128, 0, 0, 0], [2, 0, 0, 1], [130, 2, 1, 1], [2, 2, 1, 129]], //  2
            [[128, 2, 2, 130], [0, 0, 2, 2], [0, 0, 1, 1], [0, 1, 1, 129]], //  3
            [[128, 0, 0, 0], [0, 0, 0, 0], [129, 1, 2, 2], [1, 1, 2, 130]], //  4
            [[128, 0, 1, 129], [0, 0, 1, 1], [0, 0, 2, 2], [0, 0, 2, 130]], //  5
            [[128, 0, 2, 130], [0, 0, 2, 2], [1, 1, 1, 1], [1, 1, 1, 129]], //  6
            [[128, 0, 1, 1], [0, 0, 1, 1], [130, 2, 1, 1], [2, 2, 1, 129]], //  7
            [[128, 0, 0, 0], [0, 0, 0, 0], [129, 1, 1, 1], [2, 2, 2, 130]], //  8
            [[128, 0, 0, 0], [1, 1, 1, 1], [129, 1, 1, 1], [2, 2, 2, 130]], //  9
            [[128, 0, 0, 0], [1, 1, 129, 1], [2, 2, 2, 2], [2, 2, 2, 130]], // 10
            [[128, 0, 1, 2], [0, 0, 129, 2], [0, 0, 1, 2], [0, 0, 1, 130]], // 11
            [[128, 1, 1, 2], [0, 1, 129, 2], [0, 1, 1, 2], [0, 1, 1, 130]], // 12
            [[128, 1, 2, 2], [0, 129, 2, 2], [0, 1, 2, 2], [0, 1, 2, 130]], // 13
            [[128, 0, 1, 129], [0, 1, 1, 2], [1, 1, 2, 2], [1, 2, 2, 130]], // 14
            [[128, 0, 1, 129], [2, 0, 0, 1], [130, 2, 0, 0], [2, 2, 2, 0]], // 15
            [[128, 0, 0, 129], [0, 0, 1, 1], [0, 1, 1, 2], [1, 1, 2, 130]], // 16
            [[128, 1, 1, 129], [0, 0, 1, 1], [130, 0, 0, 1], [2, 2, 0, 0]], // 17
            [[128, 0, 0, 0], [1, 1, 2, 2], [129, 1, 2, 2], [1, 1, 2, 130]], // 18
            [[128, 0, 2, 130], [0, 0, 2, 2], [0, 0, 2, 2], [1, 1, 1, 129]], // 19
            [[128, 1, 1, 129], [0, 1, 1, 1], [0, 2, 2, 2], [0, 2, 2, 130]], // 20
            [[128, 0, 0, 129], [0, 0, 0, 1], [130, 2, 2, 1], [2, 2, 2, 1]], // 21
            [[128, 0, 0, 0], [0, 0, 129, 1], [0, 1, 2, 2], [0, 1, 2, 130]], // 22
            [[128, 0, 0, 0], [1, 1, 0, 0], [130, 2, 129, 0], [2, 2, 1, 0]], // 23
            [[128, 1, 2, 130], [0, 129, 2, 2], [0, 0, 1, 1], [0, 0, 0, 0]], // 24
            [[128, 0, 1, 2], [0, 0, 1, 2], [129, 1, 2, 2], [2, 2, 2, 130]], // 25
            [[128, 1, 1, 0], [1, 2, 130, 1], [129, 2, 2, 1], [0, 1, 1, 0]], // 26
            [[128, 0, 0, 0], [0, 1, 129, 0], [1, 2, 130, 1], [1, 2, 2, 1]], // 27
            [[128, 0, 2, 2], [1, 1, 0, 2], [129, 1, 0, 2], [0, 0, 2, 130]], // 28
            [[128, 1, 1, 0], [0, 129, 1, 0], [2, 0, 0, 2], [2, 2, 2, 130]], // 29
            [[128, 0, 1, 1], [0, 1, 2, 2], [0, 1, 130, 2], [0, 0, 1, 129]], // 30
            [[128, 0, 0, 0], [2, 0, 0, 0], [130, 2, 1, 1], [2, 2, 2, 129]], // 31
            [[128, 0, 0, 0], [0, 0, 0, 2], [129, 1, 2, 2], [1, 2, 2, 130]], // 32
            [[128, 2, 2, 130], [0, 0, 2, 2], [0, 0, 1, 2], [0, 0, 1, 129]], // 33
            [[128, 0, 1, 129], [0, 0, 1, 2], [0, 0, 2, 2], [0, 2, 2, 130]], // 34
            [[128, 1, 2, 0], [0, 129, 2, 0], [0, 1, 130, 0], [0, 1, 2, 0]], // 35
            [[128, 0, 0, 0], [1, 1, 129, 1], [2, 2, 130, 2], [0, 0, 0, 0]], // 36
            [[128, 1, 2, 0], [1, 2, 0, 1], [130, 0, 129, 2], [0, 1, 2, 0]], // 37
            [[128, 1, 2, 0], [2, 0, 1, 2], [129, 130, 0, 1], [0, 1, 2, 0]], // 38
            [[128, 0, 1, 1], [2, 2, 0, 0], [1, 1, 130, 2], [0, 0, 1, 129]], // 39
            [[128, 0, 1, 1], [1, 1, 130, 2], [2, 2, 0, 0], [0, 0, 1, 129]], // 40
            [[128, 1, 0, 129], [0, 1, 0, 1], [2, 2, 2, 2], [2, 2, 2, 130]], // 41
            [[128, 0, 0, 0], [0, 0, 0, 0], [130, 1, 2, 1], [2, 1, 2, 129]], // 42
            [[128, 0, 2, 2], [1, 129, 2, 2], [0, 0, 2, 2], [1, 1, 2, 130]], // 43
            [[128, 0, 2, 130], [0, 0, 1, 1], [0, 0, 2, 2], [0, 0, 1, 129]], // 44
            [[128, 2, 2, 0], [1, 2, 130, 1], [0, 2, 2, 0], [1, 2, 2, 129]], // 45
            [[128, 1, 0, 1], [2, 2, 130, 2], [2, 2, 2, 2], [0, 1, 0, 129]], // 46
            [[128, 0, 0, 0], [2, 1, 2, 1], [130, 1, 2, 1], [2, 1, 2, 129]], // 47
            [[128, 1, 0, 129], [0, 1, 0, 1], [0, 1, 0, 1], [2, 2, 2, 130]], // 48
            [[128, 2, 2, 130], [0, 1, 1, 1], [0, 2, 2, 2], [0, 1, 1, 129]], // 49
            [[128, 0, 0, 2], [1, 129, 1, 2], [0, 0, 0, 2], [1, 1, 1, 130]], // 50
            [[128, 0, 0, 0], [2, 129, 1, 2], [2, 1, 1, 2], [2, 1, 1, 130]], // 51
            [[128, 2, 2, 2], [0, 129, 1, 1], [0, 1, 1, 1], [0, 2, 2, 130]], // 52
            [[128, 0, 0, 2], [1, 1, 1, 2], [129, 1, 1, 2], [0, 0, 0, 130]], // 53
            [[128, 1, 1, 0], [0, 129, 1, 0], [0, 1, 1, 0], [2, 2, 2, 130]], // 54
            [[128, 0, 0, 0], [0, 0, 0, 0], [2, 1, 129, 2], [2, 1, 1, 130]], // 55
            [[128, 1, 1, 0], [0, 129, 1, 0], [2, 2, 2, 2], [2, 2, 2, 130]], // 56
            [[128, 0, 2, 2], [0, 0, 1, 1], [0, 0, 129, 1], [0, 0, 2, 130]], // 57
            [[128, 0, 2, 2], [1, 1, 2, 2], [129, 1, 2, 2], [0, 0, 2, 130]], // 58
            [[128, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [2, 129, 1, 130]], // 59
            [[128, 0, 0, 130], [0, 0, 0, 1], [0, 0, 0, 2], [0, 0, 0, 129]], // 60
            [[128, 2, 2, 2], [1, 2, 2, 2], [0, 2, 2, 2], [129, 2, 2, 130]], // 61
            [[128, 1, 0, 129], [2, 2, 2, 2], [2, 2, 2, 2], [2, 2, 2, 130]], // 62
            [[128, 1, 1, 129], [2, 0, 1, 1], [130, 2, 0, 1], [2, 2, 2, 0]], // 63
        ],
    ];

    let a_weight2 = [0, 21, 43, 64];
    let a_weight3 = [0, 9, 18, 27, 37, 46, 55, 64];
    let a_weight4 = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

    let s_mode_has_pbits = 0b11001011;

    let mut bstream = Bitstream {
        low: u64::from_le_bytes(compressed_block[0..8].try_into().unwrap()),
        high: u64::from_le_bytes(compressed_block[8..16].try_into().unwrap()),
    };

    let mut endpoints = [[0; 4]; 6];
    let mut indices = [[0u8; 4]; 4];

    let mut mode = 0;
    while mode < 8 && (0 == bstream.read_bit()) {
        mode += 1;
    }

    // unexpected mode, clear the block (transparent black)
    if mode >= 8 {
        for i in 0..4 {
            let index = i * destination_pitch;
            decompressed_block[index..index + 4 * 4].fill(0);
        }

        return;
    }

    let mut partition = 0;
    let mut num_partitions = 1;
    let mut rotation = 0;
    let mut index_selection_bit = 0;

    if mode == 0 || mode == 1 || mode == 2 || mode == 3 || mode == 7 {
        num_partitions = if mode == 0 || mode == 2 { 3 } else { 2 };
        partition = bstream.read_bits(if mode == 0 { 4 } else { 6 });
    }

    let num_endpoints = num_partitions * 2;

    if mode == 4 || mode == 5 {
        rotation = bstream.read_bits(2);

        if mode == 4 {
            index_selection_bit = bstream.read_bit();
        }
    }

    // Extract endpoints
    // RGB
    for i in 0..3 {
        for endpoint in endpoints.iter_mut().take(num_endpoints) {
            endpoint[i] = bstream.read_bits(ACTUAL_BITS_COUNT[0][mode] as u32);
        }
    }
    // Alpha (if any)
    if ACTUAL_BITS_COUNT[1][mode] > 0 {
        for endpoint in endpoints.iter_mut().take(num_endpoints) {
            endpoint[3] = bstream.read_bits(ACTUAL_BITS_COUNT[1][mode] as u32);
        }
    }

    // Fully decode endpoints
    // First handle modes that have P-bits
    if mode == 0 || mode == 1 || mode == 3 || mode == 6 || mode == 7 {
        for endpoint in endpoints.iter_mut().take(num_endpoints) {
            // component-wise left-shift
            for endpoint in endpoint.iter_mut().take(4) {
                *endpoint <<= 1;
            }
        }

        // if P-bit is shared
        if mode == 1 {
            let i = bstream.read_bit();
            let j = bstream.read_bit();

            // rgb component-wise insert pbits
            for k in 0..3 {
                endpoints[0][k] |= i;
                endpoints[1][k] |= i;
                endpoints[2][k] |= j;
                endpoints[3][k] |= j;
            }
        } else if (s_mode_has_pbits & (1 << mode)) != 0 {
            // unique P-bit per endpoint
            for endpoint in endpoints.iter_mut().take(num_endpoints) {
                let j = bstream.read_bit();
                for endpoint in endpoint.iter_mut().take(4) {
                    *endpoint |= j;
                }
            }
        }
    }

    for endpoint in endpoints.iter_mut().take(num_endpoints) {
        // get color components precision including pbit
        let j = ACTUAL_BITS_COUNT[0][mode] + ((s_mode_has_pbits >> mode) & 1);

        for endpoint in endpoint.iter_mut().take(3) {
            // left shift endpoint components so that their MSB lies in bit 7
            *endpoint <<= 8 - j;
            // Replicate each component's MSB into the LSBs revealed by the left-shift operation above
            *endpoint |= *endpoint >> j;
        }

        // get alpha component precision including pbit
        let j = ACTUAL_BITS_COUNT[1][mode] + ((s_mode_has_pbits >> mode) & 1);

        // left shift endpoint components so that their MSB lies in bit 7
        endpoint[3] <<= 8 - j;
        // Replicate each component's MSB into the LSBs revealed by the left-shift operation above
        endpoint[3] |= endpoint[3] >> j;
    }

    // If this mode does not explicitly define the alpha component
    // set alpha equal to 1.0
    if ACTUAL_BITS_COUNT[1][mode] == 0 {
        for endpoint in endpoints.iter_mut().take(num_endpoints) {
            endpoint[3] = 0xFF;
        }
    }

    // Determine weights tables
    let mut index_bits = if mode == 0 || mode == 1 {
        3
    } else if mode == 6 {
        4
    } else {
        2
    };
    let index_bits2 = if mode == 4 {
        3
    } else if mode == 5 {
        2
    } else {
        0
    };
    let weights = if index_bits == 2 {
        &a_weight2[..]
    } else if index_bits == 3 {
        &a_weight3[..]
    } else {
        &a_weight4[..]
    };
    let weights2 = if index_bits2 == 2 {
        &a_weight2[..]
    } else {
        &a_weight3[..]
    };

    // Quite inconvenient that indices aren't interleaved so we have to make 2 passes here
    // Pass #1: collecting color indices
    for (i, indices) in indices.iter_mut().enumerate() {
        for (j, index) in indices.iter_mut().enumerate() {
            let partition_set = if num_partitions == 1 {
                if i | j != 0 {
                    0
                } else {
                    128
                }
            } else {
                PARTITION_SETS[num_partitions - 2][partition as usize][i][j]
            };

            index_bits = if mode == 0 || mode == 1 {
                3
            } else if mode == 6 {
                4
            } else {
                2
            };
            // fix-up index is specified with one less bit
            // The fix-up index for subset 0 is always index 0
            if partition_set & 0x80 != 0 {
                index_bits -= 1;
            }

            *index = bstream.read_bits(index_bits) as u8;
        }
    }

    // Pass #2: reading alpha indices (if any) and interpolating & rotating
    for (i, indices) in indices.iter().enumerate() {
        assert!(decompressed_block.len() >= i * destination_pitch + 4 * 4);

        for (j, index) in indices.iter().enumerate() {
            let mut partition_set = if num_partitions == 1 {
                if i | j != 0 {
                    0usize
                } else {
                    128usize
                }
            } else {
                PARTITION_SETS[num_partitions - 2][partition as usize][i][j] as usize
            };
            partition_set &= 0x03;

            let weight = weights[*index as usize];

            let mut r;
            let mut g;
            let mut b;
            let mut a;
            if index_bits2 == 0 {
                r = interpolate(
                    endpoints[partition_set * 2][0],
                    endpoints[partition_set * 2 + 1][0],
                    weight,
                );
                g = interpolate(
                    endpoints[partition_set * 2][1],
                    endpoints[partition_set * 2 + 1][1],
                    weight,
                );
                b = interpolate(
                    endpoints[partition_set * 2][2],
                    endpoints[partition_set * 2 + 1][2],
                    weight,
                );
                a = interpolate(
                    endpoints[partition_set * 2][3],
                    endpoints[partition_set * 2 + 1][3],
                    weight,
                );
            } else {
                let index2 = bstream.read_bits(if i | j != 0 {
                    index_bits2
                } else {
                    index_bits2 - 1
                });
                let weight2 = weights2[index2 as usize];

                // The index value for interpolating color comes from the secondary index bits for the texel
                // if the mode has an index selection bit and its value is one, and from the primary index bits otherwise.
                // The alpha index comes from the secondary index bits if the block has a secondary index and
                // the block either doesn’t have an index selection bit or that bit is zero, and from the primary index bits otherwise.
                if index_selection_bit == 0 {
                    r = interpolate(
                        endpoints[partition_set * 2][0],
                        endpoints[partition_set * 2 + 1][0],
                        weight,
                    );
                    g = interpolate(
                        endpoints[partition_set * 2][1],
                        endpoints[partition_set * 2 + 1][1],
                        weight,
                    );
                    b = interpolate(
                        endpoints[partition_set * 2][2],
                        endpoints[partition_set * 2 + 1][2],
                        weight,
                    );
                    a = interpolate(
                        endpoints[partition_set * 2][3],
                        endpoints[partition_set * 2 + 1][3],
                        weight2,
                    );
                } else {
                    r = interpolate(
                        endpoints[partition_set * 2][0],
                        endpoints[partition_set * 2 + 1][0],
                        weight2,
                    );
                    g = interpolate(
                        endpoints[partition_set * 2][1],
                        endpoints[partition_set * 2 + 1][1],
                        weight2,
                    );
                    b = interpolate(
                        endpoints[partition_set * 2][2],
                        endpoints[partition_set * 2 + 1][2],
                        weight2,
                    );
                    a = interpolate(
                        endpoints[partition_set * 2][3],
                        endpoints[partition_set * 2 + 1][3],
                        weight,
                    );
                }
            }

            match rotation {
                1 => {
                    // 01 – Block format is Scalar(R) Vector(AGB) - swap A and R
                    core::mem::swap(&mut a, &mut r);
                }
                2 => {
                    // 10 – Block format is Scalar(G) Vector(RAB) - swap A and G
                    core::mem::swap(&mut a, &mut g);
                }
                3 => {
                    // 11 - Block format is Scalar(B) Vector(RGA) - swap A and B
                    core::mem::swap(&mut a, &mut b);
                }
                _ => (),
            }

            // TODO: function for indexing?
            let index = i * destination_pitch + j * 4;
            decompressed_block[index..index + 4]
                .copy_from_slice(&[r as u8, g as u8, b as u8, a as u8]);
        }
    }
}

fn color_block(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    only_opaque_mode: bool,
) {
    let mut ref_colors = [[0u8; 4]; 4]; // 0xAABBGGRR

    let c0 = u16::from_le_bytes(compressed_block[0..2].try_into().unwrap());
    let c1 = u16::from_le_bytes(compressed_block[2..4].try_into().unwrap());

    // Unpack 565 ref colors
    let r0 = (c0 as u32 >> 11) & 0x1F;
    let g0 = (c0 as u32 >> 5) & 0x3F;
    let b0 = c0 as u32 & 0x1F;

    let r1 = (c1 as u32 >> 11) & 0x1F;
    let g1 = (c1 as u32 >> 5) & 0x3F;
    let b1 = c1 as u32 & 0x1F;

    // Expand 565 ref colors to 888
    let r = (r0 * 527 + 23) >> 6;
    let g = (g0 * 259 + 33) >> 6;
    let b = (b0 * 527 + 23) >> 6;
    ref_colors[0] = [r as u8, g as u8, b as u8, 255];

    let r = (r1 * 527 + 23) >> 6;
    let g = (g1 * 259 + 33) >> 6;
    let b = (b1 * 527 + 23) >> 6;
    ref_colors[1] = [r as u8, g as u8, b as u8, 255];

    if c0 > c1 || only_opaque_mode {
        // Standard BC1 mode (also BC3 color block uses ONLY this mode)
        // color_2 = 2/3*color_0 + 1/3*color_1
        // color_3 = 1/3*color_0 + 2/3*color_1
        let r = ((2 * r0 + r1) * 351 + 61) >> 7;
        let g = ((2 * g0 + g1) * 2763 + 1039) >> 11;
        let b = ((2 * b0 + b1) * 351 + 61) >> 7;
        ref_colors[2] = [r as u8, g as u8, b as u8, 255u8];

        let r = ((r0 + r1 * 2) * 351 + 61) >> 7;
        let g = ((g0 + g1 * 2) * 2763 + 1039) >> 11;
        let b = ((b0 + b1 * 2) * 351 + 61) >> 7;
        ref_colors[3] = [r as u8, g as u8, b as u8, 255u8];
    } else {
        // Quite rare BC1A mode
        // color_2 = 1/2*color_0 + 1/2*color_1;
        // color_3 = 0;
        let r = ((r0 + r1) * 1053 + 125) >> 8;
        let g = ((g0 + g1) * 4145 + 1019) >> 11;
        let b = ((b0 + b1) * 1053 + 125) >> 8;
        ref_colors[2] = [r as u8, g as u8, b as u8, 255u8];

        ref_colors[3] = [0u8; 4];
    }

    let mut color_indices = u32::from_le_bytes(compressed_block[4..8].try_into().unwrap());

    // Fill out the decompressed color block
    for i in 0..4 {
        for j in 0..4 {
            let idx = color_indices & 0x03;
            let start = i * destination_pitch + j * 4;
            decompressed_block[start..start + 4].copy_from_slice(&ref_colors[idx as usize]);
            color_indices >>= 2;
        }
    }
}

fn sharp_alpha_block(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
) {
    for i in 0..4 {
        for j in 0..4 {
            // TODO: Function for indexing?
            let index = i * destination_pitch + j * 4 + 3;
            let alpha = u16::from_le_bytes(compressed_block[i * 2..i * 2 + 2].try_into().unwrap());
            decompressed_block[index] = ((alpha >> (4 * j)) & 0x0F) as u8 * 17;
        }
    }
}

fn smooth_alpha_block(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    pixel_size: usize,
) {
    let mut alpha = [0u32; 8];

    alpha[0] = compressed_block[0] as u32;
    alpha[1] = compressed_block[1] as u32;

    if alpha[0] > alpha[1] {
        // 6 interpolated alpha values.
        alpha[2] = (6 * alpha[0] + alpha[1] + 1) / 7; // 6/7*alpha_0 + 1/7*alpha_1
        alpha[3] = (5 * alpha[0] + 2 * alpha[1] + 1) / 7; // 5/7*alpha_0 + 2/7*alpha_1
        alpha[4] = (4 * alpha[0] + 3 * alpha[1] + 1) / 7; // 4/7*alpha_0 + 3/7*alpha_1
        alpha[5] = (3 * alpha[0] + 4 * alpha[1] + 1) / 7; // 3/7*alpha_0 + 4/7*alpha_1
        alpha[6] = (2 * alpha[0] + 5 * alpha[1] + 1) / 7; // 2/7*alpha_0 + 5/7*alpha_1
        alpha[7] = (alpha[0] + 6 * alpha[1] + 1) / 7; // 1/7*alpha_0 + 6/7*alpha_1
    } else {
        // 4 interpolated alpha values.
        alpha[2] = (4 * alpha[0] + alpha[1] + 1) / 5; // 4/5*alpha_0 + 1/5*alpha_1
        alpha[3] = (3 * alpha[0] + 2 * alpha[1] + 1) / 5; // 3/5*alpha_0 + 2/5*alpha_1
        alpha[4] = (2 * alpha[0] + 3 * alpha[1] + 1) / 5; // 2/5*alpha_0 + 3/5*alpha_1
        alpha[5] = (alpha[0] + 4 * alpha[1] + 1) / 5; // 1/5*alpha_0 + 4/5*alpha_1
        alpha[6] = 0x00;
        alpha[7] = 0xFF;
    }

    let block = u64::from_le_bytes(compressed_block[..8].try_into().unwrap());
    let mut indices = block >> 16;
    for i in 0..4 {
        for j in 0..4 {
            // TODO: Function for indexing?
            let index = i * destination_pitch + j * pixel_size;
            decompressed_block[index] = alpha[(indices & 0x07) as usize] as u8;
            indices >>= 3;
        }
    }
}

/// From: <https://github.com/image-rs/image/blob/ab0ec2cd79857ba902dde1b28624a1dea458fe2b/src/codecs/dds/bc.rs#L140>
fn smooth_alpha_block_signed(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    pixel_size: usize,
) {
    let mut alpha = [0u32; 8];

    fn snorm8_to_unorm8(x: u8) -> u8 {
        let y = x.wrapping_add(128).saturating_sub(1) as u16;
        ((y * 129 + 1) >> 7) as u8
    }

    let red0 = compressed_block[0];
    let red1 = compressed_block[1];

    alpha[0] = snorm8_to_unorm8(red0) as u32;
    alpha[1] = snorm8_to_unorm8(red1) as u32;

    const CONVERSION_FACTOR: f32 = 255.0 / 254.0;
    let alpha0_f = red0.wrapping_add(128).saturating_sub(1) as f32 * CONVERSION_FACTOR;
    let alpha1_f = red1.wrapping_add(128).saturating_sub(1) as f32 * CONVERSION_FACTOR;

    if alpha[0] > alpha[1] {
        // 6 interpolated alpha values.
        alpha[2] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 1.0 / 7.0);
        alpha[3] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 2.0 / 7.0);
        alpha[4] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 3.0 / 7.0);
        alpha[5] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 4.0 / 7.0);
        alpha[6] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 5.0 / 7.0);
        alpha[7] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 6.0 / 7.0);
    } else {
        // 4 interpolated alpha values.
        alpha[2] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 1.0 / 5.0);
        alpha[3] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 2.0 / 5.0);
        alpha[4] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 3.0 / 5.0);
        alpha[5] = interpolate_f32_to_u32(alpha0_f, alpha1_f, 4.0 / 5.0);
        alpha[6] = 0x00;
        alpha[7] = 0xFF;
    };

    let block = u64::from_le_bytes(compressed_block[..8].try_into().unwrap());
    let mut indices = block >> 16;
    for i in 0..4 {
        for j in 0..4 {
            // TODO: Function for indexing?
            let index = i * destination_pitch + j * pixel_size;
            decompressed_block[index] = alpha[(indices & 0x07) as usize] as u8;
            indices >>= 3;
        }
    }
}

struct Bitstream {
    low: u64,
    high: u64,
}

impl Bitstream {
    fn read_bits(&mut self, num_bits: u32) -> u32 {
        let mask = (1 << num_bits) - 1;
        // Read the low N bits
        let bits = self.low & mask;

        self.low >>= num_bits;
        // Put the low N bits of "high" into the high 64-N bits of "low".
        self.low |= (self.high & mask) << (u64::BITS as u64 - num_bits as u64);
        self.high >>= num_bits;

        bits as u32
    }

    fn read_bit(&mut self) -> u32 {
        self.read_bits(1)
    }

    // TODO: Ok to combine these with unsigned?
    fn read_bits_i32(&mut self, num_bits: u32) -> i32 {
        self.read_bits(num_bits) as i32
    }

    fn read_bit_i32(&mut self) -> i32 {
        self.read_bit() as i32
    }

    // reversed bits pulling, used in BC6H decoding
    // why ?? just why ???
    fn read_bits_r(&mut self, num_bits: u32) -> i32 {
        let mut bits = self.read_bits_i32(num_bits);
        // Reverse the bits.
        let mut result = 0;
        for _ in 0..num_bits {
            result <<= 1;
            result |= bits & 1;
            bits >>= 1;
        }
        result
    }
}

fn extend_sign(val: i32, bits: i32) -> i32 {
    (val << (32 - bits)) >> (32 - bits)
}

fn transform_inverse(val: i32, a0: i32, bits: i32, is_signed: bool) -> i32 {
    // If the precision of A0 is "p" bits, then the transform algorithm is:
    // B0 = (B0 + A0) & ((1 << p) - 1)
    let mut val = (val + a0) & ((1 << bits) - 1);
    if is_signed {
        val = extend_sign(val, bits);
    }
    val
}

// pretty much copy-paste from documentation
fn unquantize(val: i32, bits: i32, is_signed: bool) -> i32 {
    let mut unq;
    let mut s = 0;
    let mut val = val;

    if !is_signed {
        if bits >= 15 {
            unq = val;
        } else if val == 0 {
            unq = 0;
        } else if val == (1 << bits) - 1 {
            unq = 0xFFFF;
        } else {
            unq = ((val << 16) + 0x8000) >> bits;
        }
    } else {
        if bits >= 16 {
            // TODO: Dead code?
            // unq = val;
        } else if val < 0 {
            s = 1;
            val = -val;
        }

        if val == 0 {
            unq = 0;
        } else if val >= ((1 << (bits - 1)) - 1) {
            unq = 0x7FFF;
        } else {
            unq = ((val << 15) + 0x4000) >> (bits - 1);
        }

        if s != 0 {
            unq = -unq;
        }
    }
    unq
}

fn interpolate(a: u32, b: u32, weight: u32) -> u32 {
    (a * (64 - weight) + b * weight + 32) >> 6
}

// TODO: Combine these with unsigned?
fn interpolate_i32(a: i32, b: i32, weight: i32) -> i32 {
    (a * (64 - weight) + b * weight + 32) >> 6
}

fn interpolate_f32_to_u32(red0: f32, red1: f32, blend: f32) -> u32 {
    (red0 * (1.0 - blend) + red1 * blend + 0.5) as u32
}

fn finish_unquantize(val: i32, is_signed: bool) -> u16 {
    if !is_signed {
        ((val * 31) >> 6) as u16 // scale the magnitude by 31 / 64
    } else {
        let mut val = if val < 0 {
            -((-val * 31) >> 5)
        } else {
            (val * 31) >> 5
        }; // scale the magnitude by 31 / 32
        let mut s = 0;
        if val < 0 {
            s = 0x8000;
            val = -val;
        }
        (s | val) as u16
    }
}

// modified half_to_float_fast4 from https://gist.github.com/rygorous/2144712
// modified for Rust port to avoid unsafe unions and transmutes
fn half_to_float_quick(half: u16) -> f32 {
    let magic = f32::from_bits(113 << 23);
    let shifted_exp = 0x7c00 << 13; // exponent mask after shift

    let mut o = (half as u32 & 0x7fff) << 13; // exponent/mantissa bits
    let exp = shifted_exp & o; // just the exponent
    o += (127 - 15) << 23; // exponent adjust

    // handle exponent special cases
    if exp == shifted_exp {
        // Inf/NaN?
        o += (128 - 16) << 23; // extra exp adjust
    } else if exp == 0 {
        // Zero/Denormal?
        o += 1 << 23; // extra exp adjust
        o = (f32::from_bits(o) - magic).to_bits(); // renormalize
    }

    o |= (half as u32 & 0x8000) << 16; // sign bit
    f32::from_bits(o)
}
