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

On my machine, the distance field on an image with
4096 * 4096 (16 Megapixels) pixels can be computed in about 3 seconds.



## TODO
- [ ] Enable customized memory destination 
      instead of predefined allocations
- [ ] Profiling and Benchmarking
- [ ] Consider SIMD and Multithread optimization?