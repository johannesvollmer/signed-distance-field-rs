
fn main(){
    use signed_distance_field::prelude::*;

    // load data using piston image
    let mut gray_image = image::open("images/sketch.jpg").unwrap().to_luma();
    image::imageops::colorops::invert(&mut gray_image);

    // interpret grayscale image as binary image
    let binary_image = binary_piston_image::of_gray_image_with_threshold(&gray_image, 80);

    // convert binary image to distance field
    let distance_field = compute_f32_distance_field(&binary_image);

    // clip all distances greater than 10px and compress them into a byte array
    // so that a distance of -10px is 0 and a distance of 10px is 255
    // (edges, having a distance of 0px, will be 128)
    let distance_image = distance_field
        .normalize_clamped_distances(-10.0, 10.0)
        .unwrap().to_gray_u8_image(); // convert to piston image

    // save the piston image as png
    distance_image.save("images/sketch_distance.png").unwrap();
}
