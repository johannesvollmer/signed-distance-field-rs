use std::ops::Index;

pub trait BinaryImage {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn is_shape(&self, x: usize, y: usize) -> bool;
}

pub struct BinaryByteImage<'b> {
    width: usize,
    buffer: &'b [u8]
}

impl BinaryByteImage<'_> {

}

impl BinaryImage for BinaryByteImage<'_> {
    fn width(&self) -> usize {
        unimplemented!()
    }

    fn height(&self) -> usize {
        unimplemented!()
    }

    fn is_shape(&self, x: usize, y: usize) -> bool {
        unimplemented!()
    }
}


pub fn approximate_signed_distance_field(image: &impl BinaryImage) {

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
