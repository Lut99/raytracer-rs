//  SPHERE.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:56:14
//  Last edited:
//    01 May 2023, 19:18:09
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines a renderable [`Sphere`].
// 

use serde::{Deserialize, Serialize};

use crate::math::{AABB, Vec3, Vector as _};
use crate::math::vec3::dot3;

use super::spec::{BoundingBoxable, HitRecord, Hittable};


/***** LIBRARY *****/
/// Defines a perfect sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere {
    /// The center point of the sphere.
    pub center : Vec3,
    /// The radius of the sphere.
    pub radius : f64,
}

impl BoundingBoxable for Sphere {
    #[inline]
    fn aabb(&self) -> crate::math::AABB { AABB::new(self.center - self.radius, self.center + self.radius) }
}
impl Hittable for Sphere {
    fn hit(&self, ray: crate::math::Ray) -> Option<HitRecord> {
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
            Some(HitRecord {
                t : (-half_b - d.sqrt()) / a,
            })
        } else {
            None
        }
    }
}
