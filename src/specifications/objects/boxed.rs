//  BOX.rs
//    by Lut99
//
//  Description:
//!   Abstraction over a couple of quads to call it a box.
//

use std::convert::Infallible;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::pdf::PDF;
use super::{BoundingBoxable, HitData, Hittable};
use crate::math::{AABB, Ray, Vec3};


/***** LIBRARY *****/
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Box {
    /// The interal AABB we use to render.
    #[serde(flatten)]
    pub aabb: AABB,
}

// Interface
impl Loadable for Box {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl BoundingBoxable for Box {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { self.aabb }
}
impl PDF for Box {
    #[inline]
    fn value(&self, direct: Ray, env: &Environment) -> f64 { self.aabb.value(direct, env) }

    #[inline]
    fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 { self.aabb.sample(t_us, origin) }
}
impl Hittable for Box {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> { self.aabb.hit(ray, t_min, t_max, env) }
}
