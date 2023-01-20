fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba8();

    // Default to BC7 for good quality even at the lowest quality setting.
    let format = match args
        .get(3)
        .unwrap_or(&String::new())
        .to_lowercase()
        .as_str()
    {
        "bc1" => image_dds::ImageFormat::BC1Unorm,
        "bc2" => image_dds::ImageFormat::BC2Unorm,
        "bc3" => image_dds::ImageFormat::BC3Unorm,
        "bc4" => image_dds::ImageFormat::BC4Unorm,
        "bc5" => image_dds::ImageFormat::BC5Unorm,
        "bc6" => image_dds::ImageFormat::BC6Ufloat,
        "bc7" => image_dds::ImageFormat::BC7Unorm,
        _ => image_dds::ImageFormat::BC7Unorm,
    };

    let start = std::time::Instant::now();
    let dds = image_dds::dds_from_image(&image, format, image_dds::Quality::Fast, true).unwrap();
    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
