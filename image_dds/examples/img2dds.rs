fn main() {
    let args: Vec<_> = std::env::args().collect();

    // Don't assume the image comes with an alpha channel.
    let image = image::open(&args[1]).unwrap().to_rgba8();

    // BC7 gives good results even with the lowest quality setting.
    let format = image_dds::CompressionFormat::Bc7;

    let mut dds = ddsfile::Dds::new_dxgi(ddsfile::NewDxgiParams {
        height: image.height(),
        width: image.width(),
        depth: None,
        format: format.into(),
        mipmap_levels: None,
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: ddsfile::D3D10ResourceDimension::Texture2D,
        alpha_mode: ddsfile::AlphaMode::Straight, // TODO: Does this matter?
    })
    .unwrap();

    let start = std::time::Instant::now();
    dds.data = image_dds::bcn::bcn_from_rgba8(
        image.width(),
        image.height(),
        image.as_raw(),
        format,
        image_dds::Quality::Fast,
    )
    .unwrap();
    println!("Compressed data in {:?}", start.elapsed());

    let mut writer = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    dds.write(&mut writer).unwrap();
}
