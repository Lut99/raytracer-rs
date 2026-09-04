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
    fn transform_aabb(&self, mut aabb: crate::math::AABB) -> crate::math::AABB {
        aabb.x = aabb.x.translate(self.pos.x);
        aabb.y = aabb.y.translate(self.pos.y);
        aabb.z = aabb.z.translate(self.pos.z);
        aabb
    }

    #[inline]
    fn transform(&self, mut ray: Ray) -> Ray {
        ray.origin -= self.pos;
        ray
    }

    #[inline]
    fn transform_back(&self, mut rec: HitData) -> HitData {
        rec.hit += self.pos;
        rec
    }
}
