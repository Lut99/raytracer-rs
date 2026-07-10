//  TEXTURES.rs
//    by Lut99
//
//  Description:
//!   Implements supported textures.
//

pub mod checker;

pub use checker::Checker;
use serde::{Deserialize, Serialize};

use crate::math::{Colour, Vec3};


/***** INTERFACES *****/
/// Denotes something that is a texture - i.e., maps ray hit coordinates to a colour.
pub trait Textured {
    /// Returns the colour of this texture at the given hit coordinate.
    ///
    /// # Arguments
    /// - `u`: The hit coordinate's X-component mapped to the space of the object.
    /// - `v`: The hit coordinate's Y-component mapped to the space of the object.
    /// - `p`: The point (in space) where the ray hit the object. I.e., some absolute position.
    ///
    /// # Returns
    /// A [`Colour`] describing the colour of the texture at that location.
    fn value(&self, uv: (f64, f64), p: Vec3) -> Colour;
}





/***** LIBRARY *****/
macro_rules! object_impl {
    ($($(#[$($attrs:tt)*])* $tex:ident),* $(,)?) => {
        /// A runtime abstraction of all possible textures.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        pub enum Texture {
            $($(#[$($attrs)*])* $tex($tex),)*
        }

        // Interface
        impl Textured for Texture {
            #[inline]
            fn value(&self,uv: (f64, f64), p: Vec3) -> Colour {
                match self {
                    $(Self::$tex(t) => t.value(uv, p),)*
                }
            }
        }
    };
}
object_impl!(
    /// A texture rendering as a checkerboard.
    Checker,
);
