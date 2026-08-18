//  BOX.rs
//    by Lut99
//
//  Description:
//!   Abstraction over a couple of quads to call it a box.
//

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::{BoundingBoxable, HitRecord, Hittable};
use crate::math::{AABB, Ray};
use crate::specifications::scene::Environment;


/***** LIBRARY *****/
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Box<M> {
    /// The interal AABB we use to render.
    #[serde(flatten)]
    pub aabb:     AABB,
    /// The material to render all quads with.
    #[serde(alias = "mat")]
    pub material: M,
}

// Interface
impl<M: Loadable> Loadable for Box<M> {
    type Error = M::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.material.load(dir) }
}
impl<M> BoundingBoxable for Box<M> {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { self.aabb }
}
impl<M> Hittable<M> for Box<M> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ M>> {
        self.aabb.hit(ray, t_min, t_max, env).map(|rec| HitRecord { mat: &self.material, data: rec.data })
    }
}
