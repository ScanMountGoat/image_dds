#![no_std]
//! A safe, no_std, pure Rust port of [bcdec](https://github.com/iOrange/bcdec).
// A mostly 1:1 translation of the code and comments found here:
// https://github.com/iOrange/bcdec/blob/main/bcdec.h
// Names are shortened and pointer arithmetic is converted to more idiomatic Rust.
// TODO: Create helpers for working with byte slices?
// TODO: Do we need to convert to integers and deal with endianness?
// TODO: Fiddle with asserts and codegen to get similar assembly.
pub fn bc1(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    color_block(
        compressed_block,
        decompressed_block,
        destination_pitch,
        false,
    )
}

pub fn bc2(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    color_block(
        &compressed_block[8..],
        decompressed_block,
        destination_pitch,
        true,
    );
    sharp_alpha_block(compressed_block, decompressed_block, destination_pitch);
}

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

pub fn bc4(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    smooth_alpha_block(compressed_block, decompressed_block, destination_pitch, 1);
}

pub fn bc5(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    smooth_alpha_block(compressed_block, decompressed_block, destination_pitch, 2);
    smooth_alpha_block(
        &compressed_block[8..],
        &mut decompressed_block[1..],
        destination_pitch,
        2,
    );
}

pub fn bc6h_float(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    is_signed: usize,
) {
    todo!()
}

pub fn bc6h_half(
    compressed_block: &[u8],
    decompressed_block: &mut [u8],
    destination_pitch: usize,
    is_signed: usize,
) {
    todo!()
}

pub fn bc7(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    let actual_bits_count = [
        [4, 6, 5, 7, 5, 7, 7, 5], // RGBA
        [0, 0, 0, 0, 6, 8, 7, 5], // Alpha
    ];

    // There are 64 possible partition sets for a two-region tile.
    // Each 4x4 block represents a single shape.
    // Here also every fix-up index has MSB bit set.
    let partition_sets = [
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
    let mut partition_set;

    let mut endpoints = [[0; 4]; 6];
    let mut indices = [[0; 4]; 4];

    let mut index;
    let mut index2;

    let mut mode = 0;
    while mode < 8 && (0 == bstream.read_bit()) {
        mode += 1;
    }

    // unexpected mode, clear the block (transparent black)
    if mode >= 8 {
        for i in 0..4 {
            for j in 0..4 {
                // TODO: function for indexing?
                let index = i * destination_pitch + j * 4;
                decompressed_block[index] = 0;
                decompressed_block[index + 1] = 0;
                decompressed_block[index + 2] = 0;
                decompressed_block[index + 3] = 0;
            }
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
        for j in 0..num_endpoints {
            endpoints[j][i] = bstream.read_bits(actual_bits_count[0][mode]);
        }
    }
    // Alpha (if any)
    if actual_bits_count[1][mode] > 0 {
        for j in 0..num_endpoints {
            endpoints[j][3] = bstream.read_bits(actual_bits_count[1][mode]);
        }
    }

    // Fully decode endpoints
    // First handle modes that have P-bits
    if mode == 0 || mode == 1 || mode == 3 || mode == 6 || mode == 7 {
        for i in 0..num_endpoints {
            // component-wise left-shift
            for j in 0..4 {
                endpoints[i][j] <<= 1;
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
            for i in 0..num_endpoints {
                let j = bstream.read_bit();
                for k in 0..4 {
                    endpoints[i][k] |= j;
                }
            }
        }
    }

    for i in 0..num_endpoints {
        // get color components precision including pbit
        let j = actual_bits_count[0][mode] + ((s_mode_has_pbits >> mode) & 1);

        for k in 0..3 {
            // left shift endpoint components so that their MSB lies in bit 7
            endpoints[i][k] <<= 8 - j;
            // Replicate each component's MSB into the LSBs revealed by the left-shift operation above
            endpoints[i][k] |= endpoints[i][k] >> j;
        }

        // get alpha component precision including pbit
        let j = actual_bits_count[1][mode] + ((s_mode_has_pbits >> mode) & 1);

        // left shift endpoint components so that their MSB lies in bit 7
        endpoints[i][3] <<= 8 - j;
        // Replicate each component's MSB into the LSBs revealed by the left-shift operation above
        endpoints[i][3] |= endpoints[i][3] >> j;
    }

    // If this mode does not explicitly define the alpha component
    // set alpha equal to 1.0
    if actual_bits_count[1][mode] == 0 {
        for j in 0..num_endpoints {
            endpoints[j][3] = 0xFF;
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
    for i in 0..4 {
        for j in 0..4 {
            partition_set = if num_partitions == 1 {
                if i | j != 0 {
                    0
                } else {
                    128
                }
            } else {
                partition_sets[num_partitions - 2][partition as usize][i][j]
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

            indices[i][j] = bstream.read_bits(index_bits);
        }
    }

    // Pass #2: reading alpha indices (if any) and interpolating & rotating
    for i in 0..4 {
        for j in 0..4 {
            partition_set = if num_partitions == 1 {
                if i | j != 0 {
                    0
                } else {
                    128
                }
            } else {
                partition_sets[num_partitions - 2][partition as usize][i][j]
            };
            partition_set &= 0x03;

            index = indices[i][j];

            let mut r;
            let mut g;
            let mut b;
            let mut a;
            if index_bits2 == 0 {
                r = interpolate(
                    endpoints[partition_set * 2][0],
                    endpoints[partition_set * 2 + 1][0],
                    weights,
                    index as usize,
                );
                g = interpolate(
                    endpoints[partition_set * 2][1],
                    endpoints[partition_set * 2 + 1][1],
                    weights,
                    index as usize,
                );
                b = interpolate(
                    endpoints[partition_set * 2][2],
                    endpoints[partition_set * 2 + 1][2],
                    weights,
                    index as usize,
                );
                a = interpolate(
                    endpoints[partition_set * 2][3],
                    endpoints[partition_set * 2 + 1][3],
                    weights,
                    index as usize,
                );
            } else {
                index2 = bstream.read_bits(if i | j != 0 {
                    index_bits2
                } else {
                    index_bits2 - 1
                });
                // The index value for interpolating color comes from the secondary index bits for the texel
                // if the mode has an index selection bit and its value is one, and from the primary index bits otherwise.
                // The alpha index comes from the secondary index bits if the block has a secondary index and
                // the block either doesn’t have an index selection bit or that bit is zero, and from the primary index bits otherwise.
                if index_selection_bit == 0 {
                    r = interpolate(
                        endpoints[partition_set * 2][0],
                        endpoints[partition_set * 2 + 1][0],
                        weights,
                        index as usize,
                    );
                    g = interpolate(
                        endpoints[partition_set * 2][1],
                        endpoints[partition_set * 2 + 1][1],
                        weights,
                        index as usize,
                    );
                    b = interpolate(
                        endpoints[partition_set * 2][2],
                        endpoints[partition_set * 2 + 1][2],
                        weights,
                        index as usize,
                    );
                    a = interpolate(
                        endpoints[partition_set * 2][3],
                        endpoints[partition_set * 2 + 1][3],
                        weights2,
                        index2 as usize,
                    );
                } else {
                    r = interpolate(
                        endpoints[partition_set * 2][0],
                        endpoints[partition_set * 2 + 1][0],
                        weights2,
                        index2 as usize,
                    );
                    g = interpolate(
                        endpoints[partition_set * 2][1],
                        endpoints[partition_set * 2 + 1][1],
                        weights2,
                        index2 as usize,
                    );
                    b = interpolate(
                        endpoints[partition_set * 2][2],
                        endpoints[partition_set * 2 + 1][2],
                        weights2,
                        index2 as usize,
                    );
                    a = interpolate(
                        endpoints[partition_set * 2][3],
                        endpoints[partition_set * 2 + 1][3],
                        weights,
                        index as usize,
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
            decompressed_block[index] = r as u8;
            decompressed_block[index + 1] = g as u8;
            decompressed_block[index + 2] = b as u8;
            decompressed_block[index + 3] = a as u8;
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

    // Expand 565 ref colors to 888
    let r0 = (((c0 >> 11) & 0x1F) * 527 + 23) >> 6;
    let g0 = (((c0 >> 5) & 0x3F) * 259 + 33) >> 6;
    let b0 = ((c0 & 0x1F) * 527 + 23) >> 6;
    ref_colors[0] = [r0 as u8, g0 as u8, b0 as u8, 255u8];

    let r1 = (((c1 >> 11) & 0x1F) * 527 + 23) >> 6;
    let g1 = (((c1 >> 5) & 0x3F) * 259 + 33) >> 6;
    let b1 = ((c1 & 0x1F) * 527 + 23) >> 6;
    ref_colors[1] = [r1 as u8, g1 as u8, b1 as u8, 255u8];

    if c0 > c1 || only_opaque_mode {
        // Standard BC1 mode (also BC3 color block uses ONLY this mode)
        // color_2 = 2/3*color_0 + 1/3*color_1
        // color_3 = 1/3*color_0 + 2/3*color_1
        let r = (2 * r0 + r1 + 1) / 3;
        let g = (2 * g0 + g1 + 1) / 3;
        let b = (2 * b0 + b1 + 1) / 3;
        ref_colors[2] = [r as u8, g as u8, b as u8, 255u8];

        let r = (r0 + 2 * r1 + 1) / 3;
        let g = (g0 + 2 * g1 + 1) / 3;
        let b = (b0 + 2 * b1 + 1) / 3;
        ref_colors[3] = [r as u8, g as u8, b as u8, 255u8];
    } else {
        // Quite rare BC1A mode
        // color_2 = 1/2*color_0 + 1/2*color_1;
        // color_3 = 0;
        let r = (r0 + r1 + 1) >> 1;
        let g = (g0 + g1 + 1) >> 1;
        let b = (b0 + b1 + 1) >> 1;
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

struct Bitstream {
    low: u64,
    high: u64,
}

impl Bitstream {
    fn read_bits(&mut self, num_bits: u64) -> u64 {
        let mask = (1 << num_bits) - 1;
        // Read the low N bits
        let bits = self.low & mask;

        self.low >>= num_bits;
        // Put the low N bits of "high" into the high 64-N bits of "low".
        self.low |= (self.high & mask) << (u64::BITS as u64 - num_bits);
        self.high >>= num_bits;

        bits
    }

    fn read_bit(&mut self) -> u64 {
        self.read_bits(1)
    }
}

fn interpolate(a: u64, b: u64, weights: &[u64], index: usize) -> u64 {
    (a * (64 - weights[index]) + b * weights[index] + 32) >> 6
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: one test for each of the formats?
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
