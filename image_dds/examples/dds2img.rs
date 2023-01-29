fn main() {
    let args: Vec<_> = std::env::args().collect();

    let mipmap = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut reader = std::fs::File::open(&args[1]).unwrap();
    let dds = ddsfile::Dds::read(&mut reader).unwrap();

    let start = std::time::Instant::now();
    let image = image_dds::image_from_dds(&dds, mipmap).unwrap();
    println!("Decompressed data in {:?}", start.elapsed());

    image.save(&args[2]).unwrap();
}
