use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::RgbaImage;
use image_dds::{dds_from_image, ImageFormat, Mipmaps, Quality};

fn criterion_benchmark(c: &mut Criterion) {
    let image = RgbaImage::new(512, 512);
    c.bench_function("dds_from_image", |b| {
        b.iter(|| {
            dds_from_image(
                black_box(&image),
                black_box(ImageFormat::BC7RgbaUnorm),
                black_box(Quality::Fast),
                black_box(Mipmaps::GeneratedAutomatic),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
