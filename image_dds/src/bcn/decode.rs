use bytemuck::Pod;

use crate::{error::SurfaceError, mip_size, snorm_to_unorm};

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
                let [r, g] = decompressed_rg[y][x];

                decompressed[y][x] = [
                    snorm_to_unorm(r),
                    snorm_to_unorm(g),
                    snorm_to_unorm(0u8),
                    255u8,
                ];
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
                // TODO: Is this the correct blue channel value?
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
pub fn decode_bcn<F, T>(width: u32, height: u32, data: &[u8]) -> Result<Vec<T>, SurfaceError>
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
}
