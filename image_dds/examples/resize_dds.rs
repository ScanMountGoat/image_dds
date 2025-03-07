use image_dds::{Mipmaps, Quality, Surface, SurfaceRgba32Float};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let mut reader = std::fs::File::open(&args[1]).unwrap();
    let dds = ddsfile::Dds::read(&mut reader).unwrap();

    let width = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(dds.get_width());
    let height = args
        .get(4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(dds.get_height());
    let depth = args
        .get(5)
        .and_then(|s| s.parse().ok())
        .unwrap_or(dds.get_depth());

    let start = std::time::Instant::now();

    let surface = Surface::from_dds(&dds).unwrap();

    // Resize potentially compressed data by decoding, resizing, and encoding.
    let decoded = surface.decode_rgbaf32().unwrap();

    // Collect combined surface data in the expected layout.
    let mut data = Vec::new();

    // It's possible to reuse existing mipmaps when downscaling by a factor of 2.
    // Use only the base mip and regenerate mipmaps to support arbitrary dimensions.
    for layer in 0..surface.layers {
        for level in 0..surface.depth {
            let image = decoded.get_image(layer, level, 0).unwrap();
            let resized_image = image::imageops::resize(
                &image,
                width,
                height,
                image::imageops::FilterType::Triangle,
            );
            data.extend_from_slice(resized_image.as_raw());
        }
    }

    let resized_surface = SurfaceRgba32Float {
        width,
        height,
        depth,
        layers: surface.layers,
        mipmaps: 1,
        data,
    };

    let resized_dds = resized_surface
        .encode(
            surface.image_format,
            Quality::Fast,
            if surface.mipmaps > 1 {
                Mipmaps::GeneratedAutomatic
            } else {
                Mipmaps::Disabled
            },
        )
        .unwrap()
        .to_dds()
        .unwrap();

    println!(
        "Resized from {}x{}x{} to {width}x{height}x{depth} in {:?}",
        surface.width,
        surface.height,
        surface.depth,
        start.elapsed()
    );

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    resized_dds.write(&mut writer).unwrap();
}
