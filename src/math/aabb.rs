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

use super::ray::Ray;
use super::vec3::Vec3;
use crate::specifications::objects::{BoundingBoxable, HitRecord, Hittable};
use crate::specifications::scene::Environment;


/***** CONSTANTS *****/
/// Determines the minimum size for every of [`AABB`]'s dimensions.
pub const AABB_MIN_DIM_LEN: f64 = 0.0001;





/***** LIBRARY *****/
/// The Axis-Aligned Bounding Box (AABB) can be used to cheaply pre-check if we roughly hit an object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    /// The top-left point of the box.
    pub pos:  Vec3,
    /// The dimensions of the box in all directions.
    pub dims: [f64; 3],
}

// Constructors
impl AABB {
    /// Constructor for the AABB that initializes it as an "empty" box around (0, 0, 0).
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn zeroes() -> Self { Self::new(Vec3::zeroes(), [0.0, 0.0, 0.0]) }

    /// Constructor for the AABB.
    ///
    /// Note that this will always pad the AABB to have some none-zero dimensions; specifically,
    /// none of them will every be smaller than [`AABB_MIN_DIM_LEN`].
    ///
    /// # Arguments
    /// - `pos`: The position of the box.
    /// - `dims`: The dimensions of the box.
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn new(pos: Vec3, dims: [f64; 3]) -> Self {
        Self { pos, dims: [f64::max(dims[0], AABB_MIN_DIM_LEN), f64::max(dims[1], AABB_MIN_DIM_LEN), f64::max(dims[2], AABB_MIN_DIM_LEN)] }
    }

    /// Constructor for the AABB that computes it from two points.
    ///
    /// # Arguments
    /// - `pos1`: One of the two AABB points.
    /// - `pos2`: The other of the two AABB points.
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn from_points(pos1: Vec3, pos2: Vec3) -> Self {
        // Order the vectors into a minimum and maximum one
        let min: Vec3 = Vec3 { x: f64::min(pos1.x, pos2.x), y: f64::min(pos1.y, pos2.y), z: f64::min(pos1.z, pos2.z) };
        let max: Vec3 = Vec3 { x: f64::max(pos1.x, pos2.x), y: f64::max(pos1.y, pos2.y), z: f64::max(pos1.z, pos2.z) };

        // Min is the pos, then the dimensions are computable
        Self::new(min, [max.x - min.x, max.y - min.y, max.z - min.z])
    }
}

// AABB
impl AABB {
    /// Computes a bounding box surrounding ourselves and a given one.
    ///
    /// # Arguments
    /// - `other`: The other box to surround.
    ///
    /// # Returns
    /// A new [`AABB`] that perfectly fits `self` and `other`.
    #[inline]
    pub const fn surround(self, other: Self) -> Self {
        #[inline]
        const fn update_axis(x1: f64, x2: f64, dim1: f64, dim2: f64) -> (f64, f64) {
            // Compute the endpoint and then the surround endpoints for  this axis
            let b1: f64 = x1 + dim1;
            let b2: f64 = x2 + dim2;
            let (x, b) = (f64::min(x1, x2), f64::max(b1, b2));

            // Scale that back to a dimension
            (x, b - x)
        }

        let (x, dimx) = update_axis(self.pos.x, other.pos.x, self.dims[0], other.dims[0]);
        let (y, dimy) = update_axis(self.pos.y, other.pos.y, self.dims[1], other.dims[1]);
        let (z, dimz) = update_axis(self.pos.z, other.pos.z, self.dims[2], other.dims[2]);
        Self::new(Vec3::new(x, y, z), [dimx, dimy, dimz])
    }



    /// Computes a hit with a given ray quickly to use the AABB as a cheap hit to see if a ray hits
    /// an object's approximate area before computing the expensive hit.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute a hit with.
    /// - `t_min`: A minimal `t` (i.e., distance along the Ray from its origin) that we accept.
    /// - `t_max`: A maximal `t` (i.e., distance along the Ray from its origin) that we accept.
    ///
    /// # Returns
    /// Whether the given ray hits this AABB.
    #[inline]
    pub fn hittest(&self, ray: Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for i in 0..3 {
            // Compute the hit points with the AABB
            let inv_direction: f64 = 1.0 / ray.direct[i];
            let mut t0: f64 = (self.pos[i] - ray.origin[i]) * inv_direction;
            let mut t1: f64 = ((self.pos[i] + self.dims[i]) - ray.origin[i]) * inv_direction;

            // Ensure we order the values properly, and then bind them by the given min/max
            if inv_direction < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }
            t_min = t0.clamp(t_min, f64::INFINITY);
            t_max = t1.clamp(-f64::INFINITY, t_max);

            // We don't hit if t_max is now too small
            if t_max <= t_min {
                return false;
            }
        }
        true
    }



    /// Gets the dimensions of the box.
    ///
    /// # Returns
    /// A triplet of values of the box' dimensions along [X, Y, Z].
    #[inline]
    pub const fn dims(&self) -> [f64; 3] { self.dims }
}

// Hitting
impl BoundingBoxable for AABB {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { *self }
}
impl Hittable for AABB {
    #[inline]
    fn hit(&self, ray: Ray, mut t_min: f64, mut t_max: f64, _env: &Environment) -> Option<HitRecord> {
        let mut hit_axis: usize = 0;
        let mut hit_scale: f64 = 1.0;
        for i in 0..3 {
            // Compute the hit points with the AABB
            let inv_direction: f64 = 1.0 / ray.direct[i];
            let mut t0: f64 = (self.pos[i] - ray.origin[i]) * inv_direction;
            let mut t1: f64 = ((self.pos[i] + self.dims[i]) - ray.origin[i]) * inv_direction;

            // Ensure we order the values properly, and then bind them by the given min/max
            if inv_direction < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }
            if t0 > t_min {
                t_min = t0;
                hit_axis = i;
                hit_scale = if inv_direction < 0.0 { -1.0 } else { 1.0 };
            }
            t_max = t1.clamp(-f64::INFINITY, t_max);

            // We don't hit if t_max is now too small
            if t_max <= t_min {
                return None;
            }
        }
        Some(HitRecord::new(
            ray,
            ray.at(t_min),
            t_min,
            Vec3::new(
                if hit_axis == 0 { hit_scale } else { 0.0 },
                if hit_axis == 1 { hit_scale } else { 0.0 },
                if hit_axis == 2 { hit_scale } else { 0.0 },
            ),
            (0.0, 0.0),
        ))
    }
}

// Iterators
impl FromIterator<Self> for AABB {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        let mut res: Option<Self> = None;
        for b in iter {
            if let Some(res) = &mut res {
                *res = Self::surround(*res, b);
            } else {
                res = Some(b);
            }
        }
        res.unwrap_or(AABB::zeroes())
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_from_points() {
        assert_eq!(AABB::from_points([0.0, 0.0, 0.0].into(), [1.0, 1.0, 1.0].into()), AABB { pos: [0.0, 0.0, 0.0].into(), dims: [1.0, 1.0, 1.0] });
        assert_eq!(AABB::from_points([1.0, 1.0, 1.0].into(), [0.0, 0.0, 0.0].into()), AABB { pos: [0.0, 0.0, 0.0].into(), dims: [1.0, 1.0, 1.0] });
        assert_eq!(AABB::from_points([42.0, 18.0, 0.3].into(), [0.55, -60.0, 3.0].into()), AABB {
            pos:  [0.55, -60.0, 0.3].into(),
            dims: [41.45, 78.0, 2.7],
        });
    }
}
