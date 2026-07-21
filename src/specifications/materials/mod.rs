//  MOD.rs
//    by Lut99
//
//  Created:
//    05 May 2023, 10:41:36
//  Last edited:
//    07 May 2023, 10:51:52
//  Auto updated?
//    Yes
//
//  Description:
//!   The `materials` module defines the various materials we can render
//!   to. While it is structured object-oriented-like, we never call the
//!   material as a dynamic trait object. This way, we can get OOP design
//!   pros with functional speeds.
//

// Declare submodules
pub mod dielectric;
pub mod diffuse;
pub mod metal;
pub mod simple;

// Imports & Exports
pub use dielectric::{Dielectric, PartialDielectric};
pub use diffuse::{Diffuse, Lambertian, LambertianTexture};
pub use metal::Metal;
use serde::{Deserialize, Serialize};
pub use simple::{NormalMap, StaticColour};
use thiserror::Error;

use super::Loadable;
use crate::math::{Colour, Ray};
use crate::specifications::objects::HitRecord;
use crate::specifications::scene::Environment;


/***** INTERFACES *****/
/// The Scattering trait implements any material that we can use to cover an object.
pub trait Scattering {
    /// Bounces (or reflects) a ray from this material.
    ///
    /// # Arguments
    /// - `ray`: The inbound [`Ray`] that we want to scatter.
    /// - `record`: The [`HitRecord`] that determines where the hit was and what the hit normal was
    ///   and such.
    /// - `env`: An [`Environment`] object relating properties about the scene's global
    ///   environment.
    ///
    /// # Returns
    /// A tuple that represents the bounced [`Ray`] and the attenuated colour from this bounce. If
    /// [`None`] is returned for the [`Ray`], then no more bounce is necessary.
    fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour);
}





/***** LIBRARY *****/
macro_rules! material_impl {
    // Default error type insertion
    (__ { $(#[$($fattrs:tt)*])* $fmat:ident $(, $(#[$($rattrs:tt)*])* $rmat:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__ {$($(#[$($rattrs)*])* $rmat $(($rerrty))?),*} { $(#[$($fattrs)*])* $fmat (::std::convert::Infallible) $(, $(#[$($attrs)*])* $mat ($errty))* });
    };
    (__ { $(#[$($fattrs:tt)*])* $fmat:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])* $rmat:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__ {$($(#[$($rattrs)*])* $rmat $(($rerrty))?),*} { $(#[$($fattrs)*])* $fmat ($ferrty) $(, $(#[$($attrs)*])* $mat ($errty))* });
    };


    // Actual impl
    (__ {} { $($(#[$($attrs:tt)*])* $mat:ident ( $errty:ty )),* }) => {
        /// Errors occurring when loading the material.
        #[derive(Debug, Error)]
        pub enum Error {
            $(#[error("{0}")] $mat(#[source] $errty),)*
        }



        /// A runtime abstraction of all possible materials.
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum Material {
            $($(#[$($attrs)*])* $mat($mat),)*
        }

        // Interface
        impl Loadable for Material {
            type Error = Error;

            #[inline]
            fn load(&mut self) -> Result<(), Self::Error> {
                match self {
                    $(Self::$mat(m) => m.load().map_err(Error::$mat),)*
                }
            }
        }
        impl Scattering for Material {
            #[inline]
            fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) {
                match self {
                    $(Self::$mat(m) => m.scatter(ray, record, env),)*
                }
            }
        }
    };

    // Public interface
    ($($(#[$($attrs:tt)*])* $tex:ident $(( $errty:ty ))?),* $(,)?) => {
        material_impl!(__ { $($(#[$($attrs)*])? $tex $(($errty))?),* } {});
    };
}
material_impl!(
    /// A refracting material (e.g., glass, water-on-air, etc).
    Dielectric,
    /// A material randomly scattering rays, imperfectly.
    Diffuse,
    /// A material randomly scattering rays.
    Lambertian,
    /// A material randomly scattering rays but with a texture.
    LambertianTexture(super::textures::Error),
    /// A material reflecting rays perfectly.
    Metal,
    /// A material having colours of the object's normals.
    NormalMap,
    /// A partially refracting material (has some holes in the math that stops is refracting).
    PartialDielectric,
    /// A material having a static colour.
    StaticColour,
);
