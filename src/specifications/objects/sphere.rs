//  SPHERE.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:56:14
//  Last edited:
//    06 May 2023, 11:44:25
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a renderable [`Sphere`].
// 

use serde::{Deserialize, Serialize};

use crate::math::{AABB, Ray, Vec3, Vector as _};
use crate::math::vec3::dot3;

use super::spec::{BoundingBoxable, HitRecord, Hittable};


/***** LIBRARY *****/
/// Defines a perfect sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere<M> {
    /// The center point of the sphere.
    pub center : Vec3,
    /// The radius of the sphere.
    pub radius : f64,

    /// The material the sphere is composed of.
    pub material : M,
}

impl<M> BoundingBoxable for Sphere<M> {
    #[inline]
    fn aabb(&self) -> AABB { AABB::new(self.center - self.radius, self.center + self.radius) }
}
impl<M> Hittable for Sphere<M> {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // Compute the distance between the origin of the ray and the center of the sphere
        let oc: Vec3 = ray.origin - self.center;

        // We compute `a`, `b` and `c` in the classic ABC-formula. This we do to find the intersections between the Ray (origin + t*direction) and the sphere (x^2 + y^2 + z^2 = r^2).
        // For more explanation, see the tutorial (<https://raytracing.github.io/books/RayTracingInOneWeekend.html#addingasphere/ray-sphereintersection>)
        let a      : f64 = ray.direct.length2();
        let half_b : f64 = dot3(oc, ray.direct);
        let c      : f64 = oc.length2() - self.radius * self.radius;

        // Compute the discriminant only, since we're only interested in the number of roots
        // D < 0 -> no intersection, D == 0 -> one intersection (touching side), D > 0 -> two intersections (passing through)
        let d: f64 = half_b*half_b - a*c;
        if d >= 0.0 {
            let sqrtd: f64 = d.sqrt();

            // Compute the t by filling in the (optimized) ABC formula and assert it is within t_min and t_max
            let mut root : f64  = (-half_b - sqrtd) / a;
            if root < t_min || root > t_max {
                // Re-try with the other D option
                root = (-half_b + sqrtd) / a;
                if root < t_min || root > t_max { return None; }
            }

            // Compute the outward normal, i.e., the normal that always points upward from the sphere
            // Note: we divide by the radius to make it a unit sphere (since the hitpoint is guaranteed to be on the sphere itself)
            let hit  : Vec3 = ray.at(root);
            let outward_normal: Vec3 = (hit - self.center) / self.radius;

            // Populate the rest of the hitrecord on the fly
            Some(HitRecord::new(
                ray,
                hit,
                root,
                outward_normal,
            ))
        } else {
            None
        }
    }
}
