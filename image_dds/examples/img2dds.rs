fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba8();

    // Default to BC7 for good quality even at the lowest quality setting.
    // TODO: Derive strum instead?
    let format_string = args.get(3).unwrap_or(&String::new()).to_string();

    let format = match format_string.to_lowercase().as_str() {
        "bc1unorm" => image_dds::ImageFormat::BC1Unorm,
        "bc1srgb" => image_dds::ImageFormat::BC1Srgb,
        "bc2unorm" => image_dds::ImageFormat::BC2Unorm,
        "bc2srgb" => image_dds::ImageFormat::BC2Srgb,
        "bc3nnorm" => image_dds::ImageFormat::BC3Unorm,
        "bc3srgb" => image_dds::ImageFormat::BC3Srgb,
        "bc4unorm" => image_dds::ImageFormat::BC4Unorm,
        "bc4snorm" => image_dds::ImageFormat::BC4Snorm,
        "bc5unorm" => image_dds::ImageFormat::BC5Unorm,
        "bc5snorm" => image_dds::ImageFormat::BC5Snorm,
        "bc6ufloat" => image_dds::ImageFormat::BC6Ufloat,
        "bc6sfloat" => image_dds::ImageFormat::BC6Sfloat,
        "bc7unorm" => image_dds::ImageFormat::BC7Unorm,
        "bc7srgb" => image_dds::ImageFormat::BC7Srgb,
        "bc2" => image_dds::ImageFormat::BC2Unorm,
        "bc3" => image_dds::ImageFormat::BC3Unorm,
        "bc4" => image_dds::ImageFormat::BC4Unorm,
        "bc5" => image_dds::ImageFormat::BC5Unorm,
        "bc6" => image_dds::ImageFormat::BC6Ufloat,
        "bc7" => image_dds::ImageFormat::BC7Unorm,
        "r8unorm" => image_dds::ImageFormat::R8Unorm,
        "r8g8b8a8unorm" => image_dds::ImageFormat::R8G8B8A8Unorm,
        "r8g8b8a8srgb" => image_dds::ImageFormat::R8G8B8A8Srgb,
        "r32g32b32a32float" => image_dds::ImageFormat::R32G32B32A32Float,
        "b8g8r8a8unorm" => image_dds::ImageFormat::B8G8R8A8Unorm,
        "b8g8r8a8srgb" => image_dds::ImageFormat::B8G8R8A8Srgb,
        _ => panic!("Unrecognized format string {format_string}"),
    };

    let start = std::time::Instant::now();
    let dds = image_dds::dds_from_image(&image, format, image_dds::Quality::Fast, true).unwrap();
    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
