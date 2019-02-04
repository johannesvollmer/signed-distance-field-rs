
pub trait BinaryImage {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn is_inside(&self, x: usize, y: usize) -> bool;
}



pub struct BinaryByteImage<'b> {
    width: usize,
    height: usize,
    buffer: &'b [u8]
}


impl BinaryByteImage<'_> {
    pub fn new(width: usize, height: usize, buffer: &[u8]) -> Self {
        BinaryByteImage {
            width, height, buffer
        }
    }
}

impl BinaryImage for BinaryByteImage<'_> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn is_inside(&self, x: usize, y: usize) -> bool {
        self.buffer[self.width * y + x] > 127
    }
}