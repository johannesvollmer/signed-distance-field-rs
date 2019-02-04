
use crate::binary_image::BinaryImage;
use std::ops::IndexMut;



#[derive(Clone, PartialEq, Debug)]
pub struct SignedDistanceField<D: DistanceStorage> {
    pub width: u16,
    pub height: u16,
    pub distances: D,
    pub distance_targets: Vec<(u16, u16)>
}

/// Needs less storage with sufficient precision, but takes about
/// twice as long because of conversions between f16 and f32.
pub type F16DistanceStorage = Vec<half::f16>;

/// Needs more storage with high precision, but takes about
/// half as long because no conversions between f16 and f32 must be made.
pub type F32DistanceStorage = Vec<f32>;

pub trait DistanceStorage {
    fn new(length: usize) -> Self;

    #[inline]
    fn get(&self, index: usize) -> f32;

    #[inline]
    fn set(&mut self, index: usize, distance: f32);
}



impl<D> SignedDistanceField<D> where D: DistanceStorage {

    /// Approximates the signed distance field of the specified image.
    /// The algorithm used is based on the paper `The "dead reckoning" signed distance transform`
    /// by George J. Grevara, 2004.
    pub fn compute_approximate(binary_image: &impl BinaryImage) -> Self {
        let width = binary_image.width();
        let height = binary_image.height();

        let mut distance_field = SignedDistanceField {
            width, height,
            distances: D::new(width as usize * height as usize),
            distance_targets: vec![(0, 0); width as usize * height as usize],
        };

        // for every pixel directly at an edge, set its distance to zero
        for y in 0..height {
            for x in 0..width {
                if     is_at_edge(binary_image, x, y, -1,  0)
                    || is_at_edge(binary_image, x, y,  1,  0)
                    || is_at_edge(binary_image, x, y,  0, -1)
                    || is_at_edge(binary_image, x, y,  0,  1)
                {
                    distance_field.set_target_and_distance(x, y, x, y, 0.0);
                }
            }
        }

        // perform forwards iteration
        for y in 0..height {
            for x in 0..width {
                let mut distance = distance_field.get_distance(x, y);
                let (mut target_x, mut target_y) = distance_field.get_distance_target(x, y);

                distance_field.update_distance(x, y, -1, -1, &mut distance, &mut target_x, &mut target_y);
                distance_field.update_distance(x, y,  0, -1, &mut distance, &mut target_x, &mut target_y);
                distance_field.update_distance(x, y,  1, -1, &mut distance, &mut target_x, &mut target_y);
                distance_field.update_distance(x, y, -1,  0, &mut distance, &mut target_x, &mut target_y);

                distance_field.set_target_and_distance(x,y, target_x, target_y, distance);
            }
        }

        // perform backwards iteration.
        // Similar to first iteration, but only writes conditionally,
        // as not all pixels will be updated in this iteration
        // which will save us some f16 conversion and heap writes
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                let mut distance = distance_field.get_distance(x, y);
                let (mut target_x, mut target_y) = distance_field.get_distance_target(x, y);

                let right = distance_field.update_distance(
                    x, y,  1,  0, &mut distance, &mut target_x, &mut target_y
                );

                let top_left = distance_field.update_distance(
                    x, y, -1,  1, &mut distance, &mut target_x, &mut target_y
                );

                let top = distance_field.update_distance(
                    x, y,  0,  1, &mut distance, &mut target_x, &mut target_y
                );

                let top_right = distance_field.update_distance(
                    x, y,  1,  1, &mut distance, &mut target_x, &mut target_y
                );

                if right || top_left || top || top_right {
                    distance_field.set_target_and_distance(x,y, target_x, target_y, distance);
                }
            }
        }

        // flip distance signs
        // where a pixel is inside the shape
        for y in 0..height {
            for x in 0..width {
                if binary_image.is_inside(x, y) {
                    distance_field.invert_distance_sign(x, y);
                }
            }
        }

        distance_field
    }

    #[inline(always)]
    fn update_distance(
        &mut self, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32,
        own_distance: &mut f32, own_target_x: &mut u16, own_target_y: &mut u16
    ) -> bool {
        // this should be const per function call, as `neighbour` is const per function call
        let distance_to_neighbour = length(neighbour_x, neighbour_y);

        let neighbour_x = x as i32 + neighbour_x;
        let neighbour_y = y as i32 + neighbour_y;

        // if neighbour exists, update ourselves according to the neighbour
        if check_coordinates(neighbour_x, neighbour_y, self.width, self.height) {
            let neighbour_x = neighbour_x as u16;
            let neighbour_y = neighbour_y as u16;
            let neighbour_distance = self.get_distance(neighbour_x, neighbour_y);

            // if neighbour is closer to edge than ourselves,
            // set our distance to the neighbours distance plus the space between us
            if neighbour_distance + distance_to_neighbour < *own_distance {
                let neighbour_target = self.get_distance_target(neighbour_x, neighbour_y);

                *own_distance = distance(x, y, neighbour_target.0, neighbour_target.1);
                *own_target_x = neighbour_target.0;
                *own_target_y = neighbour_target.1;
                return true
            }
        }

        false
    }

    #[inline(always)]
    pub fn get_distance(&mut self, x: u16, y: u16) -> f32 {
        self.distances.get(self.flatten_index(x, y))
    }

    #[inline(always)]
    pub fn get_distance_target(&mut self, x: u16, y: u16) -> (u16, u16) {
        self.distance_targets[self.flatten_index(x, y)]
    }

    #[inline(always)]
    fn set_target_and_distance(&mut self, x: u16, y: u16, target_x: u16, target_y: u16, distance: f32) {
        let index = self.flatten_index(x, y);
        self.distances.set(index, distance);
        self.distance_targets[index] = (target_x, target_y);
    }

    #[inline(always)]
    fn invert_distance_sign(&mut self, x: u16, y: u16) {
        let index = self.flatten_index(x, y);
        self.distances.set(index, - self.distances.get(index));
    }


    #[inline]
    pub fn flatten_index(&self, x: u16, y: u16) -> usize {
        self.width as usize * y as usize + x as usize
    }
}

#[inline(always)]
fn is_at_edge(image: &impl BinaryImage, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32) -> bool {
    let neighbour_x = x as i32 + neighbour_x;
    let neighbour_y = y as i32 + neighbour_y;

    check_coordinates(neighbour_x, neighbour_y, image.width(), image.height())

        // consecutive `image.is_inside(x, y)` should be optimized to a single call in a loop
        && image.is_inside(x, y) != image.is_inside(neighbour_x as u16, neighbour_y as u16)
}

#[inline]
fn length(x: i32, y: i32) -> f32 {
    let sqr_distance = x * x + y * y;
    (sqr_distance as f32).sqrt()
}

#[inline]
fn distance(x: u16, y: u16, target_x: u16, target_y: u16) -> f32 {
    length(x as i32 - target_x as i32, y as i32 - target_y as i32)
}

#[inline]
fn check_coordinates(x: i32, y: i32, width: u16, height: u16) -> bool {
    x >= 0 && y >= 0 && x < width as i32 && y < height as i32
}


impl DistanceStorage for F16DistanceStorage {
    fn new(length: usize) -> Self {
        vec![half::consts::INFINITY; length]
    }

    #[inline]
    fn get(&self, index: usize) -> f32 {
        self[index].to_f32()
    }

    #[inline]
    fn set(&mut self, index: usize, distance: f32) {
        self[index] = half::f16::from_f32(distance)
    }
}

impl DistanceStorage for F32DistanceStorage {
    fn new(length: usize) -> Self {
        vec![std::f32::INFINITY; length]
    }

    #[inline]
    fn get(&self, index: usize) -> f32 {
        self[index]
    }

    #[inline]
    fn set(&mut self, index: usize, distance: f32) {
        self[index] = distance
    }
}