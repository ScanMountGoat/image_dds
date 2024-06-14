use std::io::BufWriter;

use image_dds::Surface;

// Legacy format decoding behavior can be tested exhaustively.
// This requires making some assumptions about the formats.
// Test data is based on the following blogpost:
// https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
fn main() {
    // BC1
    let surface = bc1_r();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_r.dds");

    let surface = bc1_g();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_g.dds");

    let surface = bc1_b();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_b.dds");

    // BC2
    let surface = bc2_r();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc2_r.dds");

    let surface = bc2_g();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc2_g.dds");

    let surface = bc2_b();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc2_b.dds");

    // BC3
    let surface = bc3_r();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc3_r.dds");

    let surface = bc3_g();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc3_g.dds");

    let surface = bc3_b();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc3_b.dds");

    // BC4
    let surface = bc4_r();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc4_r.dds");

    // BC5
    let surface = bc5_r();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc5_r.dds");

    let surface = bc5_g();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc5_g.dds");

    // TODO: How to handle BC6 and BC7?
}

fn save_dds(dds: ddsfile::Dds, path: &str) {
    let mut writer = BufWriter::new(std::fs::File::create(path).unwrap());
    dds.write(&mut writer).unwrap();
}

fn bc1_r() -> Surface<Vec<u8>> {
    // 5-bit independent R channel for BC1 end points.
    bcn(5, image_dds::ImageFormat::BC1RgbaUnorm, |i, j| {
        bc1_block(i, 0, 0, j, 0, 0).to_le_bytes()
    })
}

fn bc1_b() -> Surface<Vec<u8>> {
    // 5-bit independent B channel for BC1 end points.
    bcn(5, image_dds::ImageFormat::BC1RgbaUnorm, |i, j| {
        bc1_block(0, 0, i, 0, 0, j).to_le_bytes()
    })
}

fn bc1_g() -> Surface<Vec<u8>> {
    // 6-bit independent G channel for BC1 end points.
    bcn(6, image_dds::ImageFormat::BC1RgbaUnorm, |i, j| {
        bc1_block(0, i, 0, 0, j, 0).to_le_bytes()
    })
}

fn bc1_block(r0: u64, g0: u64, b0: u64, r1: u64, g1: u64, b1: u64) -> u64 {
    let c0 = (r0 << 11) | (g0 << 5) | b0;
    let c1 = (r1 << 11) | (g1 << 5) | b1;

    // Use each unique 2-bit value for the 4x4 indices.
    let indices = bit_indices(4 * 4, 2);

    (indices << 32) | (c1 << 16) | c0
}

fn bc2_r() -> Surface<Vec<u8>> {
    // 5-bit independent R channel for BC2 end points.
    bcn(5, image_dds::ImageFormat::BC2RgbaUnorm, |i, j| {
        bc2_block(i, 0, 0, j, 0, 0).to_le_bytes()
    })
}

fn bc2_b() -> Surface<Vec<u8>> {
    // 5-bit independent B channel for BC2 end points.
    bcn(5, image_dds::ImageFormat::BC2RgbaUnorm, |i, j| {
        bc2_block(0, 0, i, 0, 0, j).to_le_bytes()
    })
}

fn bc2_g() -> Surface<Vec<u8>> {
    // 6-bit independent G channel for BC2 end points.
    bcn(6, image_dds::ImageFormat::BC2RgbaUnorm, |i, j| {
        bc2_block(0, i, 0, 0, j, 0).to_le_bytes()
    })
}

fn bc2_block(r0: u64, g0: u64, b0: u64, r1: u64, g1: u64, b1: u64) -> u128 {
    // Generate each unique 4-bit alpha value.
    // These conveniently fit in a single block.
    let alpha_block = bit_indices(4 * 4, 4);

    // BC2 combines a BC1 RGB block with a separate alpha block.
    ((bc1_block(r0, g0, b0, r1, g1, b1) as u128) << 64) | alpha_block as u128
}

fn bc3_r() -> Surface<Vec<u8>> {
    // 5-bit independent R channel for BC3 end points.
    // 8-bit alpha end points require more pixels.
    bcn(8, image_dds::ImageFormat::BC3RgbaUnorm, |i, j| {
        bc3_block(i % 32, 0, 0, i, j % 32, 0, 0, j).to_le_bytes()
    })
}

fn bc3_b() -> Surface<Vec<u8>> {
    // 5-bit independent B channel for BC3 end points.
    // 8-bit alpha end points require more pixels.
    bcn(8, image_dds::ImageFormat::BC3RgbaUnorm, |i, j| {
        bc3_block(0, 0, i % 32, i, 0, 0, j % 32, j).to_le_bytes()
    })
}

fn bc3_g() -> Surface<Vec<u8>> {
    // 6-bit G and independent G channel for BC3 end points.
    // 8-bit alpha end points require more pixels.
    bcn(8, image_dds::ImageFormat::BC3RgbaUnorm, |i, j| {
        bc3_block(0, i % 64, 0, i, 0, j % 64, 0, j).to_le_bytes()
    })
}

fn bc3_block(r0: u64, g0: u64, b0: u64, a0: u64, r1: u64, g1: u64, b1: u64, a1: u64) -> u128 {
    let alpha_block = smooth_alpha_block(a0, a1);

    // BC3 combines a BC1 RGB block with a separate alpha block.
    ((bc1_block(r0, g0, b0, r1, g1, b1) as u128) << 64) | alpha_block as u128
}

fn smooth_alpha_block(a0: u64, a1: u64) -> u64 {
    // Use each unique 3-bit value for the 4x4 indices.
    let indices = bit_indices(4 * 4, 3);

    let alpha_block = (indices << 16) | (a1 << 8) | a0;
    alpha_block
}

fn bc4_r() -> Surface<Vec<u8>> {
    // 8-bit independent R channel for BC4 end points.
    bcn(8, image_dds::ImageFormat::BC4RUnorm, |i, j| {
        bc4_block(i, j).to_le_bytes()
    })
}

fn bc4_block(r0: u64, r1: u64) -> u64 {
    // BC4 is just a BC3 "alpha" block.
    smooth_alpha_block(r0, r1)
}

fn bc5_r() -> Surface<Vec<u8>> {
    // 8-bit independent R channel for BC5 end points.
    bcn(8, image_dds::ImageFormat::BC5RgUnorm, |i, j| {
        bc5_block_r(i, j).to_le_bytes()
    })
}

// TODO: helper functions for block 8 and block 16?
fn bc5_g() -> Surface<Vec<u8>> {
    // 8-bit independent G channel for BC5 end points.
    bcn(8, image_dds::ImageFormat::BC5RgUnorm, |i, j| {
        bc5_block_g(i, j).to_le_bytes()
    })
}

fn bc5_block_r(r0: u64, r1: u64) -> u128 {
    // BC5 is just two BC3 "alpha" blocks.
    smooth_alpha_block(r0, r1) as u128
}

fn bc5_block_g(g0: u64, g1: u64) -> u128 {
    // BC5 is just two BC3 "alpha" blocks.
    (smooth_alpha_block(g0, g1) as u128) << 64
}

fn bcn<const N: usize, F>(
    bits: u64,
    image_format: image_dds::ImageFormat,
    block: F,
) -> Surface<Vec<u8>>
where
    F: Fn(u64, u64) -> [u8; N],
{
    let blocks = 1 << bits;

    let mut data = Vec::new();
    for i in 0..blocks {
        for j in 0..blocks {
            let bytes = block(i, j);
            data.extend_from_slice(&bytes);
        }
    }

    Surface {
        width: blocks as u32 * 4,
        height: blocks as u32 * 4,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format,
        data,
    }
}

fn bit_indices(count: u64, bits: u64) -> u64 {
    // Repeat unique bit patterns for count.
    let mut indices = 0;
    for i in 0..count {
        indices |= (i % (1 << bits)) << (i * bits);
    }
    indices
}
