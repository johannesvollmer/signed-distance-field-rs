use crate::binary_image::BinaryImage;


/// Contains the distance field and the vector field produced by `SignedDistanceField::compute`.
/// Can be normalized in order to convert to an image with limited range.
/// The type parameter `D` can be used to customize the memory layout of the distance field.
/// The library provides default Storages for `Vec<f16>` and `Vec<f23>`
/// alias `F16DistanceStorage` and `F32DistanceStorage`.
///
/// If any distance in this field is `INFINITY`, no shapes were found in the binary image.
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
    /// The algorithm used is based on the paper "The dead reckoning signed distance transform"
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
                    distance_field.set_target_with_distance(x, y, x, y, 0.0);
                }
            }
        }

        // perform forwards iteration
        for y in 0..height {
            for x in 0..width {
                // encourage auto vectorization and fetching all distances in parallel
                let left_bottom  = distance_field.distance_by_neighbour(x, y, -1, -1);
                let bottom       = distance_field.distance_by_neighbour(x, y,  0, -1);
                let right_bottom = distance_field.distance_by_neighbour(x, y,  1, -1);
                let left         = distance_field.distance_by_neighbour(x, y, -1,  0);
                let mut own      = distance_field.get_distance(x, y);

                // if any of the neighbour is smaller, update ourselves
                // TODO only write the true smallest instead of overwriting previous distances?
                if left_bottom  < own { own = distance_field.take_neighbour_target(x, y, -1, -1); }
                if bottom       < own { own = distance_field.take_neighbour_target(x, y,  0, -1); }
                if right_bottom < own { own = distance_field.take_neighbour_target(x, y,  1, -1); }
                if left         < own {       distance_field.take_neighbour_target(x, y, -1,  0); }
            }
        }

        // perform backwards iteration
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                // encourage auto vectorization and fetching all distances in parallel
                let right    = distance_field.distance_by_neighbour(x, y,  1,  0);
                let top_left = distance_field.distance_by_neighbour(x, y, -1,  1);
                let top      = distance_field.distance_by_neighbour(x, y,  0,  1);
                let top_right= distance_field.distance_by_neighbour(x, y,  1,  1);
                let mut own  = distance_field.get_distance(x, y);

                // if any of the neighbour is smaller, update ourselves
                // TODO only write the true smallest instead of overwriting previous distances?
                if right     < own { own = distance_field.take_neighbour_target(x, y,  1,  0); }
                if top_left  < own { own = distance_field.take_neighbour_target(x, y, -1,  1); }
                if top       < own { own = distance_field.take_neighbour_target(x, y,  0,  1); }
                if top_right < own {       distance_field.take_neighbour_target(x, y,  1,  1); }
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

    /// Returns a potentially smaller distance, based on the neighbour's distance.
    /// If there is no neighbour (at the bounds of the image), `INFINITY` is returned.
    #[inline(always)]
    fn distance_by_neighbour(&mut self, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32, ) -> f32 {
        // this should be const per function call, as `neighbour` is const per function call
        let distance_to_neighbour = length(neighbour_x, neighbour_y);
        let neighbour_x = x as i32 + neighbour_x;
        let neighbour_y = y as i32 + neighbour_y;

        // if neighbour exists, return the potentially smaller distance to the target
        if is_valid_index(neighbour_x, neighbour_y, self.width, self.height) {
            let neighbours_distance = self.get_distance(
                neighbour_x as u16, neighbour_y as u16
            );

            neighbours_distance + distance_to_neighbour
        }

        else {
            std::f32::INFINITY
        }
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
    fn set_target_with_distance(&mut self, x: u16, y: u16, target_x: u16, target_y: u16, distance: f32) {
        let index = self.flatten_index(x, y);
        self.distances.set(index, distance);
        self.distance_targets[index] = (target_x, target_y);
    }

    /// Update the target field at the specified pixel index and compute the distance
    #[inline(always)]
    fn set_target_and_distance(&mut self, x: u16, y: u16, target_x: u16, target_y: u16) -> f32 {
        let distance = distance(x, y, target_x, target_y);
        self.set_target_with_distance(x, y, target_x, target_y, distance);
        distance
    }

    #[inline(always)]
    fn take_neighbour_target(&mut self, x: u16, y: u16, neighbour_x: i32, neighbour_y: i32) -> f32 {
        debug_assert!(x as i32 + neighbour_x >= 0 && y as i32 + neighbour_y >= 0);
        let target_of_neighbour = self.get_distance_target(
            (x as i32 + neighbour_x) as u16,
            (y as i32 + neighbour_y) as u16
        );

        self.set_target_and_distance(x, y, target_of_neighbour.0, target_of_neighbour.1)
    }

    #[inline(always)]
    fn invert_distance_sign(&mut self, x: u16, y: u16) {
        let index = self.flatten_index(x, y);
        self.distances.set(index, - self.distances.get(index));
    }

    /// Convert x and y pixel coordinates to the corresponding
    /// one-dimensional index in a row-major image vector.
    // Always inline so that the result of self.flatten_index() can be reused in consecutive calls
    #[inline(always)]
    pub fn flatten_index(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            is_valid_index(x as i32, y as i32, self.width, self.height),
            "Invalid pixel target index"
        );

        self.width as usize * y as usize + x as usize
    }

    /// Scales all distances such that the smallest distance is zero and the largest is one.
    /// Also computes the former minimum and maximum distance, as well as the new edge-value.
    /// Returns `None` if the binary image did not contain any shapes.
    pub fn normalize_distances(self) -> NormalizedDistanceField<D> {
        NormalizedDistanceField::normalize(self)
    }

    /// Scales all distances such that the `-max` distances are zero and `max` distances are one.
    /// All distances smaller than `-max` and larger than `max` will be clamped.
    /// Edges (formerly zero-distances) will be at the center, put to `0.5`.
    /// Also collects the former minimum and maximum distance.
    /// Returns `None` if the binary image did not contain any shapes.
    pub fn normalize_clamped_distances(self, max: f32) -> NormalizedDistanceField<D> {
        NormalizedDistanceField::normalize_clamped(self, max)
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
    ((x * x + y * y) as f32).sqrt()
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

/// Scale the value so that it fits into the range `[0,1]`.
#[inline]
fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}


impl<D> NormalizedDistanceField<D> where D: DistanceStorage {

    /// Scales all distances such that the smallest distance is zero and the largest is one.
    /// Also computes the former minimum and maximum distance, as well as the new edge-value.
    /// Returns `None` if the binary image did not contain any shapes.
    pub fn normalize(distance_field: SignedDistanceField<D>) -> Option<Self> {
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

        if min.is_infinite() || max.is_infinite() {
            return None;
        }

        for index in 0..width as usize * height as usize {
            let distance = distance_field.distances.get(index);
            let normalized = normalize(distance, min, max);
            distance_field.distances.set(index, normalized);
        }

        Some(NormalizedDistanceField {
            width, height,
            distances: distance_field.distances,
            zero_distance: (0.0 - min) / (max - min), // FIXME untested
            former_max_distance: max, former_min_distance: min
        })
    }

    /// Scales all distances such that the `-max` distances are zero and `max` distances are one.
    /// All distances smaller than `-max` and larger than `max` will be clamped.
    /// Edges (formerly zero-distances) will be at the center, put to `0.5`.
    /// Also collects the former minimum and maximum distance.
    /// Returns `None` if the binary image did not contain any shapes.
    pub fn normalize_clamped(distance_field: SignedDistanceField<D>, max: f32) -> Option<Self> {
        let mut normalized = NormalizedDistanceField {
            width: distance_field.width,
            height: distance_field.width,
            distances: distance_field.distances,
            former_min_distance: std::f32::INFINITY,
            former_max_distance: std::f32::NEG_INFINITY,
            zero_distance: 0.5,
        };

        for index in 0..normalized.width as usize * normalized.height as usize {
            let distance = normalized.distances.get(index);
            if distance.is_infinite() { return None; }

            normalized.former_max_distance = normalized.former_max_distance.max(distance);
            normalized.former_min_distance = normalized.former_min_distance.min(distance);

            let clamped = distance.min(max).max(-max);
            let normalized_distance = normalize(clamped, -max, max);
            normalized.distances.set(index, normalized_distance);
        }

        Some(normalized)
    }

    /// Convert the normalized distance to an `u8` image with the range fully utilized.
    pub fn to_u8(&self) -> Vec<u8> {
        (0..self.width as usize * self.height as usize)
            .map(|index| (self.distances.get(index).min(1.0).max(0.0) * std::u8::MAX) as u8)
            .collect()
    }

    /// Convert the normalized distance to an `u16` image with the range fully utilized.
    pub fn to_u16(&self) -> Vec<u16> {
        (0..self.width as usize * self.height as usize)
            .map(|index| (self.distances.get(index).min(1.0).max(0.0) * std::u16::MAX) as u16)
            .collect()
    }

    /// Convert the normalized distance to an `u8` gray piston image with the range fully utilized.
    #[cfg(feature = "piston_image")]
    pub fn to_gray_u8_image(&self) -> image::GrayImage {
        image::GrayImage::from_raw(self.width as u32, self.height as u32, self.to_u8())
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