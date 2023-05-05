//  GENERATOR.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:13:21
//  Last edited:
//    05 May 2023, 10:36:12
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`RayGenerator`], which is an iterator that casts
//!   [`Ray`]s.
// 

use rand::Rng as _;
use rand::distributions::Uniform;

use crate::math::{Camera, Ray};


/***** AUXILLARY *****/
/// A wrapper around iterators that wraps their result in a tuple that contains the current coordinate.
pub struct CoordinateEnumerate<I> {
    /// The iterator to iterate over.
    iter      : I,
    /// The horizontal dimension we are iterating over. We only need this one, since `iter` will do the stopping for us.
    width     : u32,
    /// The number of rays cast per pixel. For us, this is the number of times we keep the coordinates "the same" before incrementing.
    n_samples : usize,

    /// Our current sample index
    s : usize,
    /// Our current X-coordinate
    x : usize,
    /// Our current Y-coordinate
    y : usize,
}
impl<I: Iterator> Iterator for CoordinateEnumerate<I> {
    type Item = ((usize, u32, u32), I::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next element and map it
        self.iter.next().map(|elem| {
            // Get the to-be-returned coordinates
            let (s, x, y): (usize, u32, u32) = (self.s, self.x as u32, self.y as u32);

            // Increment the appropriate stuff
            self.s += 1;
            if self.s >= self.n_samples {
                self.s = 0;
                self.x += 1;
                if self.x >= self.width as usize {
                    self.x = 0;
                    self.y += 1;
                }
            }

            // Return that which we found
            ((s, x, y), elem)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}
impl<I: ExactSizeIterator> ExactSizeIterator for CoordinateEnumerate<I> {
    #[inline]
    fn len(&self) -> usize { self.iter.len() }
}





/***** LIBRARY *****/
/// The RayGenerator is an iterator over [`Ray`]s.
#[derive(Clone, Copy, Debug)]
pub struct RayGenerator {
    /// Our current index of iteration.
    index : usize,

    /// The Camera through which we cast rays.
    camera    : Camera,
    /// The total width and height that we are generating over.
    dims      : (u32, u32),
    /// The number of rays we cast per pixel.
    n_samples : usize,
}

impl RayGenerator {
    /// Constructor for the RayGenerator.
    /// 
    /// # Arguments
    /// - `camera`: The [`Camera`] that defines the logical viewport through which we cast rays.
    /// - `dims`: The physical pixel values of the the image to render.
    /// - `n_samples`: The number of rays we cast per pixel. Passing `1` is the same as disabling anti-aliasing.
    /// 
    /// # Returns
    /// A new instance of Self that can be used to generate rays.
    #[inline]
    pub fn new(camera: Camera, dims: (impl Into<u32>, impl Into<u32>), n_samples: usize) -> Self {
        Self {
            index : 0,
    
            camera,
            dims : (dims.0.into(), dims.1.into()),
            n_samples,
        }
    }



    /// Returns a wrapper around the `RayGenerator` that produces an `(x, y)` index in addition to the element, much like [`std::iter::Enumerate`] does,
    /// 
    /// # Returns
    /// A new [`CoordinateEnumerate`] class that wraps ourselves.
    #[inline]
    pub fn coords(self) -> CoordinateEnumerate<Self> {
        CoordinateEnumerate {
            iter      : self,
            width     : self.dims.0,
            n_samples : self.n_samples,

            s : 0,
            x : 0,
            y : 0,
        }
    }
}

impl Iterator for RayGenerator {
    type Item = Ray;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if out-of-bounds
        if self.index >= self.n_samples * self.dims.0 as usize * self.dims.1 as usize { return None; }

        // Split the index into a pixel-base X & Y
        let rem: usize = self.index / self.n_samples;
        let mut x: f64 = (rem % self.dims.0 as usize) as f64;
        let mut y: f64 = (rem / self.dims.0 as usize) as f64;

        // Add a random value if we are antialiasing
        if self.n_samples > 1 {
            let mut rng = rand::thread_rng();
            let dist: Uniform<f64> = Uniform::new(0.0, 1.0);
            x += rng.sample(dist);
            y += rng.sample(dist);
        }

        // Compute the logical values of these
        let u: f64 = x / (self.dims.0 as f64 - 1.0);
        let v: f64 = y / (self.dims.1 as f64 - 1.0);

        // Compute the Ray with those and the Camera viewport
        self.index += 1;
        Some(self.camera.cast(u, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.n_samples * self.dims.0 as usize * self.dims.1 as usize;
        (len, Some(len))
    }
}
impl ExactSizeIterator for RayGenerator {
    #[inline]
    fn len(&self) -> usize { self.n_samples * self.dims.0 as usize * self.dims.1 as usize - self.index as usize }
}
