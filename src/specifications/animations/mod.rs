//  ANIMATIONS.rs
//    by Lut99
//
//  Description:
//!   Defines abstract versions of animations through a scene.
//

// Modules
pub mod vertical;

// Imports
pub use vertical::Vertical;

use crate::math::Vec3;


/***** LIBRARY *****/
/// Defines abstract representation of an animation that determines an object's location over time.
pub trait Animation {
    /// Computes the position of an object given it's start position and the current time.
    ///
    /// Note that this computation is absolute: you always compute from the start! This because
    /// rays are cast at random moments, interleavingly, so the position cannot monotonically
    /// increase.
    ///
    /// # Arguments
    /// - `pos`: The starting position of the the sphere.
    /// - `t`: The current time, in microseconds, since the start of the scene.
    ///
    /// # Returns
    /// A new position of the sphere.
    fn animate(&self, pos: Vec3, t: u64) -> Vec3;
}
