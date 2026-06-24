//  METAL.rs
//    by Lut99
//
//  Description:
//!   Defines a metallic material that reflects more directly and less random.
//

use serde::{Deserialize, Serialize};

use super::super::objects::HitRecord;
use super::super::scene::Environment;
use super::Material;
use super::diffuse::random3_uniform;
use crate::math::vec3::dot3;
use crate::math::{Colour, Ray, Vec3};


/***** HELPER FUNCTIONS *****/
/// Reflects a vector based on the direction it came in and the normal vector.
#[inline]
pub fn reflect(vec: Vec3, norm: Vec3) -> Vec3 { vec - 2.0 * dot3(vec, norm) * norm }





/***** LIBRARY *****/
/// Implements a material that will reflect more linearly.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Metal {
    /// The colour of the material.
    pub colour: Colour,
    /// How much "fuzziness" to apply.
    ///
    /// A value of 0 means perfectly reflecting metal, like a mirror.
    #[serde(default)]
    pub fuzz:   f64,
}
impl Material for Metal {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord, _env: &Environment) -> (Option<Ray>, Colour) {
        // Compute the scattered ray, making sure the scattered one is not zero
        let reflected: Vec3 = reflect(ray.direct, record.normal);
        // Add some fuzz by offsetting the endpoint of the reflected vector by a small amount.
        // This is done by randomly choosing a small vector on a sphere (who's radius is `fuzz`)
        // and then adding it to the end vector.
        let reflected: Vec3 = reflected.unit() + self.fuzz * random3_uniform();

        // Now we can simply return the new ray to bounce and the colour
        (Some(Ray::new(record.hit, reflected)), self.colour)
    }
}
