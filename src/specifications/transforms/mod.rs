//  TRANSFORMS.rs
//    by Lut99
//
//  Description:
//!   Defines transformations on objects.
//

// The modules
pub mod rotate;
pub mod translate;

// The imports/exports
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use rotate::{RotateX, RotateY, RotateZ};
use serde::{Deserialize, Serialize};
pub use translate::{AnimatedTranslate, Translate};

use super::objects::HitData;
use crate::math::{AABB, Ray, Vec3};


/***** HELPER MACROS *****/
macro_rules! transforming_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Transforming> Transforming for $ty {
            #[inline]
            fn transform_aabb(&self, t_us: u64, aabb: AABB) -> AABB { <T as Transforming>::transform_aabb(self, t_us, aabb) }

            #[inline]
            fn transform_vec3_obj(&self, t_us: u64, vec: Vec3) -> Vec3 { <T as Transforming>::transform_vec3_obj(self, t_us, vec) }

            #[inline]
            fn transform_vec3_world(&self, t_us: u64, ray: Vec3) -> Vec3 { <T as Transforming>::transform_vec3_world(self, t_us, ray) }

            #[inline]
            fn transform_ray_obj(&self, ray: Ray) -> Ray { <T as Transforming>::transform_ray_obj(self, ray) }

            #[inline]
            fn transform_ray_world(&self, ray: Ray) -> Ray { <T as Transforming>::transform_ray_world(self, ray) }

            #[inline]
            fn transform_rec_obj(&self, rec: HitData) -> HitData { <T as Transforming>::transform_rec_obj(self, rec) }

            #[inline]
            fn transform_rec_world(&self, rec: HitData) -> HitData { <T as Transforming>::transform_rec_world(self, rec) }
        }
    };
    ($ty:ty) => {
        impl<T: Transforming> Transforming for $ty {
            #[inline]
            fn transform_aabb(&self, t_us: u64, aabb: AABB) -> AABB { <T as Transforming>::transform_aabb(self, t_us, aabb) }

            #[inline]
            fn transform_vec3_obj(&self, t_us: u64, vec: Vec3) -> Vec3 { <T as Transforming>::transform_vec3_obj(self, t_us, vec) }

            #[inline]
            fn transform_vec3_world(&self, t_us: u64, ray: Vec3) -> Vec3 { <T as Transforming>::transform_vec3_world(self, t_us, ray) }

            #[inline]
            fn transform_ray_obj(&self, ray: Ray) -> Ray { <T as Transforming>::transform_ray_obj(self, ray) }

            #[inline]
            fn transform_ray_world(&self, ray: Ray) -> Ray { <T as Transforming>::transform_ray_world(self, ray) }

            #[inline]
            fn transform_rec_obj(&self, rec: HitData) -> HitData { <T as Transforming>::transform_rec_obj(self, rec) }

            #[inline]
            fn transform_rec_world(&self, rec: HitData) -> HitData { <T as Transforming>::transform_rec_world(self, rec) }
        }
    };
}





/***** INTERFACES *****/
/// Defines an abstraction over all transforming objects.
pub trait Transforming {
    // AABB
    /// Transforms an object's AABB from object space to world space.
    ///
    /// # Arguments
    /// - `t_us`: The time, in us since the start of the scene, at which the transformation needs
    ///   to occur.
    /// - `aabb`: The [`AABB`] to transform.
    ///
    /// # Returns
    /// A new [`AABB`] representing the transformed version.
    fn transform_aabb(&self, _t_us: u64, aabb: AABB) -> AABB;


    // Vec
    /// Transforms a 3D vector from "world" space to "object" space.
    ///
    /// # Arguments
    /// - `t_us`: The time, in us since the start of the scene, at which the transformation needs
    ///   to occur.
    /// - `vec`: The [`Vec3`] to transform.
    ///
    /// # Returns
    /// A new [`Vec3`] that is `vec` but in object space.
    fn transform_vec3_obj(&self, t_us: u64, vec: Vec3) -> Vec3;

    /// Transforms a 3D vector from "object" space to "world" space.
    ///
    /// # Arguments
    /// - `t_us`: The time, in us since the start of the scene, at which the transformation needs
    ///   to occur.
    /// - `vec`: The [`Vec3`] to transform.
    ///
    /// # Returns
    /// A new [`Vec3`] that is `vec` but in world space.
    fn transform_vec3_world(&self, t_us: u64, vec: Vec3) -> Vec3;


    // Ray
    /// Transforms a Ray shot at an object.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to transform.
    ///
    /// # Returns
    /// A new [`Ray`] in object space.
    fn transform_ray_obj(&self, ray: Ray) -> Ray;

    /// Transforms a Ray shot at an object back into normal space.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to transform.
    ///
    /// # Returns
    /// A new [`Ray`] in world space.
    fn transform_ray_world(&self, ray: Ray) -> Ray;


    // HitData
    /// Transforms a record.
    ///
    /// # Arguments
    /// - `rec`: The [`HitData`] in transformed space to transform back.
    ///
    /// # Returns
    /// A new [`HitData`] in object space.
    fn transform_rec_obj(&self, rec: HitData) -> HitData;

    /// Transforms a record recording a ray shot at an object back into normal space.
    ///
    /// # Arguments
    /// - `rec`: The [`HitData`] in transformed space to transform back.
    ///
    /// # Returns
    /// A new [`HitData`] in transformed space.
    fn transform_rec_world(&self, rec: HitData) -> HitData;
}

// Pointer-like impls
transforming_ptr_impl!('a, &'a T);
transforming_ptr_impl!('a, &'a mut T);
transforming_ptr_impl!(std::boxed::Box<T>);
transforming_ptr_impl!(Rc<T>);
transforming_ptr_impl!(Arc<T>);
transforming_ptr_impl!('a, Ref<'a, T>);
transforming_ptr_impl!('a, RefMut<'a, T>);
transforming_ptr_impl!('a, RwLockReadGuard<'a, T>);
transforming_ptr_impl!('a, RwLockWriteGuard<'a, T>);
transforming_ptr_impl!('a, MutexGuard<'a, T>);
transforming_ptr_impl!('a, parking_lot::RwLockReadGuard<'a, T>);
transforming_ptr_impl!('a, parking_lot::RwLockWriteGuard<'a, T>);
transforming_ptr_impl!('a, parking_lot::MutexGuard<'a, T>);





/***** LIBRARY *****/
macro_rules! transform_impl {
    ($($(#[$($attrs:tt)*])* $obj:ident),* $(,)?) => {
        /// Represents a dynamic form of any [`Transforming`] entity.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
        #[serde(tag = "type")]
        #[serde(rename_all = "snake_case")]
        pub enum Transform {
            $($(#[$($attrs)*])* $obj($obj),)*
        }

        // Interfaces
        impl Transforming for Transform {
            #[inline]
            fn transform_aabb(&self, t_us: u64, aabb: AABB) -> AABB {
                match self {
                    $(Self::$obj(o) => o.transform_aabb(t_us, aabb),)*
                }
            }

            #[inline]
            fn transform_vec3_obj(&self, t_us: u64, vec: Vec3) -> Vec3 {
                match self {
                    $(Self::$obj(o) => o.transform_vec3_obj(t_us, vec),)*
                }
            }

            #[inline]
            fn transform_vec3_world(&self, t_us: u64, vec: Vec3) -> Vec3 {
                match self {
                    $(Self::$obj(o) => o.transform_vec3_world(t_us, vec),)*
                }
            }

            #[inline]
            fn transform_ray_obj(&self, ray: Ray) -> Ray {
                match self {
                    $(Self::$obj(o) => o.transform_ray_obj(ray),)*
                }
            }

            #[inline]
            fn transform_ray_world(&self, ray: Ray) -> Ray {
                match self {
                    $(Self::$obj(o) => o.transform_ray_world(ray),)*
                }
            }

            #[inline]
            fn transform_rec_obj(&self, rec: HitData) -> HitData {
                match self {
                    $(Self::$obj(o) => o.transform_rec_obj(rec),)*
                }
            }

            #[inline]
            fn transform_rec_world(&self, rec: HitData) -> HitData {
                match self {
                    $(Self::$obj(o) => o.transform_rec_world(rec),)*
                }
            }
        }
    };
}
transform_impl!(
    /// Defines a rotation over the X-axis.
    RotateX,
    /// Defines a rotation over the Y-axis.
    RotateY,
    /// Defines a rotation over the Z-axis.
    RotateZ,
    /// Defines a translation in space, over time.
    AnimatedTranslate,
    /// Defines a translation in space.
    Translate,
);
