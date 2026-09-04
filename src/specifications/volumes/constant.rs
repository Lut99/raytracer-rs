//  CONSTANT.rs
//    by Lut99
//
//  Description:
//!   Defines volumes that have a constant density.
//

use serde::{Deserialize, Serialize};

use super::super::objects::HitData;
use super::Volumizing;
use crate::math::Vec3;
use crate::random;


/***** LIBRARY *****/
/// Defines a constant density volume, i.e., a volume which is just as likely to hit everywhere in it.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ConstantDensity {
    /// The density of the volume.
    pub density: f64,
}

// Interfaces
impl Volumizing for ConstantDensity {
    fn volumize(&self, ray: crate::math::Ray, t1: f64, t2: f64) -> Option<HitData> {
        // Compute a random hitpoint in the gas (or outside of it)
        let ray_len: f64 = ray.direct.length();
        let dist_in_boundary: f64 = (t2 - t1) * ray_len;
        let hit_dist: f64 = (-1.0 / self.density) * random::f64().ln();
        if hit_dist > dist_in_boundary {
            // No hit, the ray passes through.
            // Unless...? - if the shape is not convex, it may re-enter the material here!
            // TODO
            return None;
        }

        // Else, we compute a hit with a random scatter (the material)
        let t: f64 = t1 + hit_dist / ray_len;
        // NOTE: The last two values are arbitrary for gasses.
        Some(HitData::new(ray, ray.at(t), t, Vec3::new(1.0, 0.0, 0.0), (0.0, 0.0)))
    }
}
