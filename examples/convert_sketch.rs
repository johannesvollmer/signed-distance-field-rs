
fn main(){
    use signed_distance_field::prelude::*;

    let mut gray_image = image::open("images/sketch.jpg").unwrap().to_luma();
    image::imageops::colorops::invert(&mut gray_image);

    let binary_image = binary_piston_image::of_gray_u8_image_with_threshold(&gray_image, 80);

    let distance_field = compute_f32_distance_field(&binary_image);
    let distance_image = distance_field
        .normalize_clamped_distances(-10.0, 10.0)
        .unwrap().to_gray_u8_image();

    distance_image.save("images/sketch_distance.png").unwrap();
}
