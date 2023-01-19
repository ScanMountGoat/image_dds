fn main() {
    let args: Vec<_> = std::env::args().collect();

    let mut reader = std::fs::File::open(&args[1]).unwrap();
    let dds = ddsfile::Dds::read(&mut reader).unwrap();

    let start = std::time::Instant::now();
    let image = image_dds::image_from_dds(&dds).unwrap();
    println!("Decompressed data in {:?}", start.elapsed());

    image.save(&args[2]).unwrap();
}
