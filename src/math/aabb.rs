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
    pub const fn zeroes() -> Self { Self { pos: Vec3::zeroes(), dims: [0.0; 3] } }

    /// Constructor for the AABB.
    ///
    /// # Arguments
    /// - `pos`: The position of the box.
    /// - `dims`: The dimensions of the box.
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn new(pos: Vec3, dims: [f64; 3]) -> Self { Self { pos, dims } }

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
        Self { pos: Vec3::new(x, y, z), dims: [dimx, dimy, dimz] }
    }
}

// AABB
impl AABB {
    /// Gets the dimensions of the box.
    ///
    /// # Returns
    /// A triplet of values of the box' dimensions along [X, Y, Z].
    #[inline]
    pub const fn dims(&self) -> [f64; 3] { self.dims }
}

// Hitting
impl AABB {
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
