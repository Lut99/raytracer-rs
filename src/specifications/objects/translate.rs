//  TRANSLATE.rs
//    by Lut99
//
//  Description:
//!   Implements "objects" that take other objects and shows translations or
//!   transforms.
//

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::{BoundingBoxable, HitRecord, Hittable};
use crate::math::{AABB, Ray, Vec3};


/***** LIBRARY *****/
// /// Changes the rendered size of an object.
// #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
// pub struct Scale<T> {
//     /// The nested object.
//     pub obj:   T,
//     /// The offset vector.
//     pub scale: f64,
// }

// // Interfaces
// impl<T: Loadable> Loadable for Scale<T> {
//     type Error = T::Error;

//     #[inline]
//     fn load(&mut self) -> Result<(), Self::Error> { self.obj.load() }
// }
// impl<T: BoundingBoxable> BoundingBoxable for Scale<T> {
//     #[inline]
//     fn aabb(&self, t_us: u64) -> AABB {
//         let mut aabb: AABB = self.obj.aabb(t_us);
//         aabb.dims[0] *= self.scale;
//         aabb.dims[1] *= self.scale;
//         aabb.dims[2] *= self.scale;
//         aabb
//     }
// }
// impl<T: Hittable<M>, M> Hittable<M> for Scale<T> {
//     #[inline]
//     fn hit(&self, mut ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ M>> {
//         ray.origin
//         let mut rec: HitRecord<&M> = self.obj.hit(ray, t_min, t_max, env)?;
//         rec.data.hit += self.pos;
//         Some(rec)
//     }
// }



/// Defines a positional translation on an object.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Translate<T> {
    /// The nested object.
    pub obj: T,
    /// The offset vector.
    pub pos: Vec3,
}

// Interfaces
impl<T: Loadable> Loadable for Translate<T> {
    type Error = T::Error;

    #[inline]
    fn load(&mut self) -> Result<(), Self::Error> { self.obj.load() }
}
impl<T: BoundingBoxable> BoundingBoxable for Translate<T> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB {
        let mut aabb: AABB = self.obj.aabb(t_us);
        aabb.pos += self.pos;
        aabb
    }
}
impl<T: Hittable<M>, M> Hittable<M> for Translate<T> {
    #[inline]
    fn hit(&self, mut ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&'_ M>> {
        ray.origin -= self.pos;
        let mut rec: HitRecord<&M> = self.obj.hit(ray, t_min, t_max, env)?;
        rec.data.hit += self.pos;
        Some(rec)
    }
}
