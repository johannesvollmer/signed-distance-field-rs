
fn main(){
    let width = 31;
    let height = 79;

    reconstruct_binary_image(512, 512, |x, y|{
        // this is a checkerboard pattern
        (x % width < width / 2) != (y % height < height / 2)
    });
}

fn reconstruct_binary_image(
    width: usize, height: usize,
    shape_function: impl Fn(usize, usize) -> bool
){
    use sdf_dead_reckoning::prelude::*;

    let mut binary_image = vec![0_u8; width * height];

    for y in 0..height {
        for x in 0..width {
            binary_image[width * y + x] = if shape_function(x, y) { 255 } else { 0 };
        }
    }

    let binary_image = BinaryByteImage::from_slice(width as u16, height as u16, &binary_image);
    let distance_field = compute_f32_distance_field(&binary_image);

    let mut imgbuf = image::ImageBuffer::new(512, 512);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let distance = distance_field.get_distance(x as u16, y as u16);

        *pixel = image::Rgb([
            0_u8, 0_u8,
            if distance < 0.0 { 255_u8 } else { 0_u8 }
        ]);
    }

    imgbuf.save("distance_field.png").unwrap();
}