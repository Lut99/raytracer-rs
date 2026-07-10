//  DIELECTRIC.rs
//    by Lut99
//
//  Description:
//!   Defines a glass-, diamond- or water-like material that refracts when
//!   possible.
//

use serde::{Deserialize, Serialize};

use super::super::scene::Environment;
use super::Scattering;
use super::metal::reflect;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitRecord;


/***** HELPER FUNCTIONS *****/
/// Default [`Colour`] for [`PartialDielectric`] materials.
#[inline]
pub const fn default_dielectric_colour() -> Colour { Colour { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }



/// Refrects a ray instead of reflecting it.
///
/// # Arguments
/// - `vec`: The input vector, given as a unit vector.
/// - `normal`: The normal vector to refract against.
/// - `cos_theta`: The computation of `cos(theta)`, where `theta` is the angle between `vec` and
///   `normal`.
/// - `etai_over_etat`: An already computed representation of the refraction index.
///
/// # Returns
/// A refracted ray.
pub fn refract(vec: Vec3, normal: Vec3, cos_theta: f64, eta_over_eta_prime: f64) -> Vec3 {
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
    let ray_out_perp: Vec3 = eta_over_eta_prime * (vec + cos_theta * normal);
    let ray_out_parallel: Vec3 = -((1.0 - ray_out_perp.length2()).abs()).sqrt() * normal;
    return ray_out_perp + ray_out_parallel;
}



/// Uses Christophe Slick's refraction approximation solution to have glass look like a mirror
/// under the right angles.
pub fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
    let r0: f64 = (1.0 - refraction_index) / (1.0 + refraction_index);
    let r02: f64 = r0 * r0;
    // NOTE: Maybe `powf`?
    return r02 + (1.0 - r02) * (1.0 - cosine).powi(5);
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
impl Scattering for PartialDielectric {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) {
        // NOTE: We are always assuming we are refracting against air here
        let eta_over_eta_prime: f64 =
            if record.front_face { env.air_refraction_index / self.refraction_index } else { self.refraction_index / env.air_refraction_index };

        // Compute the refraction
        let unit_direction: Vec3 = ray.direct.unit();
        let mut cos_theta = (-unit_direction).dot(record.normal);
        if cos_theta > 1.0 {
            cos_theta = 1.0;
        }
        let refracted: Vec3 = refract(unit_direction, record.normal, cos_theta, eta_over_eta_prime);

        // Then bounce the ray
        (Some(Ray::new(record.hit, refracted)), self.colour)
    }
}



/// A meterial that refracts light whenever possible instead of (just) reflecting it.
///
/// The latter still happens if the refraction function is unsolvable.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Dielectric {
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
impl Scattering for Dielectric {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) {
        // NOTE: We are always assuming we are refracting against air here
        let eta_over_eta_prime: f64 =
            if record.front_face { env.air_refraction_index / self.refraction_index } else { self.refraction_index / env.air_refraction_index };

        // Determine if we can refract
        let unit_direction: Vec3 = ray.direct.unit();
        let mut cos_theta = (-unit_direction).dot(record.normal);
        if cos_theta > 1.0 {
            cos_theta = 1.0;
        }
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract: bool = eta_over_eta_prime * sin_theta > 1.0;

        // Compute the refraction
        let out: Vec3 = if cannot_refract || reflectance(cos_theta, eta_over_eta_prime) > fastrand::f64() {
            reflect(unit_direction, record.normal)
        } else {
            refract(unit_direction, record.normal, cos_theta, eta_over_eta_prime)
        };

        // Then bounce the ray
        (Some(Ray::new(record.hit, out)), self.colour)
    }
}
