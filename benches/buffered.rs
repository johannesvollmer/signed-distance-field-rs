
use signed_distance_field::prelude::*;
use criterion::{ Criterion, criterion_group, criterion_main };

fn byte_image_from_function<I>(width: usize, height: usize, image: I) -> Vec<u8>
    where I: Fn(usize, usize) -> bool
{
    let mut image_bytes = vec![0_u8; width * height];

    for y in 0..height {
        for x in 0..width {
            image_bytes[y * width + x] = {
                if image(x, y) { 255 } else { 0 }
            };
        }
    }

    image_bytes
}

fn circle(center_x: usize, center_y: usize, radius: usize)
    -> impl (Fn(usize, usize) -> bool)
{
    move |x, y|{
        let x = x as f32 - center_x as f32;
        let y = y as f32 - center_y as f32;
        (x * x + y * y).sqrt() < radius as f32
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dot", |bencher| {
        let width = 1080;
        let height = 1920;

        let image = byte_image_from_function(
            width, height, circle(width/2, height/2, 6)
        );

        bencher.iter(||{
            let binary = BinaryByteImage::from_slice(width as u16, height as u16, &image);
            let sdf: SignedDistanceField<F32DistanceStorage> = compute_distance_field(&binary);
            sdf
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);