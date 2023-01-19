fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba8();

    // BC7 gives good results even with the lowest quality setting.
    let format = match args
        .get(3)
        .unwrap_or(&String::new())
        .to_lowercase()
        .as_str()
    {
        "bc1" => image_dds::CompressionFormat::Bc1,
        "bc2" => image_dds::CompressionFormat::Bc2,
        "bc3" => image_dds::CompressionFormat::Bc3,
        "bc4" => image_dds::CompressionFormat::Bc4,
        "bc5" => image_dds::CompressionFormat::Bc5,
        "bc6" => image_dds::CompressionFormat::Bc6,
        "bc7" => image_dds::CompressionFormat::Bc7,
        _ => image_dds::CompressionFormat::Bc7,
    };

    let start = std::time::Instant::now();
    let dds = image_dds::dds_from_image(&image, format, image_dds::Quality::Fast).unwrap();
    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
