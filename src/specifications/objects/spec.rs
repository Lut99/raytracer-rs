//  SPEC.rs
//    by Lut99
// 
//  Created:
//    01 May 2023, 18:58:37
//  Last edited:
//    01 May 2023, 19:18:53
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines some additional interfaces used commonly in Objects.
// 

use crate::math::{AABB, Ray};


/***** AUXILLARY TYPES *****/
/// Defines everything we want to know about a hit.
#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    /// The closest point on the shot [`Ray`] where it hits the object.
    pub t : f64,
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
    /// 
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else [`None`] if the ray does not hit.
    fn hit(&self, ray: Ray) -> Option<HitRecord>;
}
impl<T: Hittable> Hittable for &T {
    #[inline]
    fn hit(&self, ray: Ray) -> Option<HitRecord> { (**self).hit(ray) }
}
impl<T: Hittable> Hittable for &mut T {
    #[inline]
    fn hit(&self, ray: Ray) -> Option<HitRecord> { (**self).hit(ray) }
}
