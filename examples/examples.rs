

fn main(){
    let width = 31;
    let height = 31;
    let center_x = 197;
    let center_y = 197;

    generate_image_from_distance_function(512, 512, |x, y|{
        let x = x as f32 / width as f32 - center_x as f32;
        let y = y as f32 / height as f32 - center_y as f32;
        x.min(y)
    });
}

fn generate_image_from_distance_function(
    width: usize, height: usize,
    distance_function: impl Fn(usize, usize) -> f32
){
    use sdf_dead_reckoning::prelude::*;

    let mut distance_image = vec![0.0; width * height];

    for y in 0..height {
        for x in 0..width {
            distance_image[width * y + x] = distance_function(x, y);
        }
    }

    let binary_image: Vec<_> = distance_image.iter()
        .map(|distance| if *distance < 0.0 { 255_u8 } else { 0_u8 })
        .collect();

    let binary_image = BinaryByteImage::from_slice(width as u16, height as u16, &binary_image);
    let distance_field = SignedDistanceField::compute_approximate(&binary_image);

    let mut imgbuf = image::ImageBuffer::new(512, 512);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let difference = distance_image[(y * 512 + x) as usize]
            - distance_field.distances[(y * 512 + x) as usize].to_f32();

        *pixel = image::Rgb([
            0_u8, 0_u8, (difference.abs() * 0.05 * 255.0) as u8
        ]);
    }

    imgbuf.save("circle.png").unwrap();
}