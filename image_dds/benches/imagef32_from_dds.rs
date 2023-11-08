use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image_dds::{imagef32_from_dds, ImageFormat, Surface};

fn criterion_benchmark(c: &mut Criterion) {
    // Overestimate the surface size to avoid errors.
    let surface = Surface {
        width: 512,
        height: 512,
        depth: 1,
        layers: 1,
        mipmaps: 1,
        image_format: ImageFormat::BC7Unorm,
        data: vec![0u8; 512 * 512 * 2],
    };
    let dds = surface.to_dds().unwrap();
    c.bench_function("imagef32_from_dds", |b| {
        b.iter(|| imagef32_from_dds(black_box(&dds), 0))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
