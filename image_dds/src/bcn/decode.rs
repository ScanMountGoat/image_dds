use crate::Rgba;
use crate::{error::DecompressSurfaceError, mip_size};

use super::{Bc1, Bc2, Bc3, Bc4, Bc5, Bc6, Bc7, BLOCK_HEIGHT, BLOCK_WIDTH};

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

// The underlying C/C++ code may cast the array pointer.
// Use a generous alignment to avoid alignment issues.
#[repr(align(8))]
pub struct Block8([u8; 8]);

impl ReadBlock for Block8 {
    const SIZE_IN_BYTES: usize = 8;

    fn read_block(data: &[u8], offset: usize) -> Self {
        Self(data[offset..offset + 8].try_into().unwrap())
    }
}

#[repr(align(16))]
pub struct Block16([u8; 16]);

impl ReadBlock for Block16 {
    const SIZE_IN_BYTES: usize = 16;

    fn read_block(data: &[u8], offset: usize) -> Self {
        Self(data[offset..offset + 16].try_into().unwrap())
    }
}

impl BcnDecode<[u8; 4]> for Bc1 {
    type CompressedBlock = Block8;

    fn decompress_block(block: &Block8) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc1(
                block.0.as_ptr(),
                decompressed.as_mut_ptr() as _,
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc2 {
    type CompressedBlock = Block16;

    fn decompress_block(block: &Block16) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc2(
                block.0.as_ptr(),
                decompressed.as_mut_ptr() as _,
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc3 {
    type CompressedBlock = Block16;

    fn decompress_block(block: &Block16) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc3(
                block.0.as_ptr(),
                decompressed.as_mut_ptr() as _,
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc4 {
    type CompressedBlock = Block8;

    fn decompress_block(block: &Block8) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC4 stores grayscale data, so each decompressed pixel is 1 byte.
        let mut decompressed_r = [[0u8; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc4(
                block.0.as_ptr(),
                decompressed_r.as_mut_ptr() as _,
                (BLOCK_WIDTH) as i32,
            );
        }

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

impl BcnDecode<[u8; 4]> for Bc5 {
    type CompressedBlock = Block16;

    fn decompress_block(block: &Block16) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // BC5 stores RG data, so each decompressed pixel is 2 bytes.
        let mut decompressed_rg = [[[0u8; 2]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc5(
                block.0.as_ptr(),
                decompressed_rg.as_mut_ptr() as _,
                (BLOCK_WIDTH * 2) as i32,
            );
        }

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

impl BcnDecode<[u8; 4]> for Bc6 {
    type CompressedBlock = Block16;

    fn decompress_block(block: &Block16) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        // TODO: signed vs unsigned?
        // TODO: Also support exr or radiance hdr under feature flags?
        // exr or radiance only make sense for bc6

        // BC6H uses half precision floating point data.
        // Convert to single precision since f32 is better supported on CPUs.
        let mut decompressed_rgb = [[[0f32; 3]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            // Cast the pointer to a less strictly aligned type.
            // The pitch is in terms of floats rather than bytes.
            bcndecode_sys::bcdec_bc6h_float(
                block.0.as_ptr(),
                decompressed_rgb.as_mut_ptr() as _,
                (BLOCK_WIDTH * 3) as i32,
                0,
            );
        }

        // Truncate to clamp to 0 to 255.
        // TODO: Add a separate function that returns floats?
        let float_to_u8 = |x: f32| (x * 255.0) as u8;

        // Pad to RGBA with alpha set to white.
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];
        for y in 0..BLOCK_HEIGHT {
            for x in 0..BLOCK_HEIGHT {
                // It's convention to zero the blue channel when decompressing BC5.
                let [r, g, b] = decompressed_rgb[y][x];
                decompressed[y][x] = [float_to_u8(r), float_to_u8(g), float_to_u8(b), 255u8];
            }
        }

        decompressed
    }
}

impl BcnDecode<[u8; 4]> for Bc7 {
    type CompressedBlock = Block16;

    fn decompress_block(block: &Block16) -> [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT] {
        let mut decompressed = [[[0u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT];

        unsafe {
            bcndecode_sys::bcdec_bc7(
                block.0.as_ptr(),
                decompressed.as_mut_ptr() as _,
                (BLOCK_WIDTH * Rgba::BYTES_PER_PIXEL) as i32,
            );
        }

        decompressed
    }
}

// TODO: Make this generic over the pixel type (f32 or u8).
/// Decompress the bytes in `data` to the uncompressed RGBA8 format.
pub fn rgba8_from_bcn<T: BcnDecode<[u8; 4]>>(
    width: u32,
    height: u32,
    depth: u32,
    data: &[u8],
) -> Result<Vec<u8>, DecompressSurfaceError>
where
    T::CompressedBlock: ReadBlock,
{
    // TODO: Add an option to parallelize this using rayon?
    // Each block can be decoded independently.

    // Surface dimensions are not validated yet and may cause overflow.

    let expected_size = mip_size(
        width as usize,
        height as usize,
        depth as usize,
        BLOCK_WIDTH,
        BLOCK_HEIGHT,
        1,
        T::CompressedBlock::SIZE_IN_BYTES,
    )
    .ok_or(DecompressSurfaceError::PixelCountWouldOverflow {
        width,
        height,
        depth,
    })?;

    // Mipmap dimensions do not need to be multiples of the block dimensions.
    // A mipmap of size 1x1 pixels can still be decoded.
    // Simply checking the data length is sufficient.
    if data.len() < expected_size {
        return Err(DecompressSurfaceError::NotEnoughData {
            expected: expected_size,
            actual: data.len(),
        });
    }

    let mut rgba =
        vec![0u8; width as usize * height as usize * depth as usize * Rgba::BYTES_PER_PIXEL];

    // BCN formats lay out blocks in row-major order.
    // TODO: calculate x and y using division and mod?
    // TODO: Add an outer loop for depth?
    let mut block_start = 0;
    for z in 0..depth {
        for y in (0..height).step_by(BLOCK_HEIGHT) {
            for x in (0..width).step_by(BLOCK_WIDTH) {
                // Use a special type to enforce alignment.
                let block = T::CompressedBlock::read_block(data, block_start);
                // TODO: Add rgba8 and rgbaf32 variants for decompress block.
                let decompressed_block = T::decompress_block(&block);

                // TODO: This can be generic over the pixel type to also support float.
                // Each block is 4x4, so we need to update multiple rows.
                put_rgba_block(
                    &mut rgba,
                    decompressed_block,
                    x as usize,
                    y as usize,
                    z as usize,
                    width as usize,
                    height as usize,
                );

                block_start += T::CompressedBlock::SIZE_IN_BYTES;
            }
        }
    }

    Ok(rgba)
}

fn put_rgba_block(
    surface: &mut [u8],
    pixels: [[[u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
    x: usize,
    y: usize,
    z: usize,
    width: usize,
    height: usize,
) {
    // Place the compressed block into the decompressed surface.
    // The data from each block will update up to 4 rows of the RGBA surface.
    // Add checks since the edges won't always have full blocks.
    // TODO: potential overflow if x > width or y > height?
    let bytes_per_row = std::mem::size_of::<[u8; 4]>() * BLOCK_WIDTH.min(width - x);

    for (row, row_pixels) in pixels.iter().enumerate().take(BLOCK_HEIGHT.min(height - y)) {
        // Convert pixel coordinates to byte coordinates.
        let surface_index = ((z * width * height) + (y + row) * width + x) * Rgba::BYTES_PER_PIXEL;
        // The correct slice length is calculated above.
        surface[surface_index..surface_index + bytes_per_row]
            .copy_from_slice(&bytemuck::cast_slice(row_pixels)[..bytes_per_row]);
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
            0,
            5,
            5,
        );
        put_rgba_block(
            &mut surface,
            [[[2u8; 4]; BLOCK_WIDTH]; BLOCK_HEIGHT],
            1,
            1,
            0,
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
