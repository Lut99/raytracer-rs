//  TEXTURES.rs
//    by Lut99
//
//  Description:
//!   Implements supported textures.
//

// Modules
pub mod checker;
pub mod image;

// Imports
pub use checker::{Checker, SpatialChecker};
pub use image::Image;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Loadable;
use crate::math::{Colour, Vec3};


/***** INTERFACES *****/
/// Denotes something that is a texture - i.e., maps ray hit coordinates to a colour.
pub trait Textured {
    /// Returns the colour of this texture at the given hit coordinate.
    ///
    /// # Arguments
    /// - `uv`: The hit coordinates mapped to the space of the object.
    /// - `p`: The point (in space) where the ray hit the object. I.e., some absolute position.
    ///
    /// # Returns
    /// A [`Colour`] describing the colour of the texture at that location.
    fn value(&self, uv: (f64, f64), p: Vec3) -> Colour;
}





/***** LIBRARY *****/
macro_rules! texture_impl {
    // Default error type insertion
    (__ { $(#[$($fattrs:tt)*])* $ftex:ident $(, $(#[$($rattrs:tt)*])* $rtex:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $tex:ident ( $errty:ty )),* }) => {
        texture_impl!(__ {$($(#[$($rattrs)*])* $rtex $(($rerrty))?),*} { $(#[$($fattrs)*])* $ftex (::std::convert::Infallible) $(, $(#[$($attrs)*])* $tex ($errty))* });
    };
    (__ { $(#[$($fattrs:tt)*])* $ftex:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])* $rtex:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $tex:ident ( $errty:ty )),* }) => {
        texture_impl!(__ {$($(#[$($rattrs)*])* $rtex $(($rerrty))?),*} { $(#[$($fattrs)*])* $ftex ($ferrty) $(, $(#[$($attrs)*])* $tex ($errty))* });
    };


    // Actual impl
    (__ {} { $($(#[$($attrs:tt)*])* $tex:ident ( $errty:ty )),* }) => {
        /// Errors occurring when loading the texture.
        #[derive(Debug, Error)]
        pub enum Error {
            $(#[error("{0}")] $tex(#[source] $errty),)*
        }



        /// A runtime abstraction of all possible textures.
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum Texture {
            $($(#[$($attrs)*])* $tex($tex),)*
        }

        // Interface
        impl Loadable for Texture {
            type Error = Error;

            #[inline]
            fn load(&mut self) -> Result<(), Self::Error> {
                match self {
                    $(Self::$tex(t) => t.load().map_err(Error::$tex),)*
                }
            }
        }
        impl Textured for Texture {
            #[inline]
            fn value(&self,uv: (f64, f64), p: Vec3) -> Colour {
                match self {
                    $(Self::$tex(t) => t.value(uv, p),)*
                }
            }
        }
    };

    // Public interface
    ($($(#[$($attrs:tt)*])* $tex:ident $(( $errty:ty ))?),* $(,)?) => {
        texture_impl!(__ { $($(#[$($attrs)*])? $tex $(($errty))?),* } {});
    };
}
texture_impl!(
    /// A texture rendering as a checkerboard.
    Checker,
    /// A texture loaded from an image.
    Image(crate::render::image::Error),
    /// A texture rendering as a checkerboard but using spatial coordinates.
    SpatialChecker,
);
