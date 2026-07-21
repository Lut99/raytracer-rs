//  MOD.rs
//    by Lut99
//
//  Created:
//    01 May 2023, 18:54:46
//  Last edited:
//    05 May 2023, 11:17:42
//  Auto updated?
//    Yes
//
//  Description:
//!   The `objects` module defines the objects to which we can render. It
//!   is structured object-oriented _like_, but because we use our
//!   ECS-like [`crate::hitlist::HitList`] and we never turn the objects
//!   into dynamic trait instances, we won't have the downsides of virtual
//!   function pointers.
//

// Define the submodules
mod hitrecord;
#[cfg(feature = "obj")]
pub mod model;
pub mod plane;
pub mod sphere;

// Imports & Exports
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use hitrecord::*;
pub use plane::{Quad, Vertex};
use serde::{Deserialize, Serialize};
pub use sphere::{AnimatedSphere, Sphere};
use thiserror::Error;

use super::Loadable;
use super::materials::{Material, Scattering};
use super::scene::Environment;
use crate::math::{AABB, Colour, Ray};


/***** MACRO RULES *****/
macro_rules! bounding_boxable_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: BoundingBoxable> BoundingBoxable for $ty {
            #[inline]
            fn aabb(&self, t_us: u64) -> AABB { <T as BoundingBoxable>::aabb(self, t_us) }
        }
    };
    ($ty:ty) => {
        impl<T: BoundingBoxable> BoundingBoxable for $ty {
            #[inline]
            fn aabb(&self, t_us: u64) -> AABB { <T as BoundingBoxable>::aabb(self, t_us) }
        }
    };
}

macro_rules! hittable_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Hittable> Hittable for $ty {
            #[inline]
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord> {
                <T as Hittable>::hit(self, ray, t_min, t_max, env)
            }
        }
    };
    ($ty:ty) => {
        impl<T: Hittable> Hittable for $ty {
            #[inline]
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord> {
                <T as Hittable>::hit(self, ray, t_min, t_max, env)
            }
        }
    };
}





/***** INTERFACE *****/
/// Defines a common interface for objects that can compute a sensible [`AABB`].
pub trait BoundingBoxable {
    /// Computes the Axis-Aligned Bounding Box (AABB) of this object.
    ///
    /// # Arguments
    /// - `t_us`: The time at which we compute the bounding box. Matters if this object is
    ///   animated. Time is in microseconds since the start of the scene.
    ///
    /// # Returns
    /// A new [`AABB`] struct that describes the computed bounding box.
    fn aabb(&self, t_us: u64) -> AABB;
}

// Pointer-like impls
bounding_boxable_ptr_impl!('a, &'a T);
bounding_boxable_ptr_impl!('a, &'a mut T);
bounding_boxable_ptr_impl!(Box<T>);
bounding_boxable_ptr_impl!(Rc<T>);
bounding_boxable_ptr_impl!(Arc<T>);
bounding_boxable_ptr_impl!('a, Ref<'a, T>);
bounding_boxable_ptr_impl!('a, RefMut<'a, T>);
bounding_boxable_ptr_impl!('a, RwLockReadGuard<'a, T>);
bounding_boxable_ptr_impl!('a, RwLockWriteGuard<'a, T>);
bounding_boxable_ptr_impl!('a, MutexGuard<'a, T>);
bounding_boxable_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
bounding_boxable_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
bounding_boxable_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);



/// Defines the functions that hittable objects have in common.
pub trait Hittable: BoundingBoxable {
    /// Computes any hitpoints of the given ray with this object.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute any hits with.
    /// - `t_min`: The minimum point along the ray we still accept (we don't count it as a hit before that).
    /// - `t_max`: The maximum point along the ray we still accept (we don't count is as a hit after that).
    /// - `env`: An [`Environment`] struct relating information about the scene's total environment.
    ///
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else [`None`] if the ray does not hit.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord>;
}

// Pointer-like impls
hittable_ptr_impl!('a, &'a T);
hittable_ptr_impl!('a, &'a mut T);
hittable_ptr_impl!(Box<T>);
hittable_ptr_impl!(Rc<T>);
hittable_ptr_impl!(Arc<T>);
hittable_ptr_impl!('a, Ref<'a, T>);
hittable_ptr_impl!('a, RefMut<'a, T>);
hittable_ptr_impl!('a, RwLockReadGuard<'a, T>);
hittable_ptr_impl!('a, RwLockWriteGuard<'a, T>);
hittable_ptr_impl!('a, MutexGuard<'a, T>);
hittable_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
hittable_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
hittable_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);





/***** LIBRARY *****/
macro_rules! object_impl {
    // Default error type insertion
    (__ { $(#[$($fattrs:tt)*])* $fobj:ident $(, $(#[$($rattrs:tt)*])* $robj:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $obj:ident ( $errty:ty )),* }) => {
        object_impl!(__ {$($(#[$($rattrs)*])* $robj $(($rerrty))?),*} { $(#[$($fattrs)*])* $fobj (::std::convert::Infallible) $(, $(#[$($attrs)*])* $obj ($errty))* });
    };
    (__ { $(#[$($fattrs:tt)*])* $fobj:ident ($ferrty:ty) $(, $(#[$($rattrs:tt)*])* $robj:ident $(( $rerrty:ty ))?)* } { $($(#[$($attrs:tt)*])* $obj:ident ( $errty:ty )),* }) => {
        object_impl!(__ {$($(#[$($rattrs)*])* $robj $(($rerrty))?),*} { $(#[$($fattrs)*])* $fobj ($ferrty) $(, $(#[$($attrs)*])* $obj ($errty))* });
    };


    // Actual impl
    (__ {} { $($(#[$($attrs:tt)*])* $obj:ident ( $errty:ty )),* }) => {
        /// Errors occurring when loading an object.
        #[derive(Debug, Error)]
        pub enum Error {
            $(#[error("{0}")] $obj(#[source] $errty),)*
        }



        /// A runtime abstraction of all possible objects.
        ///
        /// # Generics
        /// - `M`: The type of material used.
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub enum Object {
            $($(#[$($attrs)*])* $obj($obj<Material>),)*
        }

        // Interface
        impl Loadable for Object {
            type Error = Error;

            #[inline]
            fn load(&mut self) -> Result<(), Self::Error> {
                match self {
                    $(Self::$obj(o) => o.load().map_err(Error::$obj),)*
                }
            }
        }
        impl BoundingBoxable for Object {
            #[inline]
            fn aabb(&self, t_us: u64) -> AABB {
                match self {
                    $(Self::$obj(o) => o.aabb(t_us),)*
                }
            }
        }
        impl Hittable for Object {
            #[inline]
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord> {
                match self {
                    $(Self::$obj(o) => o.hit(ray, t_min, t_max, env),)*
                }
            }
        }
        impl Scattering for Object {
            #[inline]
            fn scatter(&self, ray: Ray, record: HitRecord, env: &Environment) -> (Option<Ray>, Colour) {
                match self {
                    $(Self::$obj(o) => o.scatter(ray, record, env),)*
                }
            }
        }
    };

    // Public interface
    ($($(#[$($attrs:tt)*])* $obj:ident $(( $errty:ty ))?),* $(,)?) => {
        object_impl!(__ { $($(#[$($attrs)*])? $obj $(($errty))?),* } {});
    };
}
object_impl!(
    /// A regular sphere but animated.
    AnimatedSphere(super::materials::Error),
    /// A regular 3D circle.
    Sphere(super::materials::Error),
    /// A four-point shape on a 2D-plane.
    Quad(super::materials::Error),
    /// A three-point shape on a 2D-plane.
    Vertex(super::materials::Error),
);
