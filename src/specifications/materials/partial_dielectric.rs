//  DIELECTRIC.rs
//    by Lut99
//
//  Description:
//!   Defines a glass-, diamond- or water-like material that refracts when
//!   possible.
//

use serde::{Deserialize, Serialize};

use super::Material;
use crate::math::vec3::dot3;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitRecord;


/***** HELPER FUNCTIONS *****/
/// Default [`Colour`] for [`PartialDielectric`] materials.
#[inline]
const fn default_dielectric_colour() -> Colour { Colour { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }



/// Refrects a ray instead of reflecting it.
///
/// # Arguments
/// - `vec`: The input vector, given as a unit vector.
/// - `normal`: The normal vector to refract against.
/// - `etai_over_etat`: An already computed representation of the refraction index.
///
/// # Returns
/// A refracted ray.
fn refract(vec: Vec3, normal: Vec3, eta_over_eta_prime: f64) -> Vec3 {
    // Implements the following formulas to compute the part of the outgoing ray `perp`endicular to
    // the object and `parallel` to the object:
    //   ray_out_perp = refr_index1/refr_index2(vec + vec.length() * cos_theta * normal)
    //   ray_out_parallel = -sqrt(1 - ray_out_perp.length()^2 * normal)
    //   ray_out = ray_out_perp + ray_out_parallel
    // where
    //   eta_over_eta_prime = refr_index1/refr_index2
    // and
    //   cos_theta = -vec \dot normal
    // (i.e., the angle between the incoming vector to the normal vector)
    let mut cos_theta = dot3(-vec, normal);
    if cos_theta > 1.0 {
        cos_theta = 1.0;
    }
    let ray_out_perp: Vec3 = eta_over_eta_prime * (vec + cos_theta * normal);
    let ray_out_parallel: Vec3 = -((1.0 - ray_out_perp.length2()).abs()).sqrt() * normal;
    return ray_out_perp + ray_out_parallel;
}





/***** LIBRARY *****/
/// A meterial that refrects light instead of (just) reflecting it.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct PartialDielectric {
    /// The refraction index.
    ///
    /// For air, this is 1.000293. For glass, it tends to be 1.5-1.7, and diamond is about 2.4.
    pub refraction_index: f64,
    /// The colour of the glass-like object.
    ///
    /// By default, this is white, since it doesn't attenuate anything.
    #[serde(default = "default_dielectric_colour")]
    pub colour: Colour,
}
impl Material for PartialDielectric {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord) -> (Option<Ray>, Colour) {
        // NOTE: We are always assuming we are refracting against air here
        let eta_over_eta_prime: f64 = if record.front_face { 1.0 / self.refraction_index } else { self.refraction_index };

        // Compute the refraction
        let unit_direction: Vec3 = ray.direct.unit();
        let refracted: Vec3 = refract(unit_direction, record.normal, eta_over_eta_prime);

        // Then bounce the ray
        (Some(Ray::new(record.hit, refracted)), self.colour)
    }
}
