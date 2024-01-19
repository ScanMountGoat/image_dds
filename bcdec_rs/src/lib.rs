//! A pure Rust port of [bcdec](https://github.com/iOrange/bcdec) using only safe code.

// TODO: make this nostd?

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
    todo!()
}

pub fn bc4(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    todo!()
}

pub fn bc5(compressed_block: &[u8], decompressed_block: &mut [u8], destination_pitch: usize) {
    todo!()
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
    todo!()
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: one test for each of the formats?
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
