//  VOLUMES.rs
//    by Lut99
//
//  Description:
//!   Defines so-called volumes, which turn a two-sides object into something
//!   hitting sub-surface.
//

// Modules
pub mod constant;

// Imports/Exports
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use constant::ConstantDensity;
use serde::{Deserialize, Serialize};

use super::materials::{ScatterRecord, Scattering};
use super::objects::HitData;
use super::objects::pdf::PDF;
use super::scene::Environment;
use crate::math::{Colour, Ray, Vec3};


/***** HELPER MACROS *****/
macro_rules! volumizing_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Volumizing> Volumizing for $ty {
            #[inline]
            fn volumize(&self, ray: Ray, t_front: f64, t_back: f64) -> Option<HitData> { <T as Volumizing>::volumize(self, ray, t_front, t_back) }
        }
    };
    ($ty:ty) => {
        impl<'a, T: Volumizing> Volumizing for $ty {
            #[inline]
            fn volumize(&self, ray: Ray, t_front: f64, t_back: f64) -> Option<HitData> { <T as Volumizing>::volumize(self, ray, t_front, t_back) }
        }
    };
}





/***** INTERFACES *****/
/// Defines something that turns a front/back sides, hollow object into a "volumous" object, i.e.,
/// one that may also hit _in between_ the sides.
pub trait Volumizing: Scattering {
    /// Given a ray and two points along it where it hits the object (at the front and at the
    /// back), compute a hit somewhere _within_ the volume.
    ///
    /// For example, for [`ConstantDensity`] volumes, this (might!) randomly scatter at a point in
    /// the cloud.
    ///
    /// Note that this just computes the hitpoint - _how_ it is scattered depends on the material.
    ///
    /// # Arguments
    /// - `ray`
    fn volumize(&self, ray: Ray, t_front: f64, t_back: f64) -> Option<HitData>;
}

// Pointer-like impls
volumizing_ptr_impl!('a, &'a T);
volumizing_ptr_impl!('a, &'a mut T);
volumizing_ptr_impl!(std::boxed::Box<T>);
volumizing_ptr_impl!(Rc<T>);
volumizing_ptr_impl!(Arc<T>);
volumizing_ptr_impl!('a, Ref<'a, T>);
volumizing_ptr_impl!('a, RefMut<'a, T>);
volumizing_ptr_impl!('a, RwLockReadGuard<'a, T>);
volumizing_ptr_impl!('a, RwLockWriteGuard<'a, T>);
volumizing_ptr_impl!('a, MutexGuard<'a, T>);
volumizing_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
volumizing_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
volumizing_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);





/***** LIBRARY *****/
macro_rules! volume_impl {
    ($($(#[$($attrs:tt)*])* $volume:ident),* $(,)?) => {
        /// Represents the possible PDFs for [`Volume`].
        #[derive(Clone, Copy, Debug)]
        pub enum VolumePDF {
            $($volume(<$volume as Scattering>::PDF),)*
        }

        // Interfaces
        impl PDF for VolumePDF {
            #[inline]
            fn value(&self, direct: Ray, env: &Environment) -> f64 {
                match self {
                    $(Self::$volume(p) => p.value(direct, env),)*
                }
            }

            #[inline]
            fn sample(&self, t_us: u64, origin: Vec3) -> Vec3 {
                match self {
                    $(Self::$volume(p) => p.sample(t_us, origin),)*
                }
            }
        }



        /// Represents a dynamic form of any [`Volumizing`] entity.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        #[serde(untagged)]
        #[serde(rename_all = "snake_case")]
        pub enum Volume {
            $($(#[$($attrs)*])* $volume($volume),)*
        }

        // Interfaces
        impl Scattering for Volume {
            type PDF = VolumePDF;

            #[inline]
            #[track_caller]
            fn pdf(&self, ray: Ray, record: &HitData, env: &Environment, scattered: Ray) -> f64 {
                match self {
                    $(Self::$volume(v) => v.pdf(ray, record, env, scattered),)*
                }
            }

            #[inline]
            #[track_caller]
            fn emitted(&self, rec: &HitData) -> Colour {
                match self {
                    $(Self::$volume(v) => v.emitted(rec),)*
                }
            }

            #[inline]
            #[track_caller]
            fn scatter(&self, ray: Ray, record: &HitData, env: &Environment) -> Option<ScatterRecord<Self::PDF>> {
                match self {
                    $(Self::$volume(v) => v.scatter(ray, record, env).map(|r| r.map_pdf(VolumePDF::$volume)),)*
                }
            }
        }
        impl Volumizing for Volume {
            #[inline]
            fn volumize(&self, ray: Ray, t_front: f64, t_back: f64) -> Option<HitData> {
                match self {
                    $(Self::$volume(v) => v.volumize(ray, t_front, t_back),)*
                }
            }
        }
    };
}
volume_impl!(
    /// An object with a constant density that might randomly hit - e.g., a gas cloud.
    ConstantDensity,
);
