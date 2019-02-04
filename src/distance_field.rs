use crate::binary_image::BinaryImage;


/// Contains the distance field and the vector field produced by `SignedDistanceField::compute`.
/// Can be normalized in order to convert to an image with limited range.
/// The type parameter `D` can be used to customize the memory layout of the distance field.
/// The default storages are `Vec<f23>` and `Vec<f16>`.
#[derive(Clone, PartialEq, Debug)]
pub struct SignedDistanceField<D: DistanceStorage> {
    pub width: u16,
    pub height: u16,

    /// A row-major image vector with
    /// for each pixel of the original image
    /// containing the distance from that pixel to the nearest edge
    pub distances: D,

    /// A row-major image vector with
    /// for each pixel of the original image
    /// containing the absolute position of the nearest edge from that pixel
    pub distance_targets: Vec<(u16, u16)>
}

/// Store distances as a vector of `f16` numbers.
/// Needs less storage with sufficient precision,
/// but significantly longer to compute
/// because of conversions between f16 and f32.
pub type F16DistanceStorage = Vec<half::f16>;

/// Store distances as a vector of `f32` numbers.
/// Needs more storage while providing high precision, but is significantly quicker
/// because no conversions between f16 and f32 must be made.
pub type F32DistanceStorage = Vec<f32>;

/// Specifies how to store distances in memory.
/// This library defines an `f16` storage and an `f32` storage.
pub trait DistanceStorage {

    /// Construct a new linear storage with the specified length.
    /// __All distances in this array must be initialized to `INFINITY`.__
    fn new(length: usize) -> Self;

    #[inline(always)]
    fn get(&self, index: usize) -> f32;

    #[inline(always)]
    fn set(&mut self, index: usize, distance: f32);
}

/// Represents a distance field which was normalized to the range `[0, 1]`.
/// Also contains information about the greatest distances of the unnormalized distance field.
pub struct NormalizedDistanceField<D: DistanceStorage> {
    pub width: u16,
    pub height: u16,

    /// All distances are in the range of `[0..1]`.
    pub distances: D,

    /// In the original distance field, edges are represented by a distance of zero.
    /// Normalizing the distance field will result in edges no longer being zero.
    /// The normalized field will have edges somewhere between zero and one.
    /// This float describes the new value that edges in the normalized field have.
    pub zero_distance: f32,

    /// The largest distance in the image
    /// to the nearest edge
    /// __outside__ of a shape .
    pub former_min_distance: f32,

    /// The largest distance in the image
    /// to the nearest edge
    /// __inside__ of a shape
    pub former_max_distance: f32
}


impl<D> SignedDistanceField<D> where D: DistanceStorage {

    /// Approximates the signed distance field of the specified image.
    /// The algorithm used is based on the paper `The "dead reckoning" signed distance transform`
    /// by George J. Grevara, 2004.
    pub fn compute(binary_image: &impl BinaryImage) -> Self {
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

                // write unconditionally to avoid branching,
                // as almost all values will be written in the first pass
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

                // only write if something changed, as the second pass may touch few pixels
                // (profiled with an image of a circle)
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

    /// Returns true, if the mutable values have been changed should be updated
    #[inline(always)]
    fn update_distance(
        &mut self, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32,
        own_distance: &mut f32, own_target_x: &mut u16, own_target_y: &mut u16
    ) -> bool
    {
        // this should be const per function call, as `neighbour` is const per function call
        let distance_to_neighbour = length(neighbour_x, neighbour_y);

        let neighbour_x = x as i32 + neighbour_x;
        let neighbour_y = y as i32 + neighbour_y;

        // if neighbour exists, update ourselves according to the neighbour
        if is_valid_index(neighbour_x, neighbour_y, self.width, self.height) {
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

    /// Returns the distance of the specified pixel to the nearest edge in the original image.
    #[inline(always)]
    pub fn get_distance(&self, x: u16, y: u16) -> f32 {
        self.distances.get(self.flatten_index(x, y))
    }

    /// Returns the absolute index of the nearest edge to the specified pixel in the original image.
    #[inline(always)]
    pub fn get_distance_target(&self, x: u16, y: u16) -> (u16, u16) {
        self.distance_targets[self.flatten_index(x, y)]
    }

    /// Update the distance and target field at the specified pixel index
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

    /// Convert x and y pixel coordinates to the corresponding
    /// one-dimensional index in a row-major image vector.
    #[inline]
    pub fn flatten_index(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            is_valid_index(x as i32, y as i32, self.width, self.height),
            "Invalid pixel target index"
        );

        self.width as usize * y as usize + x as usize
    }

    /// Scales all distances such that the smallest distance is zero and the largest is one.
    /// Also computes the former minimum and maximum distance, as well as the new edge-value.
    pub fn normalize_distances(self) -> NormalizedDistanceField<D> {
        NormalizedDistanceField::from_distance_field(self)
    }
}

/// Returns if the binary image contains an edge
/// at the specified pixel compared to the specified neighbour.
#[inline(always)]
fn is_at_edge(image: &impl BinaryImage, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32) -> bool {
    let neighbour_x = x as i32 + neighbour_x;
    let neighbour_y = y as i32 + neighbour_y;

    is_valid_index(neighbour_x, neighbour_y, image.width(), image.height())

        // consecutive `image.is_inside(x, y)` should be optimized to a single call in a loop
        && image.is_inside(x, y) != image.is_inside(neighbour_x as u16, neighbour_y as u16)
}

/// The length of a vector with x and y coordinates.
#[inline]
fn length(x: i32, y: i32) -> f32 {
    let sqr_distance = x * x + y * y;
    (sqr_distance as f32).sqrt()
}

/// The distance between to points with x and y coordinates.
#[inline]
fn distance(x: u16, y: u16, target_x: u16, target_y: u16) -> f32 {
    length(x as i32 - target_x as i32, y as i32 - target_y as i32)
}

/// Check if x and y are valid pixel coordinates
/// inside an image with the specified width and height.
#[inline]
fn is_valid_index(x: i32, y: i32, width: u16, height: u16) -> bool {
    x >= 0 && y >= 0 && x < width as i32 && y < height as i32
}


impl<D> NormalizedDistanceField<D> where D: DistanceStorage {

    /// Scales all distances such that the smallest distance is zero and the largest is one.
    /// Also computes the former minimum and maximum distance, as well as the new edge-value.
    pub fn from_distance_field(distance_field: SignedDistanceField<D>) -> Self {
        let mut distance_field = distance_field;
        let width = distance_field.width;
        let height = distance_field.height;

        let (min, max) = (0..width as usize * height as usize)
            .map(|index| distance_field.distances.get(index))
            .fold(
                (std::f32::INFINITY, std::f32::NEG_INFINITY),
                |(min, max), distance| (
                    min.min(distance),
                    max.max(distance)
                )
            );

        for index in 0..width as usize * height as usize {
            let distance = distance_field.distances.get(index);
            let normalized = (distance - min) / (max - min);
            distance_field.distances.set(index, normalized);
        }

        NormalizedDistanceField {
            width, height,
            distances: distance_field.distances,
            zero_distance: (0.0 - min) / (max - min),
            former_max_distance: max, former_min_distance: min
        }
    }

    /// Convert the normalized distance to an `u8` image with the range fully utilized.
    #[cfg(feature = "piston_image")]
    pub fn to_gray_u8_image(&self) -> image::GrayImage {
        let vec = (0..self.width as usize * self.height as usize)
            .map(|index| (self.distances.get(index).min(1.0).max(0.0) * 255.0) as u8)
            .collect();

        image::GrayImage::from_raw(self.width as u32, self.height as u32, vec)
            .expect("incorrect vector length")
    }
}

impl DistanceStorage for F16DistanceStorage {
    fn new(length: usize) -> Self {
        vec![half::consts::INFINITY; length]
    }

    #[inline(always)]
    fn get(&self, index: usize) -> f32 {
        self[index].to_f32()
    }

    #[inline(always)]
    fn set(&mut self, index: usize, distance: f32) {
        self[index] = half::f16::from_f32(distance)
    }
}

impl DistanceStorage for F32DistanceStorage {
    fn new(length: usize) -> Self {
        vec![std::f32::INFINITY; length]
    }

    #[inline(always)]
    fn get(&self, index: usize) -> f32 {
        self[index]
    }

    #[inline(always)]
    fn set(&mut self, index: usize, distance: f32) {
        self[index] = distance
    }
}