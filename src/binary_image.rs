
pub trait BinaryImage {
    #[inline]
    fn width(&self) -> u16;

    #[inline]
    fn height(&self) -> u16;

    #[inline]
    fn is_inside(&self, x: u16, y: u16) -> bool;
}

#[inline(always)]
pub fn is_at_edge(image: &impl BinaryImage, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32) -> bool {
    let neighbour_x = x as i32 + neighbour_x;
    let neighbour_y = y as i32 + neighbour_y;

    crate::distance_field::check_coordinates(neighbour_x, neighbour_y, image.width(), image.height())

        // consecutive `image.is_inside(x, y)` should be optimized to a single call in a loop
        && image.is_inside(x, y) != image.is_inside(neighbour_x as u16, neighbour_y as u16)
}


pub struct BinaryByteImage<'b> {
    width: u16,
    height: u16,
    buffer: &'b [u8]
}


impl<'b> BinaryByteImage<'b> {
    pub fn from_slice(width: u16, height: u16, buffer: &'b [u8]) -> Self {
        BinaryByteImage {
            width, height, buffer
        }
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
        self.buffer[self.width as usize * y as usize + x as usize] > 127
    }
}