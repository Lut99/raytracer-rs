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

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::animations::{Animating, Animation};
use super::super::materials::Scattering;
use super::super::scene::Environment;
use super::hitrecord::HitRecord;
use super::{BoundingBoxable, Hittable};
use crate::math::{AABB, Colour, Ray, Vec3};


/***** HELPER FUNCTIONS *****/
/// Computes a sphere's AABB.
#[inline]
fn sphere_aabb(center: Vec3, radius: f64) -> AABB { AABB::new(center - radius, [2.0 * radius; 3]) }

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
fn sphere_hit(center: Vec3, radius: f64, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        Some(HitRecord::new(ray, hit, root, outward_normal, sphere_uv(outward_normal)))
    } else {
        None
    }
}





/***** LIBRARY *****/
/// Defines a perfect sphere.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Sphere<M> {
    /// The center point of the sphere.
    pub center: Vec3,
    /// The radius of the sphere.
    pub radius: f64,

    /// The material the sphere is composed of.
    #[serde(alias = "mat")]
    pub material: M,
}

impl<M: Loadable> Loadable for Sphere<M> {
    type Error = M::Error;

    #[inline]
    fn load(&mut self) -> Result<(), Self::Error> { self.material.load() }
}
impl<M> BoundingBoxable for Sphere<M> {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { sphere_aabb(self.center, self.radius) }
}
impl<M> Hittable for Sphere<M> {
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitRecord> {
        sphere_hit(self.center, self.radius, ray, t_min, t_max)
    }
}
impl<M: Scattering> Scattering for Sphere<M> {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) { self.material.scatter(ray, record, env) }
}



/// Defines an animated sphere.
///
/// This is a regular [`Sphere`] wrapped to do some movement.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AnimatedSphere<M, A = Animation> {
    /// The sphere we wrap
    pub sphere:    Sphere<M>,
    /// The animation that determines how the sphere moves over time.
    pub animation: A,
}

impl<M: Loadable> Loadable for AnimatedSphere<M> {
    type Error = M::Error;

    #[inline]
    fn load(&mut self) -> Result<(), Self::Error> { self.sphere.load() }
}
impl<M, A: Animating> BoundingBoxable for AnimatedSphere<M, A> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB { sphere_aabb(self.animation.animate(self.sphere.center, t_us), self.sphere.radius) }
}
impl<M, A: Animating> Hittable for AnimatedSphere<M, A> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitRecord> {
        sphere_hit(self.animation.animate(self.sphere.center, ray.time), self.sphere.radius, ray, t_min, t_max)
    }
}
impl<M: Scattering, A> Scattering for AnimatedSphere<M, A> {
    #[inline]
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) { self.sphere.scatter(ray, record, env) }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::specifications::animations::Vertical;
    use crate::specifications::materials::NormalMap;

    #[test]
    fn test_sphere_aabb() {
        let sphere = Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 0.5, material: NormalMap };
        assert_eq!(sphere.aabb(0), AABB::new([-0.5, -0.5, -0.5].into(), [1.0, 1.0, 1.0]));
    }

    #[test]
    fn test_animated_sphere_aabb() {
        let sphere = AnimatedSphere {
            sphere:    Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 0.5, material: NormalMap },
            animation: Vertical { len: 100.0, at: 0, duration: 100 },
        };
        assert_eq!(sphere.aabb(0), AABB::new([-0.5, -0.5, -0.5].into(), [1.0, 1.0, 1.0]));
        assert_eq!(sphere.aabb(50), AABB::new([-0.5, 49.5, -0.5].into(), [1.0, 1.0, 1.0]));
        assert_eq!(sphere.aabb(100), AABB::new([-0.5, 99.5, -0.5].into(), [1.0, 1.0, 1.0]));
        assert_eq!(sphere.aabb(150), AABB::new([-0.5, 99.5, -0.5].into(), [1.0, 1.0, 1.0]));
        assert_eq!(AABB::surround(sphere.aabb(0), sphere.aabb(100)), AABB::new([-0.5, -0.5, -0.5].into(), [1.0, 101.0, 1.0]));
    }
}
