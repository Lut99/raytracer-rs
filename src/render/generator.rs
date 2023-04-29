//  GENERATOR.rs
//    by Lut99
// 
//  Created:
//    29 Apr 2023, 10:13:21
//  Last edited:
//    29 Apr 2023, 10:38:43
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`RayGenerator`], which is an iterator that casts
//!   [`Ray`]s.
// 

use crate::math::{Camera, Ray};


/***** AUXILLARY *****/
/// A wrapper around iterators that wraps their result in a tuple that contains the current coordinate.
pub struct CoordinateEnumerate<I> {
    /// The iterator to iterate over.
    iter  : I,
    /// The index of the dimensions.
    index : u32,
    /// The horizontal dimension we are iterating over. We only need this one, since `iter` will do the stopping for us.
    width : u32,
}

impl<I: Iterator> Iterator for CoordinateEnumerate<I> {
    type Item = ((u32, u32), I::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next element and map it
        self.iter.next().map(|elem| {
            // Compute the coordinates
            let x: u32 = self.index % self.width;
            let y: u32 = self.index / self.width;

            // Increment `index`, then return
            self.index += 1;
            ((x, y), elem)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}





/***** LIBRARY *****/
/// The RayGenerator is an iterator over [`Ray`]s.
#[derive(Clone, Copy, Debug)]
pub struct RayGenerator {
    /// Our current index of iteration.
    index  : u32,
    /// The total width and height that we are generating over.
    dims   : (u32, u32),
    /// The Camera through which we cast rays.
    camera : Camera,
}

impl RayGenerator {
    /// Constructor for the RayGenerator.
    /// 
    /// # Arguments
    /// - `dims`: The physical pixel values of the the image to render.
    /// - `camera`: The [`Camera`] that defines the logical viewport through which we cast rays.
    /// 
    /// # Returns
    /// A new instance of Self that can be used to generate rays.
    #[inline]
    pub fn new(dims: (impl Into<u32>, impl Into<u32>), camera: Camera) -> Self {
        Self {
            index : 0,
            dims  : (dims.0.into(), dims.1.into()),
            camera,
        }
    }



    /// Returns a wrapper around the `RayGenerator` that produces an `(x, y)` index in addition to the element, much like [`std::iter::Enumerate`] does,
    /// 
    /// # Returns
    /// A new [`CoordinateEnumerate`] class that wraps ourselves.
    #[inline]
    pub fn coords(self) -> CoordinateEnumerate<Self> {
        CoordinateEnumerate {
            iter  : self,
            index : self.index,
            width : self.dims.0,
        }
    }
}

impl Iterator for RayGenerator {
    type Item = Ray;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if out-of-bounds
        if self.index >= self.dims.0 * self.dims.1 { return None; }

        // Split the index into a pixel-base X & Y
        let x: u32 = self.index % self.dims.0;
        let y: u32 = self.index / self.dims.0;

        // Compute the logical values of these
        let u: f64 = x as f64 / (self.dims.0 as f64 - 1.0);
        let v: f64 = y as f64 / (self.dims.1 as f64 - 1.0);

        // Compute the Ray with those and the Camera viewport
        self.index += 1;
        Some(Ray::new(self.camera.origin, self.camera.lower_left_corner + u * self.camera.horizontal + v * self.camera.vertical - self.camera.origin))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = (self.dims.0 * self.dims.1) as usize;
        (len, Some(len))
    }
}
impl ExactSizeIterator for RayGenerator {
    #[inline]
    fn len(&self) -> usize { (self.dims.0 * self.dims.1 - self.index) as usize }
}
