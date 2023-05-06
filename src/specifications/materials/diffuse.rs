//  DIFFUSE.rs
//    by Lut99
// 
//  Created:
//    05 May 2023, 10:50:32
//  Last edited:
//    05 May 2023, 11:40:05
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements various kinds of diffuse-like materials, all with
//!   slightly different methods of "randomly" bouncing rays.
// 

use num_traits::{Float, Num};
use rand::Rng as _;
use rand::distributions::uniform::{SampleUniform, Uniform};
use serde::{Deserialize, Serialize};

use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitRecord;

use super::spec::Material;


/***** HELPER FUNCTIONS *****/
/// Generates a random, uniformly sampled vector in a unit sphere around the origin.
/// 
/// # Returns
/// A new [`Vec3`] that represents the random vector.
pub fn random3_uniform<T: Float + Num + SampleUniform>() -> Vec3<T> {
    // Define the distribution we will sample from
    let mut rng = rand::thread_rng();
    let dist: Uniform<T> = Uniform::new(T::zero(), T::one());

    // Generate the three coordinates randomly
    let res: Vec3<T> = Vec3 {
        x : rng.sample(&dist),
        y : rng.sample(&dist),
        z : rng.sample(&dist),
    };

    // Compute the scale we need
    let scale: T = (res.x * res.x + res.y * res.y + res.z * res.z).sqrt();

    // Now put it in a vector with scaling to fix the unit length to 1.
    res / scale
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
        // Simply return the new ray to bounce and the colour
        (Some(Ray::new(record.hit, record.normal + random3_uniform())), self.colour)
    }
}
