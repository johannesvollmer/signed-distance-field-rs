
fn main(){
    use signed_distance_field::prelude::*;

    // load data using piston image
    let mut gray_image = image::open("images/sketch.jpg").unwrap().to_luma();
    image::imageops::colorops::invert(&mut gray_image);

    // interpret grayscale image as binary image
    let binary_image = binary_piston_image::of_gray_image_with_threshold(&gray_image, 80);

    // convert binary image to distance field
    let distance_field = compute_f32_distance_field(&binary_image);

    // compress all distances between -10 and 10 into a byte array while clipping greater distances
    let distance_image = distance_field
        .normalize_clamped_distances(-10.0, 10.0)
        .unwrap().to_gray_u8_image(); // convert to piston image

    // save the piston image as png
    distance_image.save("images/sketch_distance.png").unwrap();
}
