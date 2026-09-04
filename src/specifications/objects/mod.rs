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
pub mod boxed;
pub mod group;
mod hitrecord;
#[cfg(feature = "obj")]
pub mod model;
pub mod plane;
pub mod sphere;

// Imports & Exports
use std::cell::{Ref, RefMut};
use std::convert::Infallible;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use boxed::Box;
pub use group::Group;
pub use hitrecord::*;
pub use model::Model;
pub use plane::{Quad, Triangle};
use serde::{Deserialize, Serialize};
pub use sphere::Sphere;
use thiserror::Error;

use super::Loadable;
use super::materials::{Material, Scattering};
use super::scene::Environment;
use super::transforms::{RotateX, RotateY, RotateZ, Transform, Transforming as _, Translate};
use super::volumes::{Volume, Volumizing as _};
use crate::math::{AABB, Ray};


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
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> {
                <T as Hittable>::hit(self, ray, t_min, t_max, env)
            }
        }
    };
    ($ty:ty) => {
        impl<T: Hittable> Hittable for $ty {
            #[inline]
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> {
                <T as Hittable>::hit(self, ray, t_min, t_max, env)
            }
        }
    };
}





/***** ERRORS *****/
/// Defines errors occurring when [loading](Loadable::load()) [`Object`]s.
#[derive(Debug, Error)]
pub enum JsonObjectLoadError {
    /// The model failed.
    #[error("{0}")]
    Model(#[from] model::Error),
    /// The object failed.
    #[error("{0}")]
    Object(#[source] <Object<DynObject, Material> as Loadable>::Error),
    /// The group failed.
    #[error("{0}")]
    Group(#[source] std::boxed::Box<<Group as Loadable>::Error>),
}

/// Defines errors occurring when [loading](Loadable::load()) [`Object`]s.
#[derive(Debug, Error)]
pub enum ObjectLoadError<E1, E2> {
    /// The object failed.
    #[error("{0}")]
    Obj(#[source] E1),
    /// The material failed.
    #[error("{0}")]
    Mat(#[source] E2),
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
bounding_boxable_ptr_impl!(std::boxed::Box<T>);
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
    /// A new [`HitData`] struct, which collects relevant information of this hit, or else [`None`] if the ray does not hit.
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData>;
}

// Pointer-like impls
hittable_ptr_impl!('a, &'a T);
hittable_ptr_impl!('a, &'a mut T);
hittable_ptr_impl!(std::boxed::Box<T>);
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
/// Defines either a single object, or a group.
///
/// Used only to complete serialization.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum JsonObject {
    /// It's an unloaded model.
    Model(Model),
    /// It's a loose object.
    Object(Object<DynObject, Material>),
    /// It's a group.
    Group(Group),
}

// Interfaces
impl Loadable for JsonObject {
    type Error = JsonObjectLoadError;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> {
        match self {
            Self::Model(m) => {
                // Attempt to load the model into a group
                let group = m.load(dir)?;
                // Replace the object with that group
                *self = JsonObject::Group(group);
                Ok(())
            },
            Self::Object(o) => o.load(dir).map_err(JsonObjectLoadError::Object),
            Self::Group(g) => g.load(dir).map_err(std::boxed::Box::new).map_err(JsonObjectLoadError::Group),
        }
    }
}



/// Defines the wrapper around an [`Object`], including any modifiers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Object<T, M> {
    /// Defines the actual object that implements itself.
    #[serde(flatten)]
    pub obj: T,
    /// Defines the material on the object.
    #[serde(default, alias = "material")]
    pub mat: M,
    /// Defines if this object is volumized somehow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volumized: Option<Volume>,
    /// Defines any transformations on the object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,
}

// Object
impl<T, M> Object<T, M> {
    /// Optimizes the transforms in this object by consolidating them.
    pub fn consolidate_transforms(&mut self) {
        let mut transforms = Vec::new();
        let mut prev = None;
        for transform in self.transforms.drain(..) {
            match (prev, transform) {
                // Group equal transforms in a row
                (Some(Transform::RotateX(p)), Transform::RotateX(t)) => prev = Some(Transform::RotateX(RotateX { angle: p.angle + t.angle })),
                (Some(Transform::RotateY(p)), Transform::RotateY(t)) => prev = Some(Transform::RotateY(RotateY { angle: p.angle + t.angle })),
                (Some(Transform::RotateZ(p)), Transform::RotateZ(t)) => prev = Some(Transform::RotateZ(RotateZ { angle: p.angle + t.angle })),
                (Some(Transform::Translate(p)), Transform::Translate(t)) => prev = Some(Transform::Translate(Translate { pos: p.pos + t.pos })),

                // If they are equal, then push the prev
                (Some(p), t) => {
                    transforms.push(p);
                    prev = Some(t);
                },

                // Otherwise, if there is no prev, then this becomes the prev
                (None, transform) => prev = Some(transform),
            }
        }
        if let Some(prev) = prev {
            transforms.push(prev);
        }
        self.transforms = transforms;
    }
}

// Interfaces
impl<T: Loadable, M: Loadable> Loadable for Object<T, M>
where
    T::Error: 'static,
    M::Error: 'static,
{
    type Error = ObjectLoadError<T::Error, M::Error>;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> {
        self.obj.load(dir).map_err(ObjectLoadError::Obj)?;
        self.mat.load(dir).map_err(ObjectLoadError::Mat)?;
        Ok(())
    }
}
impl<T: BoundingBoxable, M> BoundingBoxable for Object<T, M> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB {
        // Apply the transformations to the computed AABB
        let mut aabb: AABB = self.obj.aabb(t_us);
        for trans in &self.transforms {
            aabb = trans.transform_aabb(aabb);
        }
        aabb
    }
}
impl<T: Hittable, M> Hittable for Object<T, M> {
    #[inline]
    fn hit(&self, mut ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> {
        // First, transform the ray on the way there...
        for trans in self.transforms.iter() {
            ray = trans.transform(ray);
        }

        // Then decide how to hit the object
        let mut rec: HitData = if let Some(volume) = &self.volumized {
            // Compute both hits - one on the front of the object, one on the back. And they must
            // both hit!
            let mut rec1: HitData = self.obj.hit(ray, -f64::INFINITY, f64::INFINITY, env)?;
            let mut rec2: HitData = self.obj.hit(ray, rec1.t + 0.0001, f64::INFINITY, env)?;

            // Bound the record's t's by the given ones and quit if it's too close
            rec1.t = f64::max(rec1.t, t_min);
            rec2.t = f64::min(rec2.t, t_max);
            if rec1.t >= rec2.t {
                return None;
            }
            rec1.t = f64::max(rec1.t, 0.0);

            // Then run the volume function to turn that ray into a final hit
            volume.volumize(ray, rec1.t, rec2.t)?
        } else {
            // Compute the hit
            self.obj.hit(ray, t_min, t_max, env)?
        };

        // Transform the result back
        for trans in self.transforms.iter().rev() {
            rec = trans.transform_back(rec);
        }
        Some(rec)
    }
}
impl<T: Hittable, M: Scattering> Object<T, M> {
    /// Computes whether this Object is [`hit()`](Hittable::hit()), except that the relevant
    /// material is also returned.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute any hits with.
    /// - `t_min`: The minimum point along the ray we still accept (we don't count it as a hit before that).
    /// - `t_max`: The maximum point along the ray we still accept (we don't count is as a hit after that).
    /// - `env`: An [`Environment`] struct relating information about the scene's total environment.
    ///
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else [`None`] if the ray does not hit.
    #[inline]
    pub fn hit_full(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<'_>> {
        self.obj.hit(ray, t_min, t_max, env).map(|data| HitRecord { data, mat: &self.mat })
    }
}



macro_rules! dyn_object_impl {
    ($($(#[$($attrs:tt)*])* $obj:ident $({$($gen:tt)*})? ( $errty:ty )),* $(,)?) => {
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
        #[serde(tag = "type")]
        #[serde(rename_all = "snake_case")]
        pub enum DynObject {
            $($(#[$($attrs)*])* $obj($obj$(<$($gen)*>)?),)*
        }

        // Interface
        impl Loadable for DynObject {
            type Error = Error;

            #[inline]
            fn load(&mut self, dir: &Path) -> Result<(), Self::Error> {
                match self {
                    $(Self::$obj(o) => o.load(dir).map_err(Error::$obj),)*
                }
            }
        }
        impl BoundingBoxable for DynObject {
            #[inline]
            fn aabb(&self, t_us: u64) -> AABB {
                match self {
                    $(Self::$obj(o) => o.aabb(t_us),)*
                }
            }
        }
        impl Hittable for DynObject {
            #[inline]
            fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> {
                match self {
                    $(Self::$obj(o) => o.hit(ray, t_min, t_max, env),)*
                }
            }
        }
    };
}
dyn_object_impl!(
    /// A bunch of quads that make a box shape.
    Box(Infallible),
    /// A four-point shape on a 2D-plane.
    Quad(Infallible),
    /// A regular 3D circle.
    Sphere(Infallible),
    /// A three-point shape on a 2D-plane.
    Triangle(Infallible),
);
