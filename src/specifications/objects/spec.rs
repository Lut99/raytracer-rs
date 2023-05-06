//  SPEC.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:58:37
//  Last edited:
//    06 May 2023, 11:43:06
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines some additional interfaces used commonly in Objects.
// 

use crate::math::{AABB, Ray, Vec3};
use crate::math::vec3::dot3;


/***** AUXILLARY TYPES *****/
/// Defines everything we want to know about a hit.
#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    /// The (closest) point where the [`Ray`] hits an object.
    pub hit        : Vec3,
    /// The (closest) point where the [`Ray`] hits an object, given as distance from the ray's origin.
    pub t          : f64,
    /// The surface normal on the point we are hitting.
    pub normal     : Vec3,
    /// Whether we are hitting the front face of the object or the backface.
    pub front_face : bool,
}
impl HitRecord {
    /// Constructor for the HitRecord that compute the internal `hit`, `normal` and `front_face` from the given ray, hit distance on that ray and outward normal.
    /// 
    /// # Arguments
    /// - `ray`: The [`Ray`] which hits an object.
    /// - `hit`: The physical point where we hit the object. Probably computed as [`Ray::at()`], but we leave this for the caller since they typically need this point to compute the normal.
    /// - `t`: The distance from the `ray`'s origin, along the ray, which hits the object.
    /// - `outward_normal`: The outward facing normal that we will store but tweaked so it's always in the direction of the `ray`.
    /// 
    /// # Returns
    /// A new `HitRecord` with the math taken care of.
    pub fn new(ray: Ray, hit: Vec3, t: f64, outward_normal: Vec3) -> Self {
        // Compute the normal from the outward normal, remembering the direction
        let front_face : bool = dot3(ray.direct, outward_normal) < 0.0;
        let normal     : Vec3 = if front_face { outward_normal } else { -outward_normal };

        // Return ourselves
        Self {
            hit,
            t,
            normal,
            front_face,
        }
    }
}





/***** LIBRARY *****/
/// Defines a common interface for objects that can compute a sensible [`AABB`].
pub trait BoundingBoxable {
    /// Computes the Axis-Aligned Bounding Box (AABB) of this object.
    /// 
    /// # Returns
    /// A new [`AABB`] struct that describes the computed bounding box.
    fn aabb(&self) -> AABB;
}
impl<T: BoundingBoxable> BoundingBoxable for &T {
    #[inline]
    fn aabb(&self) -> AABB { (**self).aabb() }
}
impl<T: BoundingBoxable> BoundingBoxable for &mut T {
    #[inline]
    fn aabb(&self) -> AABB { (**self).aabb() }
}



/// Defines the functions that hittable objects have in common.
pub trait Hittable: BoundingBoxable {
    /// Computes any hitpoints of the given ray with this object.
    /// 
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute any hits with.
    /// - `t_min`: The minimum point along the ray we still accept (we don't count it as a hit before that).
    /// - `t_max`: The maximum point along the ray we still accept (we don't count is as a hit after that).
    /// 
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else [`None`] if the ray does not hit.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
impl<T: Hittable> Hittable for &T {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> { (**self).hit(ray, t_min, t_max) }
}
impl<T: Hittable> Hittable for &mut T {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> { (**self).hit(ray, t_min, t_max) }
}
