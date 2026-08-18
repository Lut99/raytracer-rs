//  TRANSLATE.rs
//    by Lut99
//
//  Description:
//!   Implements "objects" that take other objects and shows translations or
//!   transforms.
//

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::{BoundingBoxable, HitRecord, Hittable};
use crate::math::camera::degrees_to_radians;
use crate::math::{AABB, Ray, Vec3};


/***** HELPER FUNCTIONS *****/
/// Rotates a vector around the Y-axis.
#[inline]
fn rotate_y(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x - sin_theta * vec.z, vec.y, sin_theta * vec.x + cos_theta * vec.z)
}
/// Rotates a vector back around the Y-axis.
#[inline]
fn rotate_y_back(vec: Vec3, sin_theta: f64, cos_theta: f64) -> Vec3 {
    Vec3::new(cos_theta * vec.x + sin_theta * vec.z, vec.y, -sin_theta * vec.x + cos_theta * vec.z)
}





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
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.obj.load(dir) }
}
impl<T: BoundingBoxable> BoundingBoxable for Translate<T> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB {
        let mut aabb: AABB = self.obj.aabb(t_us);
        aabb.x = aabb.x.translate(self.pos.x);
        aabb.y = aabb.y.translate(self.pos.y);
        aabb.z = aabb.z.translate(self.pos.z);
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



/// Implements rotation around the Y-axis.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RotateY<T> {
    /// The nested object.
    pub obj:   T,
    /// The angle, in degrees.
    pub angle: f64,
}

// Interfaces
impl<T: Loadable> Loadable for RotateY<T> {
    type Error = T::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.obj.load(dir) }
}
impl<T: BoundingBoxable> BoundingBoxable for RotateY<T> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB {
        // Compute the sin_theta and cos_theta for this angle
        let angle_radians: f64 = degrees_to_radians(self.angle);
        let sin_theta: f64 = angle_radians.sin();
        let cos_theta: f64 = angle_radians.cos();
        let aabb: AABB = self.obj.aabb(t_us);

        // Compute the translated points of the box and find min & max of those
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    // Compute the original position of this corner
                    let pos = Vec3::new(
                        (i as f64 * aabb.x.max()) + (1.0 - i as f64) * aabb.x.min(),
                        (j as f64 * aabb.y.max()) + (1.0 - j as f64) * aabb.y.min(),
                        (k as f64 * aabb.z.max()) + (1.0 - k as f64) * aabb.z.min(),
                    );

                    // Rotate the point
                    let rotpos = rotate_y_back(pos, sin_theta, cos_theta);

                    // Consider if it's a boundary
                    for c in 0..3 {
                        min[c] = f64::min(min[c], rotpos[c]);
                        max[c] = f64::max(max[c], rotpos[c]);
                    }
                }
            }
        }

        // Done, return the surrounding AABB
        AABB::from_points(min, max)
    }
}
impl<T: Hittable<M>, M> Hittable<M> for RotateY<T> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&M>> {
        // Compute the sin_theta and cos_theta for this angle
        let angle_radians: f64 = degrees_to_radians(self.angle);
        let sin_theta: f64 = angle_radians.sin();
        let cos_theta: f64 = angle_radians.cos();

        // Transform the ray from world space to object space
        let origin = rotate_y(ray.origin, sin_theta, cos_theta);
        let direct = rotate_y(ray.direct, sin_theta, cos_theta);
        let rotated_ray = Ray::with_time(origin, direct, ray.time);

        // Determine the intersection in object space and quit if it doesn't hit
        let mut rec: HitRecord<&M> = self.obj.hit(rotated_ray, t_min, t_max, env)?;

        // Rotate the answer back to normal space
        rec.data.hit = rotate_y_back(rec.data.hit, sin_theta, cos_theta);
        rec.data.normal = rotate_y_back(rec.data.normal, sin_theta, cos_theta);

        // And that's it!
        Some(rec)
    }
}
