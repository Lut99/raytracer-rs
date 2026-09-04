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

use super::objects::HitData;
use crate::math::Ray;


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
pub trait Volumizing {
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
        /// Represents a dynamic form of any [`Volumizing`] entity.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        #[serde(untagged)]
        #[serde(rename_all = "snake_case")]
        pub enum Volume {
            $($(#[$($attrs)*])* $volume($volume),)*
        }

        // Interfaces
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
