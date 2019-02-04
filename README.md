# SDF-DEAD-RECKONING

Fast Signed Distance Fields for Rust!

This crate approximates a signed
distance field, given a binary image. 
The algorithm used is called "dead reckoning", 
as described in `The "dead reckoning" signed distance transform`
by George J. Grevara (2004). 

The complexity of the algorithm
is linear, while an exact algorithm
would have quadratic complexity.

On my machine, the distance field of an image with
4096 * 4096 (16 Megapixels) pixels can be computed in about 1.14 seconds.

## Getting Started

```rust
use sdf_dead_reckoning::prelude::*;
    
fn main(){
    let mut gray_image = image::open("images/sketch.jpg").unwrap().to_luma();
    let binary_image = binary_piston_image::of_gray_u8_image_with_threshold(&gray_image, 80);

    let distance_field = compute_f32_distance_field(&binary_image);
    let distance_image = distance_field.normalize_distances().to_gray_u8_image();

    distance_image.save("images/sketch_distance.png").unwrap();
}

```

Note: To run this specific example, 
use sdf-dead-reckoning with the piston image crate feature, 
by enabling the feature flag `piston_image`.

## TODO
- [x] Enable customized memory destination 
      instead of predefined allocations
- [ ] Profiling and Benchmarking
- [ ] Consider SIMD and Multithread optimization?