use std::io::BufWriter;

use image_dds::Surface;

fn main() {
    // TODO: Check that these are generating correct blocks.
    // TODO: args to select the format.
    // Test data is based on the following blogpost:
    // https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
    // TODO: Combine these into a single image?
    let surface = bc1_r_test();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_r.dds");

    let surface = bc1_g_test();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_g.dds");

    let surface = bc1_b_test();
    let dds = surface.to_dds().unwrap();
    save_dds(dds, "bc1_b.dds");

    // TODO: BC2, BC3, BC4, BC5
    // TODO: How to handle BC6 and BC7?
}

fn save_dds(dds: ddsfile::Dds, path: &str) {
    let mut writer = BufWriter::new(std::fs::File::create(path).unwrap());
    dds.write(&mut writer).unwrap();
}

fn bc1_r_test() -> Surface<Vec<u8>> {
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

fn bc1_b_test() -> Surface<Vec<u8>> {
    // 5 bit independent B channel for BC1 end points.
    let mut data = Vec::new();
    for i in 0..32 {
        for j in 0..32 {
            let block = bc1_block(0, 0, j, 0, 0, i);
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

fn bc1_g_test() -> Surface<Vec<u8>> {
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
    let indices = 0b00011011000110110001101100011011;

    let block: u64 = (indices << 32) | (c1 << 16) | c0;
    block
}
