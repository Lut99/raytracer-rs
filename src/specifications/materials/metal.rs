//  METAL.rs
//    by Lut99
//
//  Description:
//!   Defines a metallic material that reflects more directly and less random.
//

use serde::{Deserialize, Serialize};

use super::Material;
use crate::math::vec3::dot3;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitRecord;


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
}
impl Material for Metal {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord) -> (Option<Ray>, Colour) {
        // Compute the scattered ray, making sure the scattered one is not zero
        let reflected: Vec3 = reflect(ray.direct, record.normal);

        // Now we can simply return the new ray to bounce and the colour
        (Some(Ray::new(record.hit, reflected)), self.colour)
    }
}
