
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

        // at every pixel directly next to an edge, set its distance to zero
        for y in 0..height {
            for x in 0..width {
                if     is_at_edge(binary_image, x, y, -1,  0)
                    || is_at_edge(binary_image, x, y,  1,  0)
                    || is_at_edge(binary_image, x, y,  0, -1)
                    || is_at_edge(binary_image, x, y,  0,  1)
                {
                    distance_field.set_distance_target(x, y, x, y);
                }
            }
        }

        // perform forwards iteration
        for y in 0..height {
            for x in 0..width {
                distance_field.update_distance_based_on_neighbour(x, y, -1, -1);
                distance_field.update_distance_based_on_neighbour(x, y,  0, -1);
                distance_field.update_distance_based_on_neighbour(x, y,  1, -1);
                distance_field.update_distance_based_on_neighbour(x, y, -1,  0);
            }
        }

        // perform backwards iteration
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                distance_field.update_distance_based_on_neighbour(x, y,  1,  0);
                distance_field.update_distance_based_on_neighbour(x, y, -1,  1);
                distance_field.update_distance_based_on_neighbour(x, y,  0,  1);
                distance_field.update_distance_based_on_neighbour(x, y,  1,  1);
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
    fn update_distance_based_on_neighbour(&mut self, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32) {
        // this should be const per function call, as `neighbour` is const per function call
        let distance_to_neighbour = length(neighbour_x, neighbour_y);

        let neighbour_x = x as i32 + neighbour_x;
        let neighbour_y = y as i32 + neighbour_y;

        // if neighbour exists, update ourselves according to the neighbour
        if check_coordinates(neighbour_x, neighbour_y, self.width, self.height) {
            let neighbour_x = neighbour_x as u16;
            let neighbour_y = neighbour_y as u16;
            let neighbour_distance = self.get_distance(neighbour_x, neighbour_y);

            // subsequent calls should use only one single lookup, after inlining
            let own_distance = self.get_distance(x, y);

            // if neighbour is closer to edge than ourselves,
            // set our distance to the neighbours distance plus the space between us
            if neighbour_distance + distance_to_neighbour < own_distance {
                let neighbour_target = self.get_distance_target(neighbour_x, neighbour_y);
                self.set_distance_target(x, y, neighbour_target.0, neighbour_target.1);
            }
        }
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
    fn set_distance_target(&mut self, x: u16, y: u16, target_x: u16, target_y: u16) {
        let index = self.flatten_index(x, y);
        self.distances.set(index, distance(x, y, target_x, target_y));
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