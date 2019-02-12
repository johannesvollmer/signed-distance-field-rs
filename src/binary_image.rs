
/// Represents an image with each pixel being either true or false,
/// corresponding to inside-the-shape and outside-of-the-shape respectively.
/// BinaryImages can be created from byte slices
/// or piston images if the feature `piston_image` is activated.
pub trait BinaryImage {
    #[inline]
    fn width(&self) -> u16;

    #[inline]
    fn height(&self) -> u16;

    #[inline(always)]
    fn is_inside(&self, x: u16, y: u16) -> bool;
}

/// An image which is described by a row major slice of bytes, with one byte per pixel.
/// To determine if a byte is inside or outside,
/// it is compared to a threshold. The default threshold is 127.
pub struct BinaryByteSliceImage<'b> {
    width: u16,
    height: u16,

    /// A row-major image vector with one byte per pixel.
    buffer: &'b [u8],

    /// A pixel must be brighter than this value
    /// in order to be inside the shape.
    threshold: u8,
}

/// Create a binary image from a row major byte slice with each byte brighter than 127 being "inside-the-shape"
pub fn of_byte_slice(buffer: &[u8], width: u16, height: u16) -> BinaryByteSliceImage {
    of_byte_slice_with_threshold(buffer, width, height, 127)
}

/// Create a binary image from a row major byte slice with each byte brighter than the threshold being "inside-the-shape"
pub fn of_byte_slice_with_threshold(buffer: &[u8], width: u16, height: u16, threshold: u8) -> BinaryByteSliceImage {
    debug_assert_eq!(buffer.len(), width as usize * height as usize, "Buffer dimension mismatch");
    BinaryByteSliceImage { width, height, buffer, threshold }
}


impl BinaryImage for BinaryByteSliceImage<'_> {
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


/// Create binary images from piston images.
#[cfg(feature = "piston_image")]
pub mod piston_image {
    use image::*;
    use super::BinaryImage;

    /// Create a binary image from a grey-scale piston image
    /// with all pixels brighter than 127 being inside-the-shape.
    pub fn of_gray_image(image: &GrayImage) -> GrayBinaryImage<u8, Vec<u8>> {
        of_gray_image_with_threshold(image, 127)
    }

    /// Create a binary image from a grey-scale piston image
    /// with all pixels brighter than the threshold being inside-the-shape.
    pub fn of_gray_image_with_threshold(image: &GrayImage, threshold: u8)
        -> GrayBinaryImage<u8, Vec<u8>>
    {
        GrayBinaryImage { image, threshold }
    }


    /// A binary image constructed from a grey-scale piston image
    pub struct GrayBinaryImage<'i, P: 'static + Primitive, Container> {
        image: &'i ImageBuffer<Luma<P>, Container>,
        threshold: P,
    }

    impl<'i, P, C> BinaryImage for GrayBinaryImage<'i, P, C>
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

