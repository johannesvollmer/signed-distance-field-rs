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

    #[cfg(piston_image)]
    pub use crate::binary_image::piston_image::{
        from_gray_u8_image, from_gray_u8_image_with_threshold
    };
}


use prelude::*;

pub fn compute_distance_field<D: DistanceStorage>(image: &impl BinaryImage) -> SignedDistanceField<D> {
    SignedDistanceField::compute(image)
}

pub fn compute_f16_distance_field(image: &impl BinaryImage) -> SignedDistanceField<F16DistanceStorage> {
    compute_distance_field(image)
}

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
        reconstruct_binary_image::<F16DistanceStorage, _>(
            2048, 2048, 0.05, is_inside_circle(128, 128, 64)
        );
    }

    #[test]
    pub fn reconstruct_dot(){ // TODO profile this
        reconstruct_binary_image::<F16DistanceStorage, _>(
            2048, 2048, 0.05, is_inside_circle(1024, 1024, 4)
        );
    }

    #[test]
    pub fn reconstruct_rectangle(){
        reconstruct_binary_image::<F32DistanceStorage, _>(
            2048, 2048, 0.05, is_inside_rectangle(179, 179, 37, 37)
        );
    }

    #[test]
    pub fn reconstruct_stripes(){
        reconstruct_binary_image::<F32DistanceStorage, _>(
            2048, 2048, 0.07, is_inside_checker(179, 37)
        );
    }


    fn reconstruct_binary_image<D: DistanceStorage, I: Fn(usize, usize) -> bool>(
        width: usize, height: usize, tolerance: f32, image: I
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

        let distance_field: SignedDistanceField<D> = crate::compute_distance_field(&binary_image);

        let mut wrong_pixels = 0;
        for y in 0..height as u16 {
            for x in 0..width as u16 {
                let ground_truth = binary_image.is_inside(x, y);
                let reconstructed = distance_field.get_distance(x, y) < 0.0;
                if ground_truth != reconstructed {
                    wrong_pixels += 1;
                }
            }
        }

        let quality = wrong_pixels as f32 / (width as f32 * height as f32);
        println!("wrong pixels: {} of {} ({})", wrong_pixels, width * height, quality);
        assert!(quality < tolerance, "too many incorrect pixels");
    }





    fn circle_distance(center_x: usize, center_y: usize, radius: usize) -> impl Fn(usize, usize) -> f32 {
        move |x,y|{
            let x = (x as isize - center_x as isize) as f32;
            let y = (y as isize - center_y as isize) as f32;
            (x * x + y * y).sqrt() - radius as f32
        }
    }

    fn rectangle_distance(center_x: usize, center_y: usize, width: usize, height: usize) -> impl Fn(usize, usize) -> f32 {
        move |x,y|{
            let x = x as f32 - center_x as f32;
            let y = y as f32 - center_y as f32;
            let x = x.abs() - width as f32;
            let y = y.abs() - height as f32;
            x.max(y)
        }
    }

    #[test]
    pub fn reconstruct_circle_distance_field(){
        reconstruct_distance_field::<F16DistanceStorage, _>(
            2048, 2048, 2.0, circle_distance(128, 128, 64)
        );
    }

    #[test]
    pub fn reconstruct_rectangle_distance_field(){
        reconstruct_distance_field::<F16DistanceStorage, _>(
            2048, 2048, 130.0, // FIXME an error of 130.0 pixeldistance per pixel is a bug
            rectangle_distance(179, 179, 137, 137)
        );
    }

    pub fn reconstruct_distance_field<D: DistanceStorage, I: Fn(usize, usize) -> f32>(
        width: usize, height: usize, tolerance: f32, image: I
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

        let distance_field: SignedDistanceField<D> = crate::compute_distance_field(&binary_image);

        let mut summed_error = 0.0;
        for y in 0..height as u16 {
            for x in 0..width as u16 {
                let ground_truth = distance_buffer[y as usize * width + x as usize];
                let reconstructed = distance_field.get_distance(x, y);

                if reconstructed.is_infinite() {
                    panic!("infinite distance at {} {}", x, y);
                }

                summed_error += (ground_truth - reconstructed).abs();
            }
        }

        println!("{}", summed_error);

        let error_per_pixel = summed_error / (width as f32 * height as f32);
        println!("average error per pixel: {}", error_per_pixel);

        assert!(error_per_pixel < tolerance, "too many incorrect pixels");
    }

}
