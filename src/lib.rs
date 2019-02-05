//! This crate approximates a signed
//! distance field, given a binary image.
//! The algorithm used is called "dead reckoning",
//! as described in `The "dead reckoning" signed distance transform`
//! by George J. Grevara (2004).

pub mod binary_image;
pub mod distance_field;

pub mod prelude {
    pub use crate::{
        compute_distance_field,
        compute_f16_distance_field,
        compute_f32_distance_field
    };

    pub use crate::binary_image::{
        BinaryImage, BinaryByteImage
    };

    pub use crate::distance_field::{
        SignedDistanceField, DistanceStorage,
        F16DistanceStorage, F32DistanceStorage
    };

    #[cfg(feature = "piston_image")]
    pub use crate::binary_image::piston_image as binary_piston_image;
}


use prelude::*;

/// Compute the signed distance field with the specified distance storage of the specified binary image.
pub fn compute_distance_field<D: DistanceStorage>(image: &impl BinaryImage) -> SignedDistanceField<D> {
    SignedDistanceField::compute(image)
}

/// Compute the signed distance field with an `f16` distance storage of the specified binary image.
pub fn compute_f16_distance_field(image: &impl BinaryImage) -> SignedDistanceField<F16DistanceStorage> {
    compute_distance_field(image)
}

/// Compute the signed distance field with an `f32` distance storage of the specified binary image.
pub fn compute_f32_distance_field(image: &impl BinaryImage) -> SignedDistanceField<F32DistanceStorage> {
    compute_distance_field(image)
}


#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn is_inside_circle(center_x: usize, center_y: usize, radius: usize) -> impl Fn(usize, usize) -> bool {
        move |x,y|{
            let x = (x as isize - center_x as isize) as f32;
            let y = (y as isize - center_y as isize) as f32;
            (x * x + y * y).sqrt() < radius as f32
        }
    }

    fn is_inside_rectangle(center_x: usize, center_y: usize, width: usize, height: usize) -> impl Fn(usize, usize) -> bool {
        let w = width as f32;
        let h = height as f32;

        move |x,y|{
            let x = x as f32 - center_x as f32;
            let y = y as f32 - center_y as f32;
            x > -w && x < w && y > -h && y < h
        }
    }

    fn is_inside_checker(width: usize, height: usize) -> impl Fn(usize, usize) -> bool {
        move |x,y|{
            (x % width < width / 2) != (y % height < height / 2)
        }
    }

    #[test]
    pub fn reconstruct_circle(){
        reconstruct_binary_image(2048, 2048, 0.05, is_inside_circle(128, 128, 64));
    }

    #[test]
    pub fn reconstruct_dot(){
        reconstruct_binary_image(2048, 2048, 0.05, is_inside_circle(1024, 1024, 4));
    }

    #[test]
    pub fn reconstruct_top_left(){
        reconstruct_binary_image(2048, 2048, 0.05, is_inside_circle(0, 0, 14));
    }

    #[test]
    pub fn reconstruct_top_right(){
        reconstruct_binary_image(2048, 2048, 0.05, is_inside_circle(2048, 0, 14));
    }

    #[test]
    pub fn reconstruct_rectangle(){
        reconstruct_binary_image(2048, 2048, 0.05, is_inside_rectangle(179, 179, 37, 37));
    }

    #[test]
    pub fn reconstruct_stripes(){
        reconstruct_binary_image(2048, 2048, 0.07, is_inside_checker(179, 37));
    }


    fn reconstruct_binary_image(
        width: usize, height: usize, tolerance: f32,
        image: impl Fn(usize, usize) -> bool
    ) {
        let mut binary_image_buffer = vec![0_u8; width * height];

        for y in 0..height {
            for x in 0..width {
                binary_image_buffer[width * y + x] = if image(x, y) { 255 } else { 0 };
            }
        }

        let binary_image = BinaryByteImage::from_slice(
            width as u16, height as u16, &binary_image_buffer
        );

        let distance_field_16 = SignedDistanceField::<F16DistanceStorage>::compute(&binary_image);
        let distance_field_32 = SignedDistanceField::<F32DistanceStorage>::compute(&binary_image);

        let mut wrong_pixels = 0;
        for y in 0..height as u16 {
            for x in 0..width as u16 {
                let ground_truth = binary_image.is_inside(x, y);
                let distance_16 = distance_field_16.get_distance(x, y);
                let distance_32 = distance_field_32.get_distance(x, y);
                let distance = (distance_16 + distance_32) / 2.0;

                if distance.is_infinite() {
                    panic!("no shape in binary image");
                }

                let reconstructed = distance < 0.0;
                if ground_truth != reconstructed {
                    wrong_pixels += 1;
                }
            }
        }

        let quality = wrong_pixels as f32 / (width as f32 * height as f32);
        println!("wrong pixels: {} of {} ({})", wrong_pixels, width * height, quality);
        assert!(quality < tolerance, "too many incorrect pixels");
    }





    fn circle_distance(center_x: usize, center_y: usize, radius: usize)
        -> impl Fn(usize, usize) -> f32
    {
        move |x,y|{
            let x = (x as isize - center_x as isize) as f32;
            let y = (y as isize - center_y as isize) as f32;
            (x * x + y * y).sqrt() - radius as f32
        }
    }

    fn rectangle_distance(center_x: usize, center_y: usize, width: usize, height: usize)
        -> impl Fn(usize, usize) -> f32
    {
        move |x,y|{
            let x = x as f32 - center_x as f32;
            let y = y as f32 - center_y as f32;
            let x = x.abs() - width as f32;
            let y = y.abs() - height as f32;
            x.min(y)
        }
    }

    #[test]
    pub fn reconstruct_circle_distance_field(){
        reconstruct_distance_field(2048, 2048, 2.0, circle_distance(128, 128, 128));
    }

    #[test]
    pub fn reconstruct_dot_distance_field(){
        reconstruct_distance_field(2048, 2048, 2.0, circle_distance(128, 128, 4));
    }

    #[test]
    pub fn reconstruct_rectangle_distance_field(){
        reconstruct_distance_field(2048, 2048, 2.0, rectangle_distance(1023, 179, 137, 137));
    }

    #[test]
    pub fn reconstruct_large_rectangle_distance_field(){ // TODO reduce error further?
        reconstruct_distance_field(2048, 2048, 25.0, rectangle_distance(1024, 1023, 613, 673));
    }

    #[test]
    pub fn reconstruct_small_rectangle_distance_field(){
        reconstruct_distance_field(2048, 2048, 2.0, rectangle_distance(179, 1023, 4, 7));
    }

    pub fn reconstruct_distance_field(
        width: usize, height: usize, tolerance: f32,
        image: impl Fn(usize, usize) -> f32
    ) {
        let mut distance_buffer = vec![0.0; width * height];

        for y in 0..height {
            for x in 0..width {
                distance_buffer[width * y + x] = image(x, y);
            }
        }

        let binary_image_buffer: Vec<u8> = distance_buffer.iter()
            .map(|distance| if *distance < 0.0 { 255 } else { 0 })
            .collect();

        let binary_image = BinaryByteImage::from_slice(
            width as u16, height as u16, &binary_image_buffer
        );

        let distance_field_16 = SignedDistanceField::<F16DistanceStorage>::compute(&binary_image);
        let distance_field_32 = SignedDistanceField::<F32DistanceStorage>::compute(&binary_image);

        let mut summed_error = 0.0;
        for y in 0..height as u16 {
            for x in 0..width as u16 {
                let ground_truth = distance_buffer[y as usize * width + x as usize];
                let reconstructed_16 = distance_field_16.get_distance(x, y);
                let reconstructed_32 = distance_field_32.get_distance(x, y);
                let reconstructed = (reconstructed_16 + reconstructed_32) / 2.0;

                if reconstructed.is_infinite() {
                    panic!("no shape in binary image");
                }

                summed_error += (ground_truth - reconstructed).abs();
            }
        }

        let error_per_pixel = summed_error / (width as f32 * height as f32);
        println!("average error per pixel: {}", error_per_pixel);

        assert!(error_per_pixel < tolerance, "too many incorrect pixels");
    }
}
