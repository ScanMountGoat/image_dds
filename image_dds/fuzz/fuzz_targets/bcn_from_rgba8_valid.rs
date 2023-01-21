#![no_main]

use libfuzzer_sys::fuzz_target;

use arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug)]
pub struct SurfaceInfo {
    width: u32,
    height: u32,
    format: image_dds::ImageFormat,
    data: Vec<u8>,
}

impl<'a> Arbitrary<'a> for SurfaceInfo {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let width = u.int_in_range(0..=512)?;
        let height = u.int_in_range(0..=512)?;
        let format = image_dds::ImageFormat::arbitrary(u)?;

        // Enforce a valid input length ahead of time.
        // This avoids constantly failing the length check.
        let mut data = vec![0u8; width as usize * height as usize * 4];
        u.fill_buffer(&mut data)?;

        Ok(Self {
            width,
            height,
            format,
            data,
        })
    }
}

fuzz_target!(|input: SurfaceInfo| {
    let SurfaceInfo {
        width,
        height,
        format,
        data,
    } = input;
    // TODO: Is it worth testing different quality settings?
    // Use fast for now so each iteration is faster.
    let quality = image_dds::Quality::Fast;
    let _bytes = image_dds::bcn::bcn_from_rgba8(width, height, &data, format.into(), quality);
});
