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

use std::fmt::{Formatter, Result as FResult};

use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

use super::ray::Ray;
use super::vec3::Vec3;
use crate::specifications::objects::{BoundingBoxable, HitRecord, Hittable};
use crate::specifications::scene::Environment;


/***** CONSTANTS *****/
/// Determines the minimum size for every of [`AABB`]'s dimensions.
pub const AABB_MIN_DIM_LEN: f64 = 0.0001;





/***** HELPERS *****/
/// Defines an interval that is ALWAYS ordered from small to large.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval(f64, f64);

// Constructors
impl Interval {
    /// Constructor for an Interval that creates it as an empty interval starting at `0`.
    ///
    /// # Returns
    /// A new Interval with ordered values but empty.
    #[inline]
    pub const fn empty() -> Self { Self(0.0, 0.0) }

    /// Constructor for an Interval that creates it as an interval of length 1 starting at `0`.
    ///
    /// # Returns
    /// A new Interval with ordered values.
    #[inline]
    pub const fn unit() -> Self { Self(0.0, 1.0) }

    /// Constructor for an Interval that creates an infinite interval in both directions.
    ///
    /// # Returns
    /// A new Interval that encompasses Everything.
    #[inline]
    pub const fn infinite() -> Self { Self(-f64::INFINITY, f64::INFINITY) }

    /// Constructor for an Interval that creates it from any two values.
    ///
    /// The values are automatically internally ordered.
    ///
    /// If you're absolutely sure they're already ordered, see [`Interval::new_ordered()`].
    ///
    /// # Arguments
    /// - `v1`: The first value.
    /// - `v2`: The second value.
    ///
    /// # Returns
    /// A new Interval with ordered values.
    #[inline]
    pub const fn new(v1: f64, v2: f64) -> Self { if v1 <= v2 { Self(v1, v2) } else { Self(v2, v1) } }

    /// Constructor for an Interval that creates it from two ordered values.
    ///
    /// # Arguments
    /// - `v1`: The smaller/equal value.
    /// - `v2`: The equal/larger value.
    ///
    /// # Safety
    /// You must make sure that `v1` is smaller than or equal to `v2` to uphold this struct's
    /// assumption.
    ///
    /// # Returns
    /// A new Interval with ordered values.
    #[inline]
    pub const unsafe fn new_ordered(v1: f64, v2: f64) -> Self { Self(v1, v2) }
}

// Mutators
impl Interval {
    /// Translates the whole interval by a fixed amount.
    ///
    /// # Arguments
    /// - `offset`: The value to translate with.
    ///
    /// # Returns
    /// A new Interval that has `min` and `max` increased by `offset`.
    #[inline]
    pub const fn translate(self, value: f64) -> Self { Self(self.0 + value, self.1 + value) }

    /// Computes a new interval surrounding ourselves and a given one.
    ///
    /// # Arguments
    /// - `other`: The other interval to surround.
    ///
    /// # Returns
    /// A new Interval that perfectly fits `self` and `other`.
    #[inline]
    pub const fn surround(self, other: Self) -> Self { Self(f64::min(self.0, other.0), f64::max(self.1, other.1)) }

    /// Pads this interval to be at least [`AABB_MIN_DIM_LEN`] in length.
    ///
    /// This is done by extending the end of the interval a little bit.
    ///
    /// # Returns
    /// Self but with at least [`AABB_MIN_DIM_LEN`] length.
    #[inline]
    pub const fn pad(self) -> Self { if self.len() < AABB_MIN_DIM_LEN { Self(self.0, self.0 + AABB_MIN_DIM_LEN) } else { self } }
}

// Accessors
impl Interval {
    /// Returns the minimum value in the interval.
    #[inline]
    pub const fn min(self) -> f64 { self.0 }

    /// Returns the maximum value in the interval.
    #[inline]
    pub const fn max(self) -> f64 { self.1 }

    /// Returns the length of the interval, i.e., [`Interval::max()`] - [`Interval::min()`].
    #[inline]
    pub const fn len(self) -> f64 { self.1 - self.0 }
}

// Serde
impl<'de> Deserialize<'de> for Interval {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IntervalVisitor;
        impl<'de> Visitor<'de> for IntervalVisitor {
            type Value = Interval;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> FResult { write!(f, "an interval of a minimum value and a maximum value") }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Get the next two elements - and only two
                let v1: f64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let v2: f64 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                if seq.next_element::<f64>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }

                // Use that to build s elf
                Ok(Interval::new(v1, v2).pad())
            }
        }

        deserializer.deserialize_seq(IntervalVisitor)
    }
}
impl Serialize for Interval {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}





/***** LIBRARY *****/
/// The Axis-Aligned Bounding Box (AABB) can be used to cheaply pre-check if we roughly hit an object.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct AABB {
    /// The interval of the AABB over the X-axis.
    pub x: Interval,
    /// The interval of the AABB over the Y-axis.
    pub y: Interval,
    /// The interval of the AABB over the Z-axis.
    pub z: Interval,
}

// Constructors
impl AABB {
    /// Constructor for the AABB that initializes it as an "empty" box around (0, 0, 0).
    ///
    /// Note that this will still pad the AABB to have some none-zero dimensions; specifically,
    /// none of them will every be smaller than [`AABB_MIN_DIM_LEN`].
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn zeroes() -> Self { Self::new(Interval::empty(), Interval::empty(), Interval::empty()) }

    /// Constructor for the AABB.
    ///
    /// Note that this will always pad the AABB to have some none-zero dimensions; specifically,
    /// none of them will every be smaller than [`AABB_MIN_DIM_LEN`].
    ///
    /// # Arguments
    /// - `x`: The [`Interval`] over the X-axis.
    /// - `y`: The [`Interval`] over the Y-axis.
    /// - `z`: The [`Interval`] over the Z-axis.
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn new(x: Interval, y: Interval, z: Interval) -> Self { Self { x: x.pad(), y: y.pad(), z: z.pad() } }

    /// Constructor for the AABB that computes it from two points.
    ///
    /// Note that this will always pad the AABB to have some none-zero dimensions; specifically,
    /// none of them will every be smaller than [`AABB_MIN_DIM_LEN`].
    ///
    /// # Arguments
    /// - `pos1`: One of the two AABB points.
    /// - `pos2`: The other of the two AABB points.
    ///
    /// # Returns
    /// A new instance of an AABB.
    #[inline]
    pub const fn from_points(pos1: Vec3, pos2: Vec3) -> Self {
        Self::new(Interval::new(pos1.x, pos2.x), Interval::new(pos1.y, pos2.y), Interval::new(pos1.z, pos2.z))
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
    pub const fn surround(self, other: Self) -> Self { Self::new(self.x.surround(other.x), self.y.surround(other.y), self.z.surround(other.z)) }



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
    pub fn hittest(&self, ray: Ray, t_min: f64, t_max: f64) -> bool {
        // let int = [self.x, self.y, self.z];
        // for i in 0..3 {
        //     // Compute the hit points with the AABB
        //     let inv_direction: f64 = 1.0 / ray.direct[i];
        //     let mut t0: f64 = (int[i].min() - ray.origin[i]) * inv_direction;
        //     let mut t1: f64 = (int[i].max() - ray.origin[i]) * inv_direction;

        //     // Ensure we order the values properly, and then bind them by the given min/max
        //     if t0 > t1 {
        //         mem::swap(&mut t0, &mut t1);
        //     }
        //     t_min = f64::max(t_min, t0);
        //     t_max = f64::min(t_max, t1);

        //     // We don't hit if t_max is now too small
        //     if t_max <= t_min {
        //         return false;
        //     }
        // }
        // true

        // Compute the hitpoints with the box' intervals
        // The `Interval` takes care to order  them from small to large anyway
        let (invdirx, invdiry, invdirz): (f64, f64, f64) = (1.0 / ray.direct.x, 1.0 / ray.direct.y, 1.0 / ray.direct.z);
        let tx: Interval = Interval::new((self.x.min() - ray.origin.x) * invdirx, (self.x.max() - ray.origin.x) * invdirx);
        let ty: Interval = Interval::new((self.y.min() - ray.origin.y) * invdiry, (self.y.max() - ray.origin.y) * invdiry);
        let tz: Interval = Interval::new((self.z.min() - ray.origin.z) * invdirz, (self.z.max() - ray.origin.z) * invdirz);

        // If it overlaps, it's a hit; otherwise it isn't.
        let hitmin = f64::max(t_min, f64::max(f64::max(tx.min(), ty.min()), tz.min()));
        let hitmax = f64::min(t_max, f64::min(f64::min(tx.max(), ty.max()), tz.max()));
        hitmin < hitmax
    }



    /// Gets the interval along a dimensions.
    ///
    /// # Arguments
    /// - `dim`: A number 0-2 that indicates which of the three dimensions.
    ///
    /// # Returns
    /// An [`Interval`] reference of the given `dim`ension.
    #[inline]
    #[track_caller]
    pub const fn dim(&self, dim: usize) -> &Interval { [&self.x, &self.y, &self.z][dim] }

    /// Gets the dimensions of the box.
    ///
    /// # Returns
    /// A triplet of values of the box' dimensions along [X, Y, Z].
    #[inline]
    pub const fn dims(&self) -> [f64; 3] { [self.x.len(), self.y.len(), self.z.len()] }
}

// Hitting
impl BoundingBoxable for AABB {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB { *self }
}
impl Hittable for AABB {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitRecord<'_>> {
        // Compute the hitpoints with the box' intervals
        // The `Interval` takes care to order  them from small to large anyway
        let (invdirx, invdiry, invdirz): (f64, f64, f64) = (1.0 / ray.direct.x, 1.0 / ray.direct.y, 1.0 / ray.direct.z);
        let tx: Interval = Interval::new((self.x.min() - ray.origin.x) * invdirx, (self.x.max() - ray.origin.x) * invdirx);
        let ty: Interval = Interval::new((self.y.min() - ray.origin.y) * invdiry, (self.y.max() - ray.origin.y) * invdiry);
        let tz: Interval = Interval::new((self.z.min() - ray.origin.z) * invdirz, (self.z.max() - ray.origin.z) * invdirz);

        // Compute the overlapping range of the interval. Importantly, remember which sides map to which normal vectors.
        // Ugly, could also have been a bunch of `f64::max()`'s and such
        let (hit0, norm0): (f64, Vec3) = if tx.min() >= ty.min() && tx.min() >= tz.min() {
            (tx.min(), Vec3::new(invdirx / invdirx.abs(), 0.0, 0.0))
        } else if ty.min() >= tx.min() && ty.min() >= tz.min() {
            (ty.min(), Vec3::new(0.0, invdiry / invdiry.abs(), 0.0))
        } else {
            (tz.min(), Vec3::new(0.0, 0.0, invdirz / invdirz.abs()))
        };
        let (hit1, norm1): (f64, Vec3) = if tx.max() <= ty.max() && tx.max() <= tz.max() {
            (tx.max(), Vec3::new(invdirx / invdirx.abs(), 0.0, 0.0))
        } else if ty.max() <= tx.max() && ty.max() <= tz.max() {
            (ty.max(), Vec3::new(0.0, invdiry / invdiry.abs(), 0.0))
        } else {
            (tz.max(), Vec3::new(0.0, 0.0, invdirz / invdirz.abs()))
        };

        // If they all overlap, then we hit (so we miss if the ranges don't overlap)
        // NOTE: Technically a redundant check, actually, because we always happen to be precisely
        // our own AABB and that already gets hitchecked.
        if hit1 <= hit0 {
            return None;
        }

        // Then comes the question: where did we really hit?
        let (hit, norm): (f64, Vec3) = if hit0 >= t_min && hit0 <= t_max {
            (hit0, norm0)
        } else if hit1 >= t_min && hit1 <= t_max {
            (hit1, norm1)
        } else {
            // Last-minute cancallation due to `t_min` and `t_max`
            return None;
        };

        // Create the hitrecord and return
        Some(HitRecord::new(
            ray,
            ray.at(hit),
            hit,
            norm,
            (0.0, 0.0), // TODO
            &(),
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
        assert_eq!(
            AABB::from_points([0.0, 0.0, 0.0].into(), [1.0, 1.0, 1.0].into()),
            AABB::new(Interval::unit(), Interval::unit(), Interval::unit())
        );
        assert_eq!(
            AABB::from_points([1.0, 1.0, 1.0].into(), [0.0, 0.0, 0.0].into()),
            AABB::new(Interval::unit(), Interval::unit(), Interval::unit())
        );
        assert_eq!(
            AABB::from_points([42.0, 18.0, 0.3].into(), [0.55, -60.0, 3.0].into()),
            AABB::new(Interval::new(0.55, 42.0), Interval::new(-60.0, 18.0), Interval::new(0.3, 3.0))
        );
    }
}
