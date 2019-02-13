[![Crate](https://img.shields.io/crates/v/signed-distance-field.svg)](https://crates.io/crates/signed-distance-field)
[![Documentation](https://docs.rs/signed-distance-field/badge.svg)](https://docs.rs/crate/signed-distance-field/)

# SIGNED-DISTANCE-FIELD-RS
*Fast Signed Distance Fields for Rust*

This crate approximates a signed
distance field, given a binary image. 
The algorithm is inspired by the paper
"The dead reckoning signed distance transform"
by George J. Grevara (2004). Don't forget
to compile in release mode!

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
signed-distance-field = { version = "0.6.3", features = [ "piston_image" ] }
```

```rust
use signed_distance_field::prelude::*;
    
fn main(){
    // load data using piston image
    let mut gray_image = image::open("sketch.jpg").unwrap().to_luma();

    // interpret grayscale image as binary image with any pixel brighter than 80 being 'on'
    let binary_image = binary_piston_image::of_gray_image_with_threshold(&gray_image, 80);

    // convert the binary image to a distance field
    let distance_field = compute_f32_distance_field(&binary_image);
    
    // clip all distances greater than 10px and compress them into a byte array
    // so that a distance of -10px is 0 and a distance of 10px is 255
    // (edges, having a distance of 0px, will be 128)
    let distance_image = distance_field
        .normalize_clamped_distances(-10.0, 10.0)

        // convert f32 distance field to u8 piston image
        .unwrap().to_gray_u8_image(); 

    // save the piston image as png
    distance_image.save("sketch_distance.png").unwrap();
}
```

## Piston Images
This library can be configured to offer some 
simple conversions to and from piston images.
The feature flag `piston_image` unlocks these functions.
The image crate is not required to calculate the
signed distance field, including piston image is truly optional. 

### Cons (yet)
- Single Core only
- Maybe not as accurate as a naive approach
- Neither GPU not SIMD acceleration explicitly used

### What's up next?
- Consider optimizing for SIMD and multithreading
- Consider adding alternative algorithms, possibly with GPU utilization