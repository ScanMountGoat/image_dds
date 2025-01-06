use bytemuck::Pod;

use crate::{error::SurfaceError, mip_size};

use super::{Bc1, Bc2, Bc3, Bc4, Bc4S, Bc5, Bc5S, Bc6, Bc7, BLOCK_HEIGHT, BLOCK_WIDTH, CHANNELS};

pub trait BcnDecode<Pixel> {
    type CompressedBlock;

    // The decoded 4x4 pixel blocks are in row-major ordering.
    // Fixing the length should reduce the amount of bounds checking.
    fn decompress_block(block: &Self::CompressedBlock) -> [[Pixel; BLOCK_WIDTH]; BLOCK_HEIGHT];
}

// Allows block types to read and copy buffer data to enforce alignment.
pub trait ReadBlock {
    const SIZE_IN_BYTES: usize;

    fn read_block(data: &[u8], offset: usize) -> Self;
}

impl ReadBlock for [u8; 8] {
    const SIZE_IN_BYTES: usize = 8;

    fn read_block(data: &[u8], offset: usize) -> Self {
        data[offset..offset + 8].try_into().unwrap()
    }
}

impl ReadBlock for [u8; 16] {
    const SIZE_IN_BYTES: usize = 16;

    fn read_block(data: &[u8], offset: usize) -> Self {
        data[offset..offset + 16].try_into().unwrap()
    }
}

impl BcnDecode<[u8; 4]> for Bc1 {
    type CompressedBlock = [u8; 8];

    fn decompress_block(block: &[u8; 8]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc1(
            block,
            bytemuck::cast_slice_mut(&mut decompressed),
            BLOCK_WIDTH * CHANNELS,
        );

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc2 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc2(
            block,
            bytemuck::cast_slice_mut(&mut decompressed),
            BLOCK_WIDTH * CHANNELS,
        );

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc3 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc3(
            block,
            bytemuck::cast_slice_mut(&mut decompressed),
            BLOCK_WIDTH * CHANNELS,
        );

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc4 {
    type CompressedBlock = [u8; 8];

    fn decompress_block(block: &[u8; 8]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
        let mut decompressed_r = [[0u8; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc4(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_r),
            BLOCK_WIDTH,
            false,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_WIDTH {
                // It's a convention in some programs display BC4 in the red channel.
                // Use grayscale instead to avoid confusing it with colored data.
                // TODO: Match how channels handled when compressing RGBA data to BC4?
                let r = decompressed_r[y][x];
                decompressed[y][x] = [r, r, r, 255u8];
            }
        }

        decompressed
    }
}

fn snorm_to_unorm(x: u8) -> u8 {
    // Validated against decoding R8Snorm DDS with GPU and paint.net (DirectXTex).
    // TODO: Is this the optimal way to write this?
    if x < 128 {
        x + 128
    } else if x == 128 {
        0
    } else {
        x - 129
    }
}

impl BcnDecode<[u8; 4]> for Bc4S {
    type CompressedBlock = [u8; 8];

    fn decompress_block(block: &[u8; 8]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
        let mut decompressed_r = [[0; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc4(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_r),
            BLOCK_WIDTH,
            true,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_WIDTH {
                // It's a convention in some programs to display BC4 in the red channel.
                // Use grayscale instead to avoid confusing it with colored data.
                // TODO: Match how channels handled when compressing RGBA data to BC4?
                let r = snorm_to_unorm(decompressed_r[y][x]);
                decompressed[y][x] = [r, r, r, 255u8];
            }
        }

        decompressed
    }
}

impl BcnDecode<[f32; 4]> for Bc4S {
    type CompressedBlock = [u8; 8];

    fn decompress_block(block: &[u8; 8]) -> [[[f32; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
        let mut decompressed_r = [[0.0; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc4_float(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_r),
            BLOCK_WIDTH,
            true,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0.0; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_WIDTH {
                // It's a convention in some programs display BC4 in the red channel.
                // Use grayscale instead to avoid confusing it with colored data.
                // TODO: Match how channels handled when compressing RGBA data to BC4?
                let r = decompressed_r[y][x];
                decompressed[y][x] = [r, r, r, 1.0];
            }
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc5 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC5 stores RG data, so each decompressed pixel is 2 bytes.
        let mut decompressed_rg = [[[0u8; 2]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc5(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_rg),
            BLOCK_WIDTH * 2,
            false,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_HEIGHT {
                // It's convention to zero the blue channel when decompressing BC5.
                let [r, g] = decompressed_rg[y][x];
                decompressed[y][x] = [r, g, 0u8, 255u8];
            }
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc5S {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC5 stores RG data, so each decompressed pixel is 2 bytes.
        let mut decompressed_rg = [[[0u8; 2]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc5(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_rg),
            BLOCK_WIDTH * 2,
            true,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_HEIGHT {
                // It's convention to zero the blue channel when decompressing BC5.
                // TODO: Should the blue channel be different for signed BC5?
                let [r, g] = decompressed_rg[y][x].map(snorm_to_unorm);

                decompressed[y][x] = [r, g, 128u8, 255u8];
            }
        }

        decompressed
    }
}

impl BcnDecode<[f32; 4]> for Bc5S {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[f32; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC5 stores RG data, so each decompressed pixel is 2 bytes.
        let mut decompressed_rg = [[[0.0; 2]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc5_float(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_rg),
            BLOCK_WIDTH * 2,
            true,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0.0; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_HEIGHT {
                // It's convention to zero the blue channel when decompressing BC5.
                // TODO: Should the blue channel be different for signed BC5?
                let [r, g] = decompressed_rg[y][x];
                decompressed[y][x] = [r, g, 0.5, 1.0];
            }
        }

        decompressed
    }
}

impl BcnDecode<[f32; 4]> for Bc6 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[f32; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC6H uses half precision floating point data.
        // Convert to single precision since f32 is better supported on CPUs.
        let mut decompressed_rgb = [[[0.0; 3]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        // Cast the pointer to a less strictly aligned type.
        // The pitch is in terms of floats rather than bytes.
        bcdec_rs::bc6h_float(
            block,
            bytemuck::cast_slice_mut(&mut decompressed_rgb),
            BLOCK_WIDTH * 3,
            // TODO: signed vs unsigned?
            false,
        );

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0.0; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_HEIGHT {
                let [r, g, b] = decompressed_rgb[y][x];
                decompressed[y][x] = [r, g, b, 1.0];
            }
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc6 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let decompressed: [[[f32; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] = Bc6::decompress_block(block);

        // Truncate to clamp to 0 to 255.
        let float_to_u8 = |x: f32| (x * 255.0) as u8;
        decompressed.map(|row| row.map(|pixel| pixel.map(float_to_u8)))
    }
}

impl BcnDecode<[u8; 4]> for Bc7 {
    type CompressedBlock = [u8; 16];

    fn decompress_block(block: &[u8; 16]) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        bcdec_rs::bc7(
            block,
            bytemuck::cast_slice_mut(&mut decompressed),
            BLOCK_WIDTH * CHANNELS,
        );

        decompressed
    }
}

/// Decompress the bytes in `data` to the uncompressed RGBA8 format.
pub fn rgba_from_bcn<F, T>(width: u32, height: u32, data: &[u8]) -> Result<Vec<T>, SurfaceError>
where
    T: Copy + Default + Pod,
    F: BcnDecode<[T; 4]>,
    F::CompressedBlock: ReadBlock,
{
    // Validate surface dimensions to check for potential overflow.
    let expected_size = mip_size(
        width as usize,
        height as usize,
        1,
        BLOCK_WIDTH,
        BLOCK_HEIGHT,
        1,
        F::CompressedBlock::SIZE_IN_BYTES,
    )
    .ok_or(SurfaceError::PixelCountWouldOverflow {
        width,
        height,
        depth: 1,
    })?;

    // Mipmap dimensions do not need to be multiples of the block dimensions.
    // A mipmap of size 1x1 pixels can still be decoded.
    // Simply checking the data length is sufficient.
    if data.len() < expected_size {
        return Err(SurfaceError::NotEnoughData {
            expected: expected_size,
            actual: data.len(),
        });
    }

    let mut rgba = vec![T::default(); width as usize * height as usize * CHANNELS];

    // BCN formats lay out blocks in row-major order.
    // TODO: calculate x and y using division and mod?
    let mut block_start = 0;
    for y in (0..height).step_by(BLOCK_HEIGHT) {
        for x in (0..width).step_by(BLOCK_WIDTH) {
            // Use a special type to enforce alignment.
            let block = F::CompressedBlock::read_block(data, block_start);
            // TODO: Add rgba8 and rgbaf32 variants for decompress block.
            let decompressed_block = F::decompress_block(&block);

            // TODO: This can be generic over the pixel type to also support float.
            // Each block is 4x4, so we need to update multiple rows.
            put_rgba_block(
                &mut rgba,
                decompressed_block,
                x as usize,
                y as usize,
                width as usize,
                height as usize,
            );

            block_start += F::CompressedBlock::SIZE_IN_BYTES;
        }
    }

    Ok(rgba)
}

fn put_rgba_block<T: Pod>(
    surface: &mut [T],
    pixels: [[[T; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    // Place the compressed block into the decompressed surface.
    // The data from each block will update up to 4 rows of the RGBA surface.
    // Add checks since the edges won't always have full blocks.
    // TODO: potential overflow if x > width or y > height?
    let elements_per_row = CHANNELS * BLOCK_WIDTH.min(width - x);

    for (row, row_pixels) in pixels.iter().enumerate().take(BLOCK_HEIGHT.min(height - y)) {
        // Convert pixel coordinates to byte coordinates.
        let surface_index = ((y + row) * width + x) * CHANNELS;
        // The correct slice length is calculated above.
        // TODO: Is it really faster to use bytemuck?
        surface[surface_index..surface_index + elements_per_row]
            .copy_from_slice(&bytemuck::cast_slice(row_pixels)[..elements_per_row]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add decoding tests?

    #[test]
    fn put_rgba_block_4x4() {
        // Write an entire block.
        let mut surface = vec![0u8; 4 * 4 * 4];
        put_rgba_block(
            &mut surface,
            [[[1u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
            0,
            0,
            4,
            4,
        );
        assert_eq!(vec![1u8; 4 * 4 * 4], surface);
    }

    #[test]
    fn put_rgba_block_5x5() {
        // Test that block xy offsets work properly.
        let mut surface = vec![0u8; 5 * 5 * 4];

        put_rgba_block(
            &mut surface,
            [[[1u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
            0,
            0,
            5,
            5,
        );
        put_rgba_block(
            &mut surface,
            [[[2u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
            1,
            1,
            5,
            5,
        );

        assert_eq!(
            [
                [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
                [1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                [1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                [1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                [0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
            surface
        );
    }

    #[test]
    fn convert_snorm_to_unorm() {
        let expected = [
            128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144,
            145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161,
            162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178,
            179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195,
            196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212,
            213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229,
            230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246,
            247, 248, 249, 250, 251, 252, 253, 254, 255, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
            34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
            56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
            78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99,
            100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
            117, 118, 119, 120, 121, 122, 123, 124, 125, 126,
        ];
        for (input, output) in expected.into_iter().enumerate() {
            assert_eq!(snorm_to_unorm(input as u8), output);
        }
    }
}
