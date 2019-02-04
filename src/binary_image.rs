
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