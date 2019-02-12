#![feature(test)]
extern crate test;


#[cfg(test)]
mod benches {
    use signed_distance_field::prelude::*;
    use test::Bencher;

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

    fn compute_various<D>(bencher: &mut Bencher) where D: DistanceStorage {
        let width = 1080;
        let height = 1920;

        for sdf in &[
            circle(width/2, height/2, 6),
            circle(width/2, height/2, height/3),
            circle(0, 0, 35),
            circle(width, 0, 35),
        ] {
            let image_buffer = byte_image_from_function(width, height, sdf);
            let binary = binary_image::of_byte_slice(&image_buffer, width as u16, height as u16);

            bencher.iter(|| SignedDistanceField::<D>::compute(&binary));
        }
    }

    fn compute_highres<D>(bencher: &mut Bencher) where D: DistanceStorage {
        let width = 4096;
        let height = 4096;

        let image_buffer = byte_image_from_function(width, height, circle(width/2, height/2, 6));
        let binary = binary_image::of_byte_slice(&image_buffer, width as u16, height as u16);
        bencher.iter(|| SignedDistanceField::<D>::compute(&binary));
    }


    #[bench]
    fn bench_various_f16(bencher: &mut Bencher) {
        compute_various::<F16DistanceStorage>(bencher);
    }

    #[bench]
    fn bench_various_f32(bencher: &mut Bencher) {
        compute_various::<F32DistanceStorage>(bencher);
    }

    #[bench]
    fn bench_highres_f16(bencher: &mut Bencher) {
        compute_highres::<F16DistanceStorage>(bencher);
    }

    #[bench]
    fn bench_highres_f32(bencher: &mut Bencher) {
        compute_highres::<F32DistanceStorage>(bencher);
    }

}

