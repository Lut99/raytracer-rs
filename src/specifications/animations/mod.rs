//  ANIMATIONS.rs
//    by Lut99
//
//  Description:
//!   Defines abstract versions of animations through a scene.
//

// Modules
pub mod vertical;

// Imports & Exports
use serde::{Deserialize, Serialize};
pub use vertical::Vertical;

use crate::math::Vec3;


/***** INTERFACES *****/
/// Defines abstract representation of an animation that determines an object's location over time.
pub trait Animating {
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





/***** LIBRARY *****/
macro_rules! animation_impl {
    ($($(#[$($attrs:tt)*])* $ani:ident),* $(,)?) => {
        /// A runtime abstraction of all possible animations.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        pub enum Animation {
            $($(#[$($attrs)*])* $ani($ani),)*
        }

        // Interface
        impl Animating for Animation {
            #[inline]
            fn animate(&self, pos: Vec3, t: u64) -> Vec3 {
                match self {
                    $(Self::$ani(a) => a.animate(pos, t),)*
                }
            }
        }
    };
}
animation_impl!(
    /// An animation sending some object up.
    Vertical,
);
