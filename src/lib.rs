pub mod binary_image;
pub mod distance_field;


pub mod prelude {
    pub use crate::binary_image::BinaryImage;
    pub use crate::binary_image::BinaryByteImage;
    pub use crate::distance_field::SignedDistanceField;
}


#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn circle_sdf(center_x: usize, center_y: usize, radius: usize) -> impl Fn(usize, usize) -> f32 {
        move |x,y|{
            let x = (x as isize - center_x as isize) as f32;
            let y = (y as isize - center_y as isize) as f32;
            (x * x + y * y).sqrt() - radius as f32
        }
    }

    fn rectangle_sdf(center_x: usize, center_y: usize, width: usize, height: usize) -> impl Fn(usize, usize) -> f32 {
        move |x,y|{
            let x = x as f32 / width as f32 - center_x as f32;
            let y = y as f32 / height as f32 - center_y as f32;
            x.min(y)
        }
    }


    /// `distance_function`: Returns negated distance if inside a shape,
    /// otherwise simply distance to the shape.
    fn reconstruct_distance_function(
        width: usize, height: usize,
        distance_function: impl Fn(usize, usize) -> f32
    ) {
        let mut distance_image = vec![0.0; width * height];

        for y in 0..height {
            for x in 0..width {
                distance_image[width * y + x] = distance_function(x, y);
            }
        }

        let binary_image_buffer: Vec<_> = distance_image.iter()
            .map(|distance| if *distance < 0.0 { 255_u8 } else { 0_u8 })
            .collect();

        let binary_image = BinaryByteImage::from_slice(width as u16, height as u16, &binary_image_buffer);
        let distance_field = SignedDistanceField::compute_approximate(&binary_image);

        let different_pixels: f32 = distance_field.distances.iter()
            .map(|distance| if distance.to_f32() < 0.0 { 255_u8 } else { 0_u8 })
            .zip(binary_image_buffer.iter())
            .map(|(a, &b)| a as i32 - b as i32)
            .map(|difference| (difference as f32).abs())
            .sum();

        println!("incorrect: {} of {}", different_pixels, width * height);
        assert!(different_pixels / (width as f32 * height as f32) < 0.4, "too many incorrect pixels");
    }


    #[test]
    pub fn reconstruct_circle(){
        reconstruct_distance_function(2048*2, 2048*2, circle_sdf(128, 128, 64));
    }

    /*#[test]
    pub fn reconstruct_rectangle(){
        reconstruct_distance_function(2048*4, 2048*4, rectangle_sdf(179, 179, 37, 37));
    }*/
}
