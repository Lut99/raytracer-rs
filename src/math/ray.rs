//  RAY.rs
//    by Lut99
//
//  Created:
//    27 Apr 2023, 14:46:36
//  Last edited:
//    03 May 2023, 08:40:32
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Ray`] class, which represents a single ray of light.
//

use std::fmt::{Display, Formatter, Result as FResult};

use super::vec3::Vec3;


/***** LIBRARY *****/
/// Represents a single ray of light that we want to bounce around. Based on the [`Vec3`] class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    /// The origin of the Ray.
    pub origin: Vec3,
    /// The directory of the Ray.
    pub direct: Vec3,
    /// The time at which the Ray is shot, as microseconds from the start of the scene.
    pub time:   u64,
}

impl Default for Ray {
    #[inline]
    fn default() -> Self { Self::zeroes() }
}
impl Ray {
    /// Constructor for the Ray.
    ///
    /// Note that this constructor will make it without the time component (sets it to `0`).
    /// For a constructor with it, see [`Ray::with_time()`] instead.
    ///
    /// # Arguments
    /// - `origin`: The `origin` vector.
    /// - `direction`: The `direction` vector of this ray.
    ///
    /// # Returns
    /// A new `Ray` instance with the given origin and direction.
    #[inline]
    pub fn new(origin: impl Into<Vec3>, direction: impl Into<Vec3>) -> Self { Self { origin: origin.into(), direct: direction.into(), time: 0 } }

    /// Constructor for the Ray.
    ///
    /// This constructor also takes a time at which the ray is cast.
    ///
    /// # Arguments
    /// - `origin`: The origin vector.
    /// - `direction`: The direction vector of this ray.
    ///
    /// # Returns
    /// A new `Ray` instance with the given `origin` and `direction`, fired at the given `time`.
    #[inline]
    pub fn with_time(origin: impl Into<Vec3>, direction: impl Into<Vec3>, time: u64) -> Self {
        Self { origin: origin.into(), direct: direction.into(), time }
    }

    /// Constructor for the Ray that initializes it to all zeroes.
    ///
    /// # Returns
    /// A new `Ray` instance that just has zeroes everywhere.
    #[inline]
    pub fn zeroes() -> Self { Self { origin: Vec3::zeroes(), direct: Vec3::zeroes(), time: 0 } }



    /// Returns a point somewhere along this ray.
    ///
    /// # Arguments
    /// - `t`: The distance from the ray's origin to travel.
    ///
    /// # Returns
    /// A new [`Vec3`] that represents the point along the Ray.
    #[inline]
    pub fn at(&self, t: f64) -> Vec3 { self.origin + t * self.direct }
}

impl Display for Ray {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}->{}", self.origin, self.direct) }
}
