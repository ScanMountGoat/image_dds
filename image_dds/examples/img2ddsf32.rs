use std::str::FromStr;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba32f();

    let format_string = args.get(3).unwrap_or(&String::new()).to_string();
    let format = image_dds::ImageFormat::from_str(&format_string).unwrap();

    let start = std::time::Instant::now();
    let dds = match args.get(4).map(|s| s.as_str()) {
        Some("layers") => {
            // Assume a square image.
            image_dds::SurfaceRgba32Float::from_image_layers(&image, image.height() / image.width())
                .encode(
                    format,
                    image_dds::Quality::Fast,
                    image_dds::Mipmaps::GeneratedAutomatic,
                )
                .unwrap()
                .to_dds()
                .unwrap()
        }
        Some("depth") => {
            // Assume a square image.
            image_dds::SurfaceRgba32Float::from_image_depth(&image, image.height() / image.width())
                .encode(
                    format,
                    image_dds::Quality::Fast,
                    image_dds::Mipmaps::GeneratedAutomatic,
                )
                .unwrap()
                .to_dds()
                .unwrap()
        }
        _ => image_dds::dds_from_imagef32(
            &image,
            format,
            image_dds::Quality::Fast,
            image_dds::Mipmaps::GeneratedAutomatic,
        )
        .unwrap(),
    };

    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
