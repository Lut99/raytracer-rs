//  RANDOM.rs
//    by Lut99
//
//  Description:
//!   Defines random utilities for the raytracer.
//

use crate::math::Vec3;


/***** LIBRARY *****/
/// Returns a random floating-point number in the range `[0, 1)`.
///
/// # Returns
/// A random [`f64`] in the range `0..1`.
#[inline]
pub fn f64() -> f64 { fastrand::f64() }

/// Returns a random floating-point number in the range `[0, 1]`.
///
/// # Returns
/// A random [`f64`] in the range `0..=1`.
#[inline]
pub fn f64inc() -> f64 { fastrand::f64_inclusive() }

/// Returns a random floating-point number in the given range.
///
/// # Arguments
/// - `min`: The minimum value (inclusive).
/// - `max`: The maximum value (exclusive).
///
/// # Returns
/// A random [`f64`] in the range `[min, max)`.
#[inline]
#[cfg_attr(debug_assertions, track_caller)]
pub fn f64int(min: f64, max: f64) -> f64 {
    #[cfg(debug_assertions)]
    if min > max {
        panic!("Cannot draw a random number in empty range [{min}, {max})")
    }
    ((max - min) * f64()) + min
}



/// Returns a random index in the given range.
///
/// # Arguments
/// - `min`: The minimum value (inclusive).
/// - `max`: The maximum value (exclusive).
///
/// # Returns
/// A random [`f64`] in the range `[min, max)`.
#[inline]
#[cfg_attr(debug_assertions, track_caller)]
pub fn usizeint(min: usize, max: usize) -> usize {
    #[cfg(debug_assertions)]
    if min > max {
        panic!("Cannot draw a random number in empty range [{min}, {max})")
    }
    fastrand::usize(min..max)
}



impl Vec3 {
    /// Returns a random 3D vector with each element the range `[0, 1)`.
    ///
    /// # Returns
    /// A random [`Vec3`] with each element in the range `0..1`.
    #[inline]
    pub fn rand() -> Vec3 { Vec3::new(f64(), f64(), f64()) }

    /// Returns a random 3D vector with each element the given range.
    ///
    /// # Arguments
    /// - `min`: The minimum value (inclusive).
    /// - `max`: The maximum value (exclusive).
    ///
    /// # Returns
    /// A random [`Vec3`] with each element in the range `[min, max)`.
    #[inline]
    pub fn randint(min: f64, max: f64) -> Vec3 { Vec3::new(f64int(min, max), f64int(min, max), f64int(min, max)) }
}
