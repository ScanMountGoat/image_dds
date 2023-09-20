fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba32f();

    let start = std::time::Instant::now();
    let dds = image_dds::dds_from_imagef32(
        &image,
        image_dds::ImageFormat::BC6Ufloat,
        image_dds::Quality::Fast,
        image_dds::Mipmaps::GeneratedAutomatic,
    )
    .unwrap();

    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
