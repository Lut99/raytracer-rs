//  AABB.rs
//    by Lut99
// 
//  Created:
//    30 Apr 2023, 11:49:29
//  Last edited:
//    30 Apr 2023, 12:28:06
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines an Axis-Aligned Bounding Box ([`AABB`]) that we can use to
//!   cheaply pre-check if we have to do the expensive hit computations
//!   for a given object or a group of objects.
// 

use std::mem;

use super::vec3::Vec3;
use super::ray::Ray;


/***** AUXILLARY FUNCTIONS *****/
/// Computes a bounding box surrounding the two given ones.
/// 
/// # Arguments
/// - `b1`: One of the two boxes two surround.
/// - `b2`: The other of the two boxes to surround.
/// 
/// # Returns
/// A new [`AABB`] that perfectly fits the two other boxes.
#[inline]
pub fn surround(b1: AABB, b2: AABB) -> AABB {
    AABB::new(
        Vec3::new(
            f64::min(b1.a.x, b2.a.x),
            f64::min(b1.a.y, b2.a.y),
            f64::min(b1.a.z, b2.a.z),
        ),
        Vec3::new(
            f64::min(b1.b.x, b2.b.x),
            f64::min(b1.b.y, b2.b.y),
            f64::min(b1.b.z, b2.b.z),
        ),
    )
}





/***** LIBRARY *****/
/// The Axis-Aligned Bounding Box (AABB) can be used to cheaply pre-check if we roughly hit an object.
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    /// The top-left point of the box, or at least, the opposite corner of point `b`.
    pub a : Vec3,
    /// The bottom-right point of the box, or at least, the opposite corner of point `a`.
    pub b : Vec3,
}

impl AABB {
    /// Constructor for the AABB.
    /// 
    /// # Arguments
    /// - `a`: The first point of the box.
    /// - `b`: The second point of the box, which is on the opposite side as point `a` on all axis.
    /// 
    /// # Returns
    /// A new instance of an AABB.
    pub fn new(a: impl Into<Vec3>, b: impl Into<Vec3>) -> Self {
        Self {
            a : a.into(),
            b : b.into(),
        }
    }



    /// Computes a hit with a given ray.
    /// 
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute a hit with.
    /// - `t_min`: A minimal `t` (i.e., distance along the Ray from its origin) that we accept.
    /// - `t_max`: A maximal `t` (i.e., distance along the Ray from its origin) that we accept.
    /// 
    /// # Returns
    /// Whether the given ray hits this AABB.
    #[inline]
    pub fn hit(&self, ray: Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for i in 0..3 {
            // Compute the hit points with the AABB
            let inv_direction: f64 = 1.0 / ray.direct[i];
            let mut t0: f64 = (self.a[i] - ray.origin[i]) * inv_direction;
            let mut t1: f64 = (self.b[i] - ray.origin[i]) * inv_direction;

            // Ensure we order the values properly, and then bind them by the given min/max
            if inv_direction < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }
            t_min = t0.clamp(t_min, f64::INFINITY);
            t_max = t1.clamp(-f64::INFINITY, t_max);

            // We don't hit if t_max is now too small
            if t_max <= t_min { return false; }
        }
        true
    }
}
