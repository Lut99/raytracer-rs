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
pub mod phase_function;
pub mod simple;

// Imports & Exports
use std::cell::{Ref, RefMut};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use dielectric::{Dielectric, PartialDielectric};
pub use diffuse::{Diffuse, DiffuseLight, Lambertian, LambertianTexture};
pub use metal::Metal;
pub use phase_function::Isotropic;
use serde::{Deserialize, Serialize};
pub use simple::{NormalMap, StaticColour};
use thiserror::Error;

use super::Loadable;
use crate::math::{Colour, Ray, Vec3};
use crate::specifications::objects::HitData;
use crate::specifications::scene::Environment;


/***** HELPER MACROS *****/
/// Pointer-like impls for [`Scattering`].
macro_rules! scattering_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Scattering> Scattering for $ty {
            #[inline]
            fn pdf(&self, ray: Ray, record: &HitData, env: &Environment, scattered: Ray) -> f64 {
                <T as Scattering>::pdf(self, ray, record, env, scattered)
            }

            #[inline]
            fn emitted(&self, uv: (f64, f64), p: Vec3) -> Colour { <T as Scattering>::emitted(self, uv, p) }

            #[inline]
            fn scatter(&self, ray: Ray, record: &HitData, env: &Environment) -> (Option<Ray>, Colour, f64) {
                <T as Scattering>::scatter(self, ray, record, env)
            }
        }
    };
    ($ty:ty) => {
        impl<T: Scattering> Scattering for $ty {
            #[inline]
            fn pdf(&self, ray: Ray, record: &HitData, env: &Environment, scattered: Ray) -> f64 {
                <T as Scattering>::pdf(self, ray, record, env, scattered)
            }

            #[inline]
            fn emitted(&self, uv: (f64, f64), p: Vec3) -> Colour { <T as Scattering>::emitted(self, uv, p) }

            #[inline]
            fn scatter(&self, ray: Ray, record: &HitData, env: &Environment) -> (Option<Ray>, Colour, f64) {
                <T as Scattering>::scatter(self, ray, record, env)
            }
        }
    };
}





/***** AUXILLARY *****/
/// Denotes the "kind" of [`Material`]s.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObjectKind {
    /// The material is a light source, so it may be random but we will still want to sample from
    /// it.
    Light,
    /// The material has a random component to the scatter.
    Scatter,
    /// The scatter is deterministic.
    Specular,
}





/***** INTERFACES *****/
/// The Scattering trait implements any material that we can use to cover an object.
pub trait Scattering {
    /// Samples the probability of this material scattering a ray in the given direction.
    ///
    /// # Returns
    /// A weight that correctly weights the result from this ray based on how likely it is that
    /// this ray hits a light.
    #[inline]
    fn pdf(&self, _ray: Ray, _record: &HitData, _env: &Environment, _scattered: Ray) -> f64 {
        /* Standard impl: no weights */
        1.0
    }

    /// Returns the colour of any light emitted by this material.
    ///
    /// # Arguments
    /// - `uv`: A pair of texture coordinates on the object to return the color of the object's
    ///   light at that spot.
    /// - `p`: A spatial position of the object's hitted surface.
    ///
    /// # Returns
    /// A [`Colour`] of the light being emitted. Is black if this emits nothing.
    #[inline]
    fn emitted(&self, _uv: (f64, f64), _p: Vec3) -> Colour {
        /* Standard impl: just black */
        Colour::BLACK
    }

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
    /// A tuple that represents the bounced [`Ray`], the attenuated colour from this bounce and the
    /// PDF weight based on the scattered ray.
    ///
    /// If [`None`] is returned for the [`Ray`], then no more bounce is necessary.
    fn scatter(&self, _ray: Ray, _record: &HitData, _env: &Environment) -> (Option<Ray>, Colour, f64) {
        /* Standard impl: no scattering */
        (None, Colour::BLACK, 1.0)
    }
}

// Standard impls
impl Scattering for () {
    #[inline]
    #[track_caller]
    fn pdf(&self, _ray: Ray, _record: &HitData, _env: &Environment, _scattered: Ray) -> f64 {
        panic!("You called <() as Scattering>::pdf() - this is not implemented")
    }

    #[inline]
    #[track_caller]
    fn emitted(&self, _uv: (f64, f64), _p: Vec3) -> Colour { panic!("You called <() as Scattering>::emitted() - this is not implemented") }

    #[inline]
    #[track_caller]
    fn scatter(&self, _ray: Ray, _record: &HitData, _env: &Environment) -> (Option<Ray>, Colour, f64) {
        panic!("You called <() as Scattering>::scatter() - this is not implemented")
    }
}

// Pointer-like impls
scattering_ptr_impl!('a, &'a T);
scattering_ptr_impl!('a, &'a mut T);
scattering_ptr_impl!(Box<T>);
scattering_ptr_impl!('a, Ref<'a, T>);
scattering_ptr_impl!('a, RefMut<'a, T>);
scattering_ptr_impl!(Rc<T>);
scattering_ptr_impl!('a, RwLockReadGuard<'a, T>);
scattering_ptr_impl!('a, RwLockWriteGuard<'a, T>);
scattering_ptr_impl!('a, MutexGuard<'a, T>);
scattering_ptr_impl!(Arc<T>);
scattering_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
scattering_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
scattering_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);





/***** LIBRARY *****/
macro_rules! material_impl {
    // Default error type insertion
    (__ { $(#[$($fattrs:tt)*])+ $fty:tt $fmat:ident $(, $(#[$($rattrs:tt)*])+ $rty:tt $rmat:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__ {$($(#[$($rattrs)*])+ $rty $rmat $(($rerrty))?),*} { $(#[$($fattrs)*])+ $fty $fmat (::std::convert::Infallible) $(, $(#[$($attrs)*])+ $ty $mat ($errty))* });
    };
    (__ { $(#[$($fattrs:tt)*])+ $fty:tt $fmat:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])+ $rty:tt $rmat:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__ {$($(#[$($rattrs)*])+ $rty $rmat $(($rerrty))?),*} { $(#[$($fattrs)*])+ $fty $fmat ($ferrty) $(, $(#[$($attrs)*])+ $ty $mat ($errty))* });
    };
    (__ {} { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__2 { $($(#[$($attrs)*])+ $ty $mat ( $errty )),* } {});
    };

    // Material type resolution
    (__2 { $(#[$($fattrs:tt)*])+ * $fmat:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])+ $rty:tt $rmat:ident ( $rerrty:ty ))* } { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__2 {$($(#[$($rattrs)*])+ $rty $rmat ($rerrty)),*} { $(#[$($fattrs)*])+ Scatter $fmat ($ferrty) $(, $(#[$($attrs)*])+ $ty $mat ($errty))* });
    };
    (__2 { $(#[$($fattrs:tt)*])+ > $fmat:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])+ $rty:tt $rmat:ident ( $rerrty:ty ))* } { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__2 {$($(#[$($rattrs)*])+ $rty $rmat ($rerrty)),*} { $(#[$($fattrs)*])+ Specular $fmat ($ferrty) $(, $(#[$($attrs)*])+ $ty $mat ($errty))* });
    };
    (__2 { $(#[$($fattrs:tt)*])+ ! $fmat:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])+ $rty:tt $rmat:ident ( $rerrty:ty ))* } { $($(#[$($attrs:tt)*])+ $ty:tt $mat:ident ( $errty:ty )),* }) => {
        material_impl!(__2 {$($(#[$($rattrs)*])+ $rty $rmat ($rerrty)),*} { $(#[$($fattrs)*])+ Light $fmat ($ferrty) $(, $(#[$($attrs)*])+ $ty $mat ($errty))* });
    };


    // Actual impl
    (__2 {} { $($(#[$($attrs:tt)*])+ $ty:ident $mat:ident ( $errty:ty )),* }) => {
        /// Errors occurring when loading the material.
        #[derive(Debug, Error)]
        pub enum Error {
            $(#[error("{0}")] $mat(#[source] $errty),)*
        }



        /// A runtime abstraction of all possible materials.
        #[derive(Clone, Debug, Deserialize, Serialize)]
        #[serde(tag = "type")]
        #[serde(rename_all = "snake_case")]
        pub enum Material {
            $($(#[$($attrs)*])* $mat($mat),)*
            /// The empty material, used when defining a model and such.
            Empty,
        }

        // Constructors
        impl Default for Material {
            #[inline]
            fn default() -> Self { Self::Empty }
        }

        // Material
        impl Material {
            /// Returns the kind of the material.
            ///
            /// This is either [`ObjectKind::Scatter`] when the material has a random component
            /// to the scatter; or [`ObjectKind::Specular`] of it does not.
            #[inline]
            pub const fn kind(&self) -> ObjectKind {
                match self {
                    $(Self::$mat(_) => ObjectKind::$ty,)*
                    Self::Empty => ObjectKind::Specular,
                }
            }
        }

        // Interface
        impl Loadable for Material {
            type Error = Error;

            #[inline]
            #[track_caller]
            fn load(&mut self, dir: &Path) -> Result<(), Self::Error> {
                match self {
                    $(Self::$mat(m) => m.load(dir).map_err(Error::$mat),)*
                    Self::Empty => Ok(()),
                }
            }
        }
        impl Scattering for Material {
            #[inline]
            #[track_caller]
            fn pdf(&self, ray: Ray, record: &HitData, env: &Environment, scattered: Ray) -> f64 {
                match self {
                    $(Self::$mat(m) => m.pdf(ray, record, env, scattered),)*
                    Self::Empty => panic!("Cannot get a PDF of the empty material; please specify one"),
                }
            }

            #[inline]
            #[track_caller]
            fn emitted(&self, uv: (f64, f64), p: Vec3) -> Colour {
                match self {
                    $(Self::$mat(m) => m.emitted(uv, p),)*
                    Self::Empty => panic!("Cannot emit anything from the empty material; please specify one"),
                }
            }

            #[inline]
            #[track_caller]
            fn scatter(&self, ray: Ray, record: &HitData, env: &Environment) -> (Option<Ray>, Colour, f64) {
                match self {
                    $(Self::$mat(m) => m.scatter(ray, record, env),)*
                    Self::Empty => panic!("Cannot scatter anything off the empty material; please specify one"),
                }
            }
        }
    };

    // Public interface
    ($($(#[$($attrs:tt)*])+ $ty:tt $tex:ident $(( $errty:ty ))?),* $(,)?) => {
        material_impl!(__ { $($(#[$($attrs)*])+ $ty $tex $(($errty))?),* } {});
    };
}
material_impl!(
    // /// The empty material.
    // (),
    /// A refracting material (e.g., glass, water-on-air, etc).
    > Dielectric,
    /// A material randomly scattering rays, imperfectly.
    * Diffuse,
    /// A meterial emitted light randomly.
    ! DiffuseLight,
    /// A material randomly scattering rays.
    * Lambertian,
    /// A material randomly scattering rays but with a texture.
    * LambertianTexture(super::textures::Error),
    /// A material reflecting rays perfectly.
    > Metal,
    /// A material having colours of the object's normals.
    > NormalMap,
    /// A partially refracting material (has some holes in the math that stops is refracting).
    > PartialDielectric,
    /// A material having a static colour.
    > StaticColour,
);
