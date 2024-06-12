use std::io::BufWriter;

use image_dds::Surface;

fn main() {
    // TODO: Check that these are generating correct blocks.
    // TODO: args to select the format.
    // Test data is based on the following blogpost:
    // https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
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

    // TODO: BC4, BC5

    // TODO: How to handle BC6 and BC7?
}

fn save_dds(dds: ddsfile::Dds, path: &str) {
    let mut writer = BufWriter::new(std::fs::File::create(path).unwrap());
    dds.write(&mut writer).unwrap();
}

// TODO: reduce repetition
fn bc1_r() -> Surface<Vec<u8>> {
    // 5 bit independent R channel for BC1 end points.
    let mut data = Vec::new();
    for i in 0..32 {
        for j in 0..32 {
            let block = bc1_block(i, 0, 0, j, 0, 0);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 128,
        height: 128,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC1RgbaUnorm,
        data,
    }
}

fn bc1_b() -> Surface<Vec<u8>> {
    // 5 bit independent B channel for BC1 end points.
    let mut data = Vec::new();
    for i in 0..32 {
        for j in 0..32 {
            let block = bc1_block(0, 0, i, 0, 0, j);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 128,
        height: 128,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC1RgbaUnorm,
        data,
    }
}

fn bc1_g() -> Surface<Vec<u8>> {
    let mut data = Vec::new();
    // 6 bit independent G channel for BC1 end points.
    for i in 0..64 {
        for j in 0..64 {
            let block = bc1_block(0, i, 0, 0, j, 0);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 256,
        height: 256,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC1RgbaUnorm,
        data,
    }
}

fn bc1_block(r0: u64, g0: u64, b0: u64, r1: u64, g1: u64, b1: u64) -> u64 {
    let c0 = (r0 << 11) | (g0 << 5) | b0;
    let c1 = (r1 << 11) | (g1 << 5) | b1;

    // Use each unique 2 bit value for the 4x4 indices.
    let indices = bit_indices(4 * 4, 2);

    (indices << 32) | (c1 << 16) | c0
}

fn bc2_r() -> Surface<Vec<u8>> {
    // 5 bit independent R channel for BC2 end points.
    let mut data = Vec::new();
    for i in 0..32 {
        for j in 0..32 {
            let block = bc2_block(i, 0, 0, j, 0, 0);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 128,
        height: 128,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC2RgbaUnorm,
        data,
    }
}

fn bc2_b() -> Surface<Vec<u8>> {
    // 5 bit independent B channel for BC2 end points.
    let mut data = Vec::new();
    for i in 0..32 {
        for j in 0..32 {
            let block = bc2_block(0, 0, j, 0, 0, i);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 128,
        height: 128,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC2RgbaUnorm,
        data,
    }
}

fn bc2_g() -> Surface<Vec<u8>> {
    let mut data = Vec::new();
    // 6 bit independent G channel for BC2 end points.
    for i in 0..64 {
        for j in 0..64 {
            let block = bc2_block(0, i, 0, 0, j, 0);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 256,
        height: 256,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC2RgbaUnorm,
        data,
    }
}

fn bc2_block(r0: u64, g0: u64, b0: u64, r1: u64, g1: u64, b1: u64) -> u128 {
    // Generate each unique 4 bit alpha value.
    // These conveniently fit in a single block.
    let alpha_block = bit_indices(4 * 4, 4);

    // BC2 combines a BC1 RGB block with a separate alpha block.
    ((bc1_block(r0, g0, b0, r1, g1, b1) as u128) << 64) | alpha_block as u128
}

fn bc3_r() -> Surface<Vec<u8>> {
    // 5 bit independent R channel for BC3 end points.
    // 8 bit alpha end points require more pixels.
    let mut data = Vec::new();
    for i in 0..256 {
        for j in 0..256 {
            let block = bc3_block(i % 32, 0, 0, i, j % 32, 0, 0, j);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 1024,
        height: 1024,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC3RgbaUnorm,
        data,
    }
}

fn bc3_b() -> Surface<Vec<u8>> {
    // 5 bit independent B channel for BC3 end points.
    // 8 bit alpha end points require more pixels.
    let mut data = Vec::new();
    for i in 0..256 {
        for j in 0..256 {
            let block = bc3_block(0, 0, i % 32, i, 0, 0, j % 32, j);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 1024,
        height: 1024,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC3RgbaUnorm,
        data,
    }
}

fn bc3_g() -> Surface<Vec<u8>> {
    let mut data = Vec::new();
    // 6 bit G and independent G channel for BC3 end points.
    // 8 bit alpha end points require more pixels.
    for i in 0..256 {
        for j in 0..256 {
            let block = bc3_block(0, i % 64, 0, i, 0, j % 64, 0, j);
            data.extend_from_slice(&block.to_le_bytes());
        }
    }
    Surface {
        width: 1024,
        height: 1024,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: image_dds::ImageFormat::BC3RgbaUnorm,
        data,
    }
}

fn bc3_block(r0: u64, g0: u64, b0: u64, a0: u64, r1: u64, g1: u64, b1: u64, a1: u64) -> u128 {
    // Use each unique 3 bit value for the 4x4 indices.
    let indices = bit_indices(4 * 4, 3);

    let alpha_block = (indices << 16) | (a1 << 8) | a0;

    // BC3 combines a BC1 RGB block with a separate alpha block.
    ((bc1_block(r0, g0, b0, r1, g1, b1) as u128) << 64) | alpha_block as u128
}

fn bit_indices(count: u64, bits: u64) -> u64 {
    // Repeat unique bit patterns for count.
    let mut indices = 0;
    for i in 0..count {
        indices |= (i % (1 << bits)) << (i * bits);
    }
    indices
}
