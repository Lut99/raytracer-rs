//  TRANSLATE.rs
//    by Lut99
//
//  Description:
//!   Defines positional [transformations](`Transforming`).
//

use serde::{Deserialize, Serialize};

use super::super::objects::HitData;
use super::Transforming;
use crate::math::{Ray, Vec3};


/***** AUXILLARY *****/
/// Defines the possible trajectories that an [`AnimatedTranslate`] can have.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Trajectory {
    /// It's a simple, vertical trajectory that is linearly walked and finishes at the duration.
    Vertical { len: f64 },
}





/***** LIBRARY *****/
/// Defines a positional translation on an object.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename = "translate")]
pub struct Translate {
    /// The offset vector.
    #[serde(alias = "offset")]
    pub pos: Vec3,
}

// Interfaces
impl Transforming for Translate {
    #[inline]
    fn transform_aabb(&self, _t_us: u64, mut aabb: crate::math::AABB) -> crate::math::AABB {
        aabb.x = aabb.x.translate(self.pos.x);
        aabb.y = aabb.y.translate(self.pos.y);
        aabb.z = aabb.z.translate(self.pos.z);
        aabb
    }

    #[inline]
    fn transform_vec3_obj(&self, _t_us: u64, vec: Vec3) -> Vec3 { vec - self.pos }

    #[inline]
    fn transform_vec3_world(&self, _t_us: u64, vec: Vec3) -> Vec3 { vec + self.pos }

    #[inline]
    fn transform_ray_obj(&self, mut ray: Ray) -> Ray {
        ray.origin -= self.pos;
        ray
    }

    #[inline]
    fn transform_rec_world(&self, mut rec: HitData) -> HitData {
        rec.hit += self.pos;
        rec
    }
}



/// A positional translation that changes over time.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename = "animated_translate")]
pub struct AnimatedTranslate {
    /// The trajectory determines the evolution of the path over time.
    pub trajectory: Trajectory,
    /// The start time of the translation.
    pub at: u64,
    /// The duration of the animation, in us.
    pub duration: u64,
}

// Animation
impl AnimatedTranslate {
    /// Computes a position delta for this timestamp.
    pub fn compute_dpos(&self, t_us: u64) -> Vec3 {
        // Check we're in range
        if t_us < self.at || t_us > self.at + self.duration {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        // Compute the fraction of time passed
        let f: f64 = (t_us - self.at) as f64 / self.duration as f64;

        // Use that to update the position
        match self.trajectory {
            Trajectory::Vertical { len } => Vec3::new(0.0, f * len, 0.0),
        }
    }
}

// Interfaces
impl Transforming for AnimatedTranslate {
    #[inline]
    fn transform_aabb(&self, t_us: u64, mut aabb: crate::math::AABB) -> crate::math::AABB {
        // Compute the position for this timestep
        let dpos: Vec3 = self.compute_dpos(t_us);

        // Run the update
        aabb.x = aabb.x.translate(dpos.x);
        aabb.y = aabb.y.translate(dpos.y);
        aabb.z = aabb.z.translate(dpos.z);
        aabb
    }

    #[inline]
    fn transform_vec3_obj(&self, t_us: u64, vec: Vec3) -> Vec3 {
        // Compute the position for this timestep
        let dpos: Vec3 = self.compute_dpos(t_us);

        // Run the update
        vec - dpos
    }

    #[inline]
    fn transform_vec3_world(&self, t_us: u64, vec: Vec3) -> Vec3 {
        // Compute the position for this timestep
        let dpos: Vec3 = self.compute_dpos(t_us);

        // Run the update
        vec + dpos
    }

    #[inline]
    fn transform_ray_obj(&self, mut ray: Ray) -> Ray {
        // Compute the position for this timestep
        let dpos: Vec3 = self.compute_dpos(ray.time);

        // Run the update
        ray.origin -= dpos;
        ray
    }

    #[inline]
    fn transform_rec_world(&self, mut rec: HitData) -> HitData {
        // Compute the position for this timestep
        let dpos: Vec3 = self.compute_dpos(rec.time);

        // Run the update
        rec.hit += dpos;
        rec
    }
}
