//  SIMPLE.rs
//    by Lut99
// 
//  Created:
//    05 May 2023, 11:41:04
//  Last edited:
//    06 May 2023, 12:05:58
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines super-simple materials, mostly used for debugging or to
//!   represent earlier parts of the tutorial.
// 

use serde::{Deserialize, Serialize};

use crate::math::{Colour, Ray};
use crate::specifications::objects::HitRecord;

use super::spec::Material;


/***** LIBRARY *****/
/// Implements a non-bouncing, just-normal-map kind of material. Mostly created for the scene in the [tutorial](https://raytracing.github.io/books/RayTracingInOneWeekend.html#surfacenormalsandmultipleobjects/commonconstantsandutilityfunctions).
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct NormalMap;
impl Material for NormalMap {
    #[inline]
    fn scatter(&self, _ray: Ray, record: HitRecord) -> (Option<Ray>, Colour) {
        // Compute the normal map colour based on the normal
        (None, 0.5 * Colour::new(record.normal.x + 1.0, record.normal.y + 1.0, record.normal.z + 1.0, 2.0))
    }
}
