//  MEDIUMS.rs
//    by Lut99
//
//  Description:
//!   Defines gasses.
//!   
//!   This sits somewhere in between an object and a material; it overlays an existing object to
//!   use its shape as the boundary of the gas.
//

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::materials::Isotropic;
use super::super::objects::{HitData, HitRecord};
use super::{BoundingBoxable, Hittable};
use crate::math::{Ray, Vec3};
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
/// A volume with constant density.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ConstantDensity<T> {
    /// The boundary is another object that defines the shape of the cloud.
    pub boundary: T,
    /// The density is the density of the gass.
    pub density: f64,
    /// The phase function determines how the gas is bounced.
    #[serde(flatten)]
    pub phase_function: Isotropic,
}

// Interface
impl<T: Loadable> Loadable for ConstantDensity<T> {
    type Error = T::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.boundary.load(dir) }
}
impl<T: BoundingBoxable> BoundingBoxable for ConstantDensity<T> {
    #[inline]
    fn aabb(&self, t_us: u64) -> crate::math::AABB { self.boundary.aabb(t_us) }
}
impl<T: Hittable> Hittable for ConstantDensity<T> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<'_>> {
        // Check if the ray hits the boundary on _two_ points (in- and out of the gas)
        let mut rec1: HitData = self.boundary.hit(ray, -f64::INFINITY, f64::INFINITY, env)?.data;
        let mut rec2: HitData = self.boundary.hit(ray, rec1.t + 0.0001, f64::INFINITY, env)?.data;

        // Bound the record's t's by the given ones and quit if it's too close
        rec1.t = f64::max(rec1.t, t_min);
        rec2.t = f64::min(rec2.t, t_max);
        if rec1.t >= rec2.t {
            return None;
        }
        rec1.t = f64::max(rec1.t, 0.0);

        // Compute a random hitpoint in the gas (or outside of it)
        let ray_len: f64 = ray.direct.length();
        let dist_in_boundary: f64 = (rec2.t - rec1.t) * ray_len;
        let hit_dist: f64 = (-1.0 / self.density) * fastrand::f64().ln();
        if hit_dist > dist_in_boundary {
            // No hit, the ray passes through.
            // Unless...? - if the shape is not convex, it may re-enter the material here!
            // TODO
            return None;
        }

        // Else, we compute a hit with a random scatter (the material)
        let t: f64 = rec1.t + hit_dist / ray_len;
        // NOTE: The last two values are arbitrary for gasses.
        Some(HitRecord { mat: &self.phase_function, data: HitData::new(ray, ray.at(t), t, Vec3::new(1.0, 0.0, 0.0), (0.0, 0.0)) })
    }
}
