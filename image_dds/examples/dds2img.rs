fn main() {
    let args: Vec<_> = std::env::args().collect();

    let mut reader = std::fs::File::open(&args[1]).unwrap();
    let dds = ddsfile::Dds::read(&mut reader).unwrap();

    // TODO: Why do bc4 dds files not work with ddsfile?
    // TODO: BC4 and BC5 DDS files created with paint.net don't have their formats recognized?
    let start = std::time::Instant::now();
    let rgba = image_dds::bcn::rgba8_from_bcn(
        dds.get_width(),
        dds.get_height(),
        &dds.data,
        dds.get_dxgi_format().unwrap().try_into().unwrap(),
    )
    .unwrap();
    println!("Decompressed data in {:?}", start.elapsed());

    let image = image::RgbaImage::from_raw(dds.get_width(), dds.get_height(), rgba).unwrap();
    image.save(&args[2]).unwrap();
}
