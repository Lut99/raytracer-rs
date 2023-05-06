//  DIFFUSE.rs
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
//!   Implements various kinds of diffuse-like materials, all with
//!   slightly different methods of "randomly" bouncing rays.
// 

use serde::{Deserialize, Serialize};

use crate::math::{Colour, Ray, Vec3, Vector as _};
use crate::specifications::objects::HitRecord;

use super::spec::Material;


/***** HELPER FUNCTIONS *****/
/// Generates a random, uniformly sampled vector in a unit sphere around the origin.
/// 
/// # Returns
/// A new [`Vec3`] that represents the random vector.
pub fn random3_uniform() -> Vec3 {
    // Generate the three coordinates randomly
    let res: Vec3 = Vec3 {
        x : fastrand::f64(),
        y : fastrand::f64(),
        z : fastrand::f64(),
    };

    // Always return a unit vector version of this vector
    res.unit()
}





/***** LIBRARY *****/
/// Implements the laziest diffuse material, which simply uniformly bounces the ray off of its surface.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Diffuse {
    /// The colour of the material.
    colour : Colour,
}
impl Material for Diffuse {
    #[inline]
    fn scatter(&self, _ray: Ray, record: HitRecord) -> (Option<Ray>, Colour) {
        // Compute the scattered ray, making sure the scattered one is not zero
        let mut scattered: Vec3 = record.normal + random3_uniform();
        if scattered.is_nearly_zero() { scattered = record.normal; }

        // Now we can simply return the new ray to bounce and the colour
        (Some(Ray::new(record.hit, scattered)), self.colour)
    }
}
