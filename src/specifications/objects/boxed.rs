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
use super::{BoundingBoxable, HitData, Hittable};
use crate::math::{AABB, Ray};
use crate::specifications::scene::Environment;


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
impl Hittable for Box {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> { self.aabb.hit(ray, t_min, t_max, env) }
}
