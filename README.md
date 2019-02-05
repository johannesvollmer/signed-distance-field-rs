# SIGNED-DISTANCE-FIELD-RS
*Fast Signed Distance Fields for Rust*

This crate approximates a signed
distance field, given a binary image. 
The algorithm is inspired by the paper
"The dead reckoning signed distance transform"
by George J. Grevara (2004).

On my laptop, the distance field of an image with
4096px x 4096px (16 Megapixels) 
containing a circle with radius of 6px in the center 
can be computed in about 0.927 seconds.

## Features
In the process of computing the signed distance field, 
the algorithm constructs an image with each pixel 
containing the vectors which point to the nearest edge. 
This vector distance field is made available
after computing the plain distance field and can be used 
for further processing. Also, the library offers a simple
conversion from distance fields to images with integer precision.


## Getting Started

Update your `Cargo.toml`:
```toml
signed-distance-field = "0.6.2"
```

Use `compute_f32_distance_field` to compute 
a distance field with `f32` precision and memory usage.

```rust
use signed_distance_field::prelude::*;
    
fn main(){
    let mut gray_image = image::open("images/sketch.jpg").unwrap().to_luma();
    let binary_image = binary_piston_image::of_gray_u8_image_with_threshold(&gray_image, 80);

    let distance_field = compute_f32_distance_field(&binary_image);
    let distance_image = distance_field.normalize_distances().to_gray_u8_image();

    distance_image.save("images/sketch_distance.png").unwrap();
}
```

To run this specific example, the `piston_image` feature flag must be enabled.

## Piston Images
This library can be configured to offer some 
simple conversions to and from piston images.
The feature flag `piston_image` unlocks these functions.
The image crate is not required to calculate the
signed distance field. 

Update your `Cargo.toml`:
```toml
signed-distance-field = { version = "0.6.2", features = [ "piston_image" ] }
```

### Cons (yet)
- Single Code only
- GPU not used
- May not as accurate as a naive approach

### What's up next?
- Consider optimizing for SIMD and multithreading
- Consider adding alternative algorithms, possibly with GPU utilization