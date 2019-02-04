
pub trait BinaryImage {
    #[inline]
    fn width(&self) -> u16;

    #[inline]
    fn height(&self) -> u16;

    #[inline]
    fn is_inside(&self, x: u16, y: u16) -> bool;
}


pub struct BinaryByteImage<'b> {
    width: u16,
    height: u16,
    buffer: &'b [u8],
    threshold: u8,
}


impl<'b> BinaryByteImage<'b> {
    pub fn from_slice(width: u16, height: u16, buffer: &'b [u8]) -> Self {
        Self::from_slice_with_threshold(width, height, buffer, 127)
    }

    pub fn from_slice_with_threshold(width: u16, height: u16, buffer: &'b [u8], threshold: u8) -> Self {
        debug_assert_eq!(buffer.len(), width as usize * height as usize, "Buffer dimension mismatch");
        BinaryByteImage { width, height, buffer, threshold }
    }
}

impl BinaryImage for BinaryByteImage<'_> {
    #[inline]
    fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    fn is_inside(&self, x: u16, y: u16) -> bool {
        self.buffer[self.width as usize * y as usize + x as usize] > self.threshold
    }
}

#[cfg(feature = "piston_image")]
pub mod piston_image {
    use image::*;
    use super::BinaryImage;


    pub fn of_gray_u8_image(image: &GrayImage) -> WithThreshold<u8, Vec<u8>> {
        of_gray_u8_image_with_threshold(image, 127)
    }

    pub fn of_gray_u8_image_with_threshold(image: &GrayImage, threshold: u8)
        -> WithThreshold<u8, Vec<u8>>
    {
        WithThreshold::of(image, threshold)
    }


    pub struct WithThreshold<'i, P: 'static + Primitive, Container> {
        image: &'i ImageBuffer<Luma<P>, Container>,
        threshold: P,
    }

    impl<'i, P, C> WithThreshold<'i, P, C> where P: 'static + Primitive {
        pub fn of(image: &'i ImageBuffer<Luma<P>, C>, threshold: P) -> Self {
            WithThreshold { image, threshold }
        }
    }

    impl<'i, P, C> BinaryImage for WithThreshold<'i, P, C>
        where P: 'static + Primitive, C: std::ops::Deref<Target = [P]>
    {
        fn width(&self) -> u16 {
            self.image.width() as u16
        }

        fn height(&self) -> u16 {
            self.image.height() as u16
        }

        fn is_inside(&self, x: u16, y: u16) -> bool {
            self.image.get_pixel(x as u32, y as u32).data[0] > self.threshold
        }
    }

}

