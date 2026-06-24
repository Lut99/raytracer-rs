//  LAMBERTIAN.rs
//    by Lut99
//
//  Created:
//    05 May 2023, 10:50:32
//  Last edited:
//    06 May 2023, 11:34:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a [`Diffuse`](super::diffuse::Diffuse)-like material but then
//!   with truer method of reflection (and, honestly, a simpler one).
//

use serde::{Deserialize, Serialize};

use super::Material;
use super::diffuse::random3_uniform;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitRecord;


/***** LIBRARY *****/
/// A diffuse material with truer scattering.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Lambertian {
    /// The colour of the material.
    pub colour: Colour,
}
impl Material for Lambertian {
    #[inline]
    fn scatter(&self, _ray: Ray, record: HitRecord) -> (Option<Ray>, Colour) {
        // Compute the scattered ray, making sure the scattered one is not zero
        let mut scattered: Vec3 = record.normal + random3_uniform();
        if scattered.is_nearly_zero() {
            scattered = record.normal;
        }

        // Now we can simply return the new ray to bounce and the colour
        (Some(Ray::new(record.hit, scattered)), self.colour)
    }
}
