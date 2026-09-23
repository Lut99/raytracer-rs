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

use std::convert::Infallible;
use std::f64::consts::PI;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::hitrecord::HitData;
use super::pdf::PDF;
use super::{BoundingBoxable, Hittable};
use crate::math::{AABB, ONB, Ray, Vec3};
use crate::random;


/***** HELPER FUNCTIONS *****/
/// Computes a sphere's AABB.
#[inline]
fn sphere_aabb(center: Vec3, radius: f64) -> AABB {
    let aabb = AABB::from_points(center - radius, center + radius);
    aabb
}

/// Computes the uv-coordinate pair on a sphere given a normal point (and direction) on it.
#[inline]
fn sphere_uv(p: Vec3) -> (f64, f64) {
    // Compute the polar coordinates on the sphere
    let theta = (-p.y).acos();
    let phi = (-p.z).atan2(p.x) + PI;
    (phi / (2.0 * PI), theta / PI)
}

/// Computes a sphere's hit yay or nay.
#[inline]
fn sphere_hit(center: Vec3, radius: f64, ray: Ray, t_min: f64, t_max: f64) -> Option<HitData> {
    // Compute the distance between the origin of the ray and the center of the sphere
    let oc: Vec3 = ray.origin - center;

    // We compute `a`, `b` and `c` in the classic ABC-formula. This we do to find the intersections between the Ray (origin + t*direction) and the sphere (x^2 + y^2 + z^2 = r^2).
    // For more explanation, see the tutorial (<https://raytracing.github.io/books/RayTracingInOneWeekend.html#addingasphere/ray-sphereintersection>)
    let a: f64 = ray.direct.length2();
    let half_b: f64 = oc.dot(ray.direct);
    let c: f64 = oc.length2() - radius * radius;

    // Compute the discriminant only, since we're only interested in the number of roots
    // D < 0 -> no intersection, D == 0 -> one intersection (touching side), D > 0 -> two intersections (passing through)
    let d: f64 = half_b * half_b - a * c;
    if d >= 0.0 {
        let sqrtd: f64 = d.sqrt();

        // Compute the t by filling in the (optimized) ABC formula and assert it is within t_min and t_max
        let mut root: f64 = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            // Re-try with the other D option
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        // Compute the outward normal, i.e., the normal that always points upward from the sphere
        // Note: we divide by the radius to make it a unit sphere (since the hitpoint is guaranteed to be on the sphere itself)
        let hit: Vec3 = ray.at(root);
        let outward_normal: Vec3 = (hit - center) / radius;

        // Populate the rest of the hitrecord on the fly
        Some(HitData::new(ray, hit, root, outward_normal, sphere_uv(outward_normal)))
    } else {
        None
    }
}



/// Generates a random point on the facing side of a sphere.
pub fn random3_to_sphere(radius: f64, dist_sqrt: f64) -> Vec3 {
    let r1 = random::f64();
    let r2 = random::f64();
    let z = 1.0 + r2 * ((1.0 - radius * radius / dist_sqrt).sqrt() - 1.0);

    let phi = 2.0 * PI * r1;
    let x = phi.cos() * (1.0 - z * z).sqrt();
    let y = phi.sin() * (1.0 - z * z).sqrt();

    Vec3::new(x, y, z)
}





/***** LIBRARY *****/
/// Defines a perfect sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere {
    /// The center point of the sphere.
    pub center: Vec3,
    /// The radius of the sphere.
    pub radius: f64,
}

impl Loadable for Sphere {
    type Error = Infallible;

    #[inline]
    fn load(&mut self, _dir: &Path) -> Result<(), Self::Error> { Ok(()) }
}
impl BoundingBoxable for Sphere {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { sphere_aabb(self.center, self.radius) }
}
impl PDF for Sphere {
    #[inline]
    fn value(&self, direct: Ray, env: &Environment) -> f64 {
        // Do the expensive, harder test
        if self.hit(direct, 0.001, f64::INFINITY, env).is_none() {
            return 0.0;
        }

        let dist_sqrt = (self.center - direct.origin).length2();
        let cos_theta_max = (1.0 - self.radius * self.radius / dist_sqrt).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);
        1.0 / solid_angle
    }

    #[inline]
    fn sample(&self, _t_us: u64, origin: Vec3) -> Vec3 {
        let direct = self.center - origin;
        let dist_sqrt = direct.length2();
        let uwv = ONB::from_single_axis(direct);
        return uwv.transform(random3_to_sphere(self.radius, dist_sqrt));
    }
}
impl Hittable for Sphere {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitData> { sphere_hit(self.center, self.radius, ray, t_min, t_max) }
}





// /***** TESTS *****/
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::specifications::animations::Vertical;

//     #[test]
//     fn test_sphere_aabb() {
//         let sphere = Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 0.5 };
//         assert_eq!(sphere.aabb(0), AABB::from_points([-0.5, -0.5, -0.5].into(), [0.5, 0.5, 0.5].into()));
//     }

//     #[test]
//     fn test_animated_sphere_aabb() {
//         let sphere = AnimatedSphere {
//             sphere:    Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 0.5 },
//             animation: Vertical { len: 100.0, at: 0, duration: 100 },
//         };
//         assert_eq!(sphere.aabb(0), AABB::from_points([-0.5, -0.5, -0.5].into(), [0.5, 0.5, 0.5].into()));
//         assert_eq!(sphere.aabb(50), AABB::from_points([-0.5, 49.5, -0.5].into(), [0.5, 50.5, 0.5].into()));
//         assert_eq!(sphere.aabb(100), AABB::from_points([-0.5, 99.5, -0.5].into(), [0.5, 100.5, 0.5].into()));
//         assert_eq!(sphere.aabb(150), AABB::from_points([-0.5, 99.5, -0.5].into(), [0.5, 100.5, 0.5].into()));
//         assert_eq!(AABB::surround(sphere.aabb(0), sphere.aabb(100)), AABB::from_points([-0.5, -0.5, -0.5].into(), [0.5, 100.5, 0.5].into()));
//     }
// }
